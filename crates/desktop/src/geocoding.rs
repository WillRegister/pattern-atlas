//! Explicit, asynchronous city searches. Never sends requests on keystrokes.
use anyhow::{ensure, Context, Result};
use atlas_core::model::Place;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{mpsc, Mutex},
    time::{Duration, Instant},
};

static LAST_REQUEST: Mutex<Option<Instant>> = Mutex::new(None);
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Candidate {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub elevation: Option<f64>,
    pub timezone: String,
    #[serde(default)]
    pub admin1: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub country_code: String,
}
impl Candidate {
    pub fn label(&self) -> String {
        [
            self.name.as_str(),
            self.admin1.as_str(),
            self.country.as_str(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
    }
    pub fn place(&self) -> Result<Place> {
        ensure!(
            self.latitude.is_finite() && (-90. ..=90.).contains(&self.latitude),
            "Invalid latitude from provider"
        );
        ensure!(
            self.longitude.is_finite() && (-180. ..=180.).contains(&self.longitude),
            "Invalid longitude from provider"
        );
        let elevation = self.elevation.unwrap_or(0.);
        ensure!(
            elevation.is_finite() && (-12000. ..=100000.).contains(&elevation),
            "Invalid elevation from provider"
        );
        self.timezone
            .parse::<chrono_tz::Tz>()
            .context("Unknown timezone from provider")?;
        let mut p = Place::new(&self.label());
        p.latitude = self.latitude;
        p.longitude = self.longitude;
        p.elevation_m = elevation;
        p.timezone = self.timezone.clone();
        p.context = "City reference point from GeoNames via Open-Meteo; not a street address or exact venue position.".into();
        p.source = "Open-Meteo Geocoding / GeoNames · https://open-meteo.com/en/docs/geocoding-api · CC BY 4.0".into();
        Ok(p)
    }
}
#[derive(Deserialize)]
struct Reply {
    #[serde(default)]
    results: Vec<Candidate>,
}
pub struct Lookup {
    pub city: String,
    pub state: String,
    pub country: String,
    pub results: Vec<Candidate>,
    pub message: String,
    receiver: Option<mpsc::Receiver<Result<Vec<Candidate>, String>>>,
    cache: BTreeMap<String, Vec<Candidate>>,
    cache_path: PathBuf,
}
impl Lookup {
    pub fn new(data_dir: PathBuf) -> Self {
        let cache_path = data_dir.join("city-search-cache.json");
        let cache = std::fs::read(&cache_path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        Self {
            city: String::new(),
            state: String::new(),
            country: "US".into(),
            results: vec![],
            message: String::new(),
            receiver: None,
            cache,
            cache_path,
        }
    }
    pub fn pending(&self) -> bool {
        self.receiver.is_some()
    }
    pub fn search(&mut self, ctx: eframe::egui::Context) {
        if self.pending() {
            return;
        }
        self.results.clear();
        if self.city.trim().chars().count() < 2 {
            self.message = "Enter a city name with at least two characters".into();
            return;
        }
        let country = self.country.trim().to_uppercase();
        if !country.is_empty()
            && (country.len() != 2 || !country.bytes().all(|c| c.is_ascii_alphabetic()))
        {
            self.message = "Country must be a two-letter code, e.g. US, or blank".into();
            return;
        }
        let query = if self.state.trim().is_empty() {
            self.city.trim().to_owned()
        } else {
            format!("{}, {}", self.city.trim(), self.state.trim())
        };
        let endpoint = std::env::var("ATLAS_GEOCODING_URL")
            .unwrap_or_else(|_| "https://geocoding-api.open-meteo.com/v1/search".into());
        let key = format!("{endpoint}|{}|{country}", query.to_lowercase());
        if let Some(results) = self.cache.get(&key) {
            self.results = results.clone();
            self.message = "Cached results · choose a city below".into();
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        self.message = "Searching cities…".into();
        let cache_path = self.cache_path.clone();
        let mut cache = self.cache.clone();
        std::thread::spawn(move || {
            let result = fetch(&endpoint, &query, &country)
                .map_err(|e| format!("City search failed: {e:#}"));
            if let Ok(results) = &result {
                cache.insert(key, results.clone());
                // Keep a bounded cache; it is separate from canonical workspace data.
                while cache.len() > 200 {
                    if let Some(key) = cache.keys().next().cloned() {
                        cache.remove(&key);
                    }
                }
                if let Ok(bytes) = serde_json::to_vec(&cache) {
                    let _ = std::fs::write(cache_path, bytes);
                }
            }
            let _ = tx.send(result);
            ctx.request_repaint();
        });
    }
    pub fn poll(&mut self) {
        if let Some(rx) = &self.receiver {
            match rx.try_recv() {
                Ok(result) => {
                    self.receiver = None;
                    match result {
                        Ok(results) => {
                            self.results = results;
                            self.message = if self.results.is_empty() {
                                "No matches. Check city/state spelling or try the full state name."
                            } else {
                                "Choose a matching city; coordinates are city reference points."
                            }
                            .into();
                        }
                        Err(message) => self.message = message,
                    }
                    if let Ok(bytes) = std::fs::read(&self.cache_path) {
                        if let Ok(cache) = serde_json::from_slice(&bytes) {
                            self.cache = cache;
                        }
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.receiver = None;
                    self.message = "City search stopped; try again".into();
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
    }
}
fn fetch(endpoint: &str, query: &str, country: &str) -> Result<Vec<Candidate>> {
    let mut gate = LAST_REQUEST
        .lock()
        .map_err(|_| anyhow::anyhow!("Search rate limiter unavailable"))?;
    if let Some(last) = *gate {
        let elapsed = last.elapsed();
        if elapsed < Duration::from_secs(1) {
            std::thread::sleep(Duration::from_secs(1) - elapsed);
        }
    }
    *gate = Some(Instant::now());
    drop(gate);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("PatternAtlas/0.4 (personal research desktop)")
        .build()?;
    let mut params = vec![
        ("name", query),
        ("count", "10"),
        ("language", "en"),
        ("format", "json"),
    ];
    if !country.is_empty() {
        params.push(("countryCode", country));
    }
    let reply: Reply = client
        .get(endpoint)
        .query(&params)
        .send()?
        .error_for_status()?
        .json()?;
    for c in &reply.results {
        c.place()?;
    }
    Ok(reply.results)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coordinates_timezone_and_city_reference_provenance_are_preserved() {
        let c: Candidate = serde_json::from_str(r#"{"name":"Springfield","admin1":"Illinois","country":"United States","latitude":39.8,"longitude":-89.6,"timezone":"America/Chicago"}"#).unwrap();
        let p = c.place().unwrap();
        assert_eq!(p.timezone, "America/Chicago");
        assert_eq!(p.latitude, 39.8);
        assert!(p.source.contains("GeoNames"));
        let mut invalid = c;
        invalid.latitude = 91.;
        assert!(invalid.place().is_err());
    }
    #[test]
    fn http_lookup_encodes_city_state_and_country_and_handles_empty_results() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0; 4096];
            let size = stream.read(&mut buffer).unwrap();
            let request = String::from_utf8_lossy(&buffer[..size]);
            assert!(request.contains("name=Springfield%2C+Illinois"));
            assert!(request.contains("countryCode=US"));
            assert!(request.to_lowercase().contains("patternatlas/0.4"));
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 14\r\nConnection: close\r\n\r\n{\"results\":[]}").unwrap();
        });
        assert!(fetch(
            &format!("http://{address}/search"),
            "Springfield, Illinois",
            "US"
        )
        .unwrap()
        .is_empty());
        server.join().unwrap();
    }
    #[test]
    #[ignore = "Explicit live provider smoke check; normal tests stay offline"]
    fn live_city_lookup_resolves_central_timezone() {
        let results = fetch(
            "https://geocoding-api.open-meteo.com/v1/search",
            "Springfield, Illinois",
            "US",
        )
        .unwrap();
        let city = results
            .iter()
            .find(|c| c.name == "Springfield" && c.admin1 == "Illinois")
            .expect("Springfield IL result");
        assert_eq!(city.timezone, "America/Chicago");
        assert!((city.latitude - 39.8).abs() < 0.2);
        assert!((city.longitude + 89.65).abs() < 0.2);
        city.place().unwrap();
    }
}
