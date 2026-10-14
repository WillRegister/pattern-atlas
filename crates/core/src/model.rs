use crate::cipher::Cipher;
use anyhow::{ensure, Result};
use chrono::{NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use uuid::Uuid;

pub fn id() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    #[serde(default)]
    pub place_id: Option<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, serde_json::Value>,
    pub id: String,
    pub name: String,
    pub kind: String,
    pub aliases: Vec<String>,
    pub context: String,
    pub tags: Vec<String>,
    pub source: String,
}
impl Entity {
    pub fn new(name: &str) -> Self {
        Self {
            place_id: None,
            attributes: BTreeMap::new(),
            id: id(),
            name: name.into(),
            kind: "person".into(),
            aliases: vec![],
            context: String::new(),
            tags: vec![],
            source: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Place {
    #[serde(default)]
    pub attributes: BTreeMap<String, serde_json::Value>,
    pub id: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_m: f64,
    pub timezone: String,
    pub context: String,
    pub source: String,
}
impl Place {
    pub fn new(name: &str) -> Self {
        Self {
            attributes: BTreeMap::new(),
            id: id(),
            name: name.into(),
            latitude: 0.,
            longitude: 0.,
            elevation_m: 0.,
            timezone: "UTC".into(),
            context: String::new(),
            source: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    #[serde(default)]
    pub attributes: BTreeMap<String, serde_json::Value>,
    pub id: String,
    pub entity_id: Option<String>,
    pub label: String,
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
    pub timezone: String,
    pub place_id: Option<String>,
    pub context: String,
    pub source: String,
}
impl Observation {
    pub fn unlinked(date: NaiveDate) -> Self {
        let mut o = Self::new("", date);
        o.entity_id = None;
        o
    }
    pub fn new(entity_id: &str, date: NaiveDate) -> Self {
        Self {
            attributes: BTreeMap::new(),
            id: id(),
            entity_id: Some(entity_id.into()),
            label: "event".into(),
            date,
            time: None,
            timezone: "UTC".into(),
            place_id: None,
            context: String::new(),
            source: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dataset {
    #[serde(default)]
    pub import_batches: Vec<String>,
    pub id: String,
    pub name: String,
    pub context: String,
    pub entities: Vec<Entity>,
    pub places: Vec<Place>,
    pub observations: Vec<Observation>,
}
impl Dataset {
    pub fn empty(name: &str) -> Self {
        Self {
            import_batches: vec![],
            id: id(),
            name: name.into(),
            context: String::new(),
            entities: vec![],
            places: vec![],
            observations: vec![],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Rounding {
    Raw,
    Nearest,
    Floor,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LeapDay {
    February28,
    March1,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_pair_limit")]
    pub pair_limit: u32,
    #[serde(default)]
    pub atmospheric_refraction: bool,
    pub ciphers: Vec<Cipher>,
    pub rounding: Rounding,
    pub decimals: u32,
    pub kilometers: bool,
    pub miles: bool,
    pub meters: bool,
    pub feet: bool,
    pub years: bool,
    pub months: bool,
    pub weeks: bool,
    pub days: bool,
    pub include_end: bool,
    pub leap_day: LeapDay,
    pub include_zero: bool,
}
fn default_pair_limit() -> u32 {
    10_000
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            pair_limit: default_pair_limit(),
            atmospheric_refraction: false,
            ciphers: vec![
                Cipher::Ordinal,
                Cipher::Reduction,
                Cipher::Reverse,
                Cipher::ReverseReduction,
            ],
            rounding: Rounding::Nearest,
            decimals: 0,
            kilometers: true,
            miles: true,
            meters: false,
            feet: false,
            years: true,
            months: true,
            weeks: false,
            days: true,
            include_end: false,
            leap_day: LeapDay::February28,
            include_zero: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocationComparison {
    pub a: Option<String>,
    pub b: Option<String>,
    pub include_a: bool,
    pub include_b: bool,
    pub all_locations: bool,
}
impl Default for LocationComparison {
    fn default() -> Self {
        Self {
            a: None,
            b: None,
            include_a: true,
            include_b: true,
            all_locations: true,
        }
    }
}
impl LocationComparison {
    pub fn selected_ids(&self) -> impl Iterator<Item = &str> {
        self.a
            .as_deref()
            .filter(|_| self.include_a)
            .into_iter()
            .chain(self.b.as_deref().filter(|_| self.include_b))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    #[serde(default)]
    pub gematria: crate::gematria::GematriaSearch,
    #[serde(default)]
    pub dates: DateComparison,
    #[serde(default)]
    pub locations: LocationComparison,
    #[serde(default)]
    pub filter: crate::filter::DataFilter,
    #[serde(default)]
    pub active_observation: Option<String>,
    pub id: String,
    pub name: String,
    pub dataset_id: String,
    pub context: String,
    // Empty selection means all entities in this dataset.
    pub selected_entities: Vec<String>,
    pub active_place: Option<String>,
    pub active_date: NaiveDate,
    pub active_time: NaiveTime,
    pub timezone: String,
    pub time_assumed: bool,
    pub query: String,
    pub settings: Settings,
}
impl Session {
    pub fn new(dataset_id: &str, date: NaiveDate) -> Self {
        Self {
            gematria: Default::default(),
            dates: DateComparison {
                all_dates: false,
                ..Default::default()
            },
            locations: LocationComparison {
                all_locations: false,
                ..Default::default()
            },
            filter: crate::filter::DataFilter::default(),
            active_observation: None,
            id: id(),
            name: "Research session".into(),
            dataset_id: dataset_id.into(),
            context: String::new(),
            selected_entities: vec![],
            active_place: None,
            active_date: date,
            active_time: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            timezone: "UTC".into(),
            time_assumed: true,
            query: String::new(),
            settings: Settings::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DateComparison {
    pub focus: Option<String>,
    pub comparison: Option<String>,
    pub all_dates: bool,
}
impl Default for DateComparison {
    fn default() -> Self {
        Self {
            focus: None,
            comparison: None,
            all_dates: true,
        }
    }
}
impl DateComparison {
    pub fn clear_removed(&mut self, ids: &HashSet<String>) {
        if self.focus.as_ref().is_some_and(|id| ids.contains(id)) {
            self.focus = None;
        }
        if self.comparison.as_ref().is_some_and(|id| ids.contains(id)) {
            self.comparison = None;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub schema_version: u32,
    pub datasets: Vec<Dataset>,
    pub sessions: Vec<Session>,
}
impl Workspace {
    pub fn upgrade(&mut self) -> Result<()> {
        self.validate()?;
        if self.schema_version < 5 {
            for s in &mut self.sessions {
                if s.locations.a.is_none() {
                    s.locations.a = s.active_place.clone();
                }
            }
        }
        if self.schema_version < 6 {
            for s in &mut self.sessions {
                if s.dates.focus.is_none() {
                    s.dates.focus = s.active_observation.clone().or_else(|| {
                        self.datasets
                            .iter()
                            .find(|d| d.id == s.dataset_id)
                            .and_then(|d| d.observations.first())
                            .map(|o| o.id.clone())
                    });
                }
            }
        }
        self.schema_version = 7;
        Ok(())
    }
    pub fn remove_session(&mut self, session_id: &str) -> Result<()> {
        let index = self
            .sessions
            .iter()
            .position(|s| s.id == session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;
        ensure!(
            self.sessions.len() > 1,
            "Keep at least one session; create another before removing this one"
        );
        self.sessions.remove(index);
        Ok(())
    }
    pub fn remove_entity(&mut self, dataset_id: &str, entity_id: &str) -> Result<()> {
        let d = self
            .datasets
            .iter_mut()
            .find(|d| d.id == dataset_id)
            .ok_or_else(|| anyhow::anyhow!("Dataset not found"))?;
        let index = d
            .entities
            .iter()
            .position(|e| e.id == entity_id)
            .ok_or_else(|| anyhow::anyhow!("Entity not found"))?;
        d.entities.remove(index);
        let removed = d
            .observations
            .iter()
            .filter(|o| o.entity_id.as_deref() == Some(entity_id))
            .map(|o| o.id.clone())
            .collect::<HashSet<_>>();
        d.observations
            .retain(|o| o.entity_id.as_deref() != Some(entity_id));
        for s in self
            .sessions
            .iter_mut()
            .filter(|s| s.dataset_id == dataset_id)
        {
            s.selected_entities.retain(|id| id != entity_id);
            s.dates.clear_removed(&removed);
            if s.active_observation
                .as_ref()
                .is_some_and(|id| removed.contains(id))
            {
                s.active_observation = None;
            }
        }
        Ok(())
    }
    pub fn remove_place(&mut self, dataset_id: &str, place_id: &str) -> Result<()> {
        let d = self
            .datasets
            .iter_mut()
            .find(|d| d.id == dataset_id)
            .ok_or_else(|| anyhow::anyhow!("Dataset not found"))?;
        let index = d
            .places
            .iter()
            .position(|p| p.id == place_id)
            .ok_or_else(|| anyhow::anyhow!("Place not found"))?;
        d.places.remove(index);
        for e in &mut d.entities {
            if e.place_id.as_deref() == Some(place_id) {
                e.place_id = None;
            }
        }
        for o in &mut d.observations {
            if o.place_id.as_deref() == Some(place_id) {
                o.place_id = None;
            }
        }
        for s in self
            .sessions
            .iter_mut()
            .filter(|s| s.dataset_id == dataset_id)
        {
            if s.filter.place_id.as_deref() == Some(place_id) {
                s.filter.place_id = None;
            }
            if s.active_place.as_deref() == Some(place_id) {
                s.active_place = None;
            }
            if s.locations.a.as_deref() == Some(place_id) {
                s.locations.a = None;
            }
            if s.locations.b.as_deref() == Some(place_id) {
                s.locations.b = None;
            }
        }
        Ok(())
    }
    pub fn demo() -> Self {
        let mut d = Dataset::empty("Starter laboratory");
        d.context = "Illustrative records only. Replace example dates and coordinates with sourced observations.".into();
        let mut e = Entity::new("Will");
        e.context = "Demonstrates Ordinal = 56. No biographical date is asserted.".into();
        let mut o = Observation::new(&e.id, NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        o.label = "illustrative milestone".into();
        o.source = "synthetic example".into();
        let mut p = Place::new("Illustrative point at 56°E");
        p.latitude = 37.;
        p.longitude = 56.;
        p.source = "Synthetic coordinate; not a verified park location".into();
        let mut q = Place::new("Geneva — approximate example");
        q.latitude = 46.2044;
        q.longitude = 6.1432;
        q.timezone = "Europe/Zurich".into();
        q.source = "Illustrative city-center coordinate; replace with a sourced point".into();
        d.entities.push(e);
        d.places.extend([p.clone(), q]);
        d.observations.push(o);
        let mut s = Session::new(&d.id, Utc::now().date_naive());
        s.active_place = Some(p.id.clone());
        s.locations.a = Some(p.id);
        s.locations.b = Some(d.places[1].id.clone());
        s.query = "56".into();
        s.dates.focus = Some(d.observations[0].id.clone());
        s.dates.all_dates = true;
        Self {
            schema_version: 7,
            datasets: vec![d],
            sessions: vec![s],
        }
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (1..=7).contains(&self.schema_version),
            "Unsupported workspace schema {}",
            self.schema_version
        );
        ensure!(
            !self.datasets.is_empty() && !self.sessions.is_empty(),
            "Workspace needs a dataset and session"
        );
        let mut ids = HashSet::new();
        let mut check_id = |v: &str| -> Result<()> {
            ensure!(
                Uuid::parse_str(v).is_ok() && ids.insert(v.to_owned()),
                "Invalid or duplicate record ID"
            );
            Ok(())
        };
        for d in &self.datasets {
            check_id(&d.id)?;
            ensure!(!d.name.trim().is_empty(), "Dataset needs a name");
            for e in &d.entities {
                check_id(&e.id)?;
                ensure!(!e.name.trim().is_empty(), "Entity needs a name");
                ensure!(
                    e.place_id
                        .as_ref()
                        .is_none_or(|id| d.places.iter().any(|p| p.id == *id)),
                    "Entity references a missing home/default place"
                );
            }
            for p in &d.places {
                check_id(&p.id)?;
                ensure!(!p.name.trim().is_empty(), "Place needs a name");
                ensure!(
                    p.latitude.is_finite() && (-90. ..=90.).contains(&p.latitude),
                    "Latitude must be -90 to 90"
                );
                ensure!(
                    p.longitude.is_finite() && (-180. ..=180.).contains(&p.longitude),
                    "Longitude must be -180 to 180"
                );
                ensure!(p.elevation_m.is_finite(), "Elevation must be finite");
                p.timezone.parse::<chrono_tz::Tz>()?;
            }
            for o in &d.observations {
                check_id(&o.id)?;
                ensure!(!o.label.trim().is_empty(), "Date needs a label");
                ensure!(
                    o.entity_id
                        .as_ref()
                        .is_none_or(|id| d.entities.iter().any(|e| e.id == *id)),
                    "Date references a missing entity"
                );
                ensure!(
                    o.place_id
                        .as_ref()
                        .is_none_or(|id| d.places.iter().any(|p| p.id == *id)),
                    "Date references a missing place"
                );
                crate::temporal::validate_date(o.date)?;
                o.timezone.parse::<chrono_tz::Tz>()?;
                if let Some(time) = o.time {
                    crate::temporal::utc_instant(o.date, time, &o.timezone)?;
                }
            }
        }
        for s in &self.sessions {
            check_id(&s.id)?;
            ensure!(!s.name.trim().is_empty(), "Session needs a name");
            let d = self
                .datasets
                .iter()
                .find(|d| d.id == s.dataset_id)
                .ok_or_else(|| anyhow::anyhow!("Session references a missing dataset"))?;
            ensure!(
                s.selected_entities
                    .iter()
                    .all(|id| d.entities.iter().any(|e| e.id == *id)),
                "Invalid session selection"
            );
            ensure!(
                s.active_place
                    .as_ref()
                    .is_none_or(|id| d.places.iter().any(|p| p.id == *id)),
                "Invalid active place"
            );
            ensure!(
                s.locations
                    .a
                    .iter()
                    .chain(s.locations.b.iter())
                    .all(|id| d.places.iter().any(|p| p.id == *id)),
                "Location comparison references a missing place"
            );
            ensure!(s.settings.decimals <= 6, "Precision must be 0 to 6");
            ensure!(
                s.settings.pair_limit <= 100_000,
                "Pair limit must be 0 to 100000"
            );
            s.filter.validate()?;
            ensure!(
                s.filter
                    .place_id
                    .as_ref()
                    .is_none_or(|id| d.places.iter().any(|p| p.id == *id)),
                "Location filter references a missing place"
            );
            crate::temporal::validate_date(s.active_date)?;
            ensure!(
                s.active_observation
                    .as_ref()
                    .is_none_or(|id| d.observations.iter().any(|o| o.id == *id)),
                "Active date references a missing observation"
            );
            ensure!(
                s.dates
                    .focus
                    .iter()
                    .chain(s.dates.comparison.iter())
                    .all(|id| d.observations.iter().any(|o| o.id == *id)),
                "Date comparison references a missing observation"
            );
            crate::temporal::utc_instant(s.active_date, s.active_time, &s.timezone)?;
        }
        Ok(())
    }
}
