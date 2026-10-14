use anyhow::{ensure, Result};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// Astronomy Engine has mutable internal caches. Serialize all calls through this adapter.
static ENGINE: Mutex<()> = Mutex::new(());
unsafe extern "C" {
    fn atlas_sky(
        year: i32,
        month: i32,
        day: i32,
        hour: i32,
        minute: i32,
        second: f64,
        latitude: f64,
        longitude: f64,
        elevation: f64,
        values: *mut f64,
        global: *mut f64,
    ) -> i32;
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Observer {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_m: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BodyPosition {
    pub name: String,
    pub azimuth: f64,
    pub altitude: f64,
    pub apparent_altitude: f64,
    pub ra_hours: f64,
    pub declination: f64,
    pub distance_au: f64,
    pub ecliptic_longitude: f64,
    pub ecliptic_latitude: f64,
}
impl BodyPosition {
    pub fn altitude_for(&self, refraction: bool) -> f64 {
        if refraction {
            self.apparent_altitude
        } else {
            self.altitude
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkySnapshot {
    pub utc: DateTime<Utc>,
    pub observer: Observer,
    pub bodies: Vec<BodyPosition>,
    pub subsolar_latitude: f64,
    pub subsolar_longitude: f64,
    pub moon_phase: f64,
    pub moon_illuminated_fraction: f64,
    pub sidereal_hours: f64,
}
pub fn snapshot(observer: Observer, utc: DateTime<Utc>) -> Result<SkySnapshot> {
    ensure!((1700..=2200).contains(&utc.year()),"Sky calculations support years 1700–2200 in this build; calendar records can use other dates");
    ensure!(
        observer.latitude.is_finite() && (-90. ..=90.).contains(&observer.latitude),
        "Invalid observer latitude"
    );
    ensure!(
        observer.longitude.is_finite() && (-180. ..=180.).contains(&observer.longitude),
        "Invalid observer longitude"
    );
    ensure!(
        observer.elevation_m.is_finite() && (-12000. ..=100000.).contains(&observer.elevation_m),
        "Sky observer elevation must be −12000 to 100000 meters"
    );
    let mut values = [0.; 72];
    let mut global = [0.; 5];
    let _guard = ENGINE
        .lock()
        .map_err(|_| anyhow::anyhow!("Astronomy engine lock failed"))?;
    // SAFETY: both pointers refer to correctly sized writable arrays; scalar inputs are validated.
    let status = unsafe {
        atlas_sky(
            utc.year(),
            utc.month() as i32,
            utc.day() as i32,
            utc.hour() as i32,
            utc.minute() as i32,
            utc.second() as f64 + utc.nanosecond() as f64 / 1e9,
            observer.latitude,
            observer.longitude,
            observer.elevation_m,
            values.as_mut_ptr(),
            global.as_mut_ptr(),
        )
    };
    ensure!(status == 0, "Astronomy calculation failed ({status})");
    ensure!(
        values.iter().chain(global.iter()).all(|v| v.is_finite()),
        "Astronomy returned non-finite coordinates"
    );
    let names = [
        "Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune",
    ];
    let bodies = names
        .iter()
        .zip(values.chunks_exact(8))
        .map(|(name, v)| BodyPosition {
            name: (*name).into(),
            azimuth: v[0],
            altitude: v[1],
            apparent_altitude: v[2],
            ra_hours: v[3],
            declination: v[4],
            distance_au: v[5],
            ecliptic_longitude: v[6],
            ecliptic_latitude: v[7],
        })
        .collect();
    Ok(SkySnapshot {
        utc,
        observer,
        bodies,
        subsolar_latitude: global[1],
        subsolar_longitude: wrap_signed((global[0] - global[2]) * 15.),
        moon_phase: global[3],
        moon_illuminated_fraction: global[4],
        sidereal_hours: global[2],
    })
}
pub fn wrap_signed(angle: f64) -> f64 {
    (angle + 180.).rem_euclid(360.) - 180.
}
pub fn solar_altitude(latitude: f64, longitude: f64, sky: &SkySnapshot) -> f64 {
    let lat = latitude.to_radians();
    let dec = sky.subsolar_latitude.to_radians();
    let hour = (longitude - sky.subsolar_longitude).to_radians();
    (lat.sin() * dec.sin() + lat.cos() * dec.cos() * hour.cos())
        .clamp(-1., 1.)
        .asin()
        .to_degrees()
}
pub fn angular_separation(a: &BodyPosition, b: &BodyPosition) -> f64 {
    let da = a.declination.to_radians();
    let db = b.declination.to_radians();
    let dra = ((a.ra_hours - b.ra_hours) * 15.).to_radians();
    (da.sin() * db.sin() + da.cos() * db.cos() * dra.cos())
        .clamp(-1., 1.)
        .acos()
        .to_degrees()
}
