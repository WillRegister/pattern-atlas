//! Portable additions, distinct from full workspace backups. Validate before mutation.
use crate::{model::*, temporal};
use anyhow::{bail, ensure, Context, Result};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

pub type Attributes = BTreeMap<String, serde_json::Value>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportFile {
    pub format: String,
    pub version: u32,
    pub batch_id: String,
    pub dataset: ImportDataset,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportDataset {
    pub name: String,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub entities: Vec<ImportEntity>,
    #[serde(default)]
    pub places: Vec<ImportPlace>,
    #[serde(default)]
    pub observations: Vec<ImportObservation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportEntity {
    #[serde(default)]
    pub place_key: Option<String>,
    pub key: String,
    pub name: String,
    #[serde(default = "word_kind")]
    pub kind: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub attributes: Attributes,
}
fn word_kind() -> String {
    "word".into()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportPlace {
    pub key: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub elevation_m: f64,
    pub timezone: String,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub attributes: Attributes,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportObservation {
    pub key: String,
    pub label: String,
    pub date: String,
    #[serde(default)]
    pub time: Option<String>,
    pub timezone: String,
    #[serde(default)]
    pub entity_key: Option<String>,
    #[serde(default)]
    pub place_key: Option<String>,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub attributes: Attributes,
}

impl ImportFile {
    pub fn parse(text: &str) -> Result<Self> {
        let file: Self = serde_json::from_str(text.trim_start_matches('\u{feff}'))
            .context("Invalid import JSON (check field names and value types)")?;
        file.dataset()?;
        Ok(file)
    }
    pub fn dataset(&self) -> Result<Dataset> {
        ensure!(
            self.format == "pattern-atlas-data",
            "format must be pattern-atlas-data"
        );
        ensure!(
            self.version == 1,
            "Unsupported import version {}",
            self.version
        );
        ensure!(!self.batch_id.trim().is_empty(), "batch_id is required");
        ensure!(
            !self.dataset.name.trim().is_empty(),
            "Dataset name is required"
        );
        ensure!(
            !self.dataset.entities.is_empty()
                || !self.dataset.places.is_empty()
                || !self.dataset.observations.is_empty(),
            "Import contains no records"
        );
        let mut d = Dataset::empty(&self.dataset.name);
        d.context = self.dataset.context.clone();
        d.import_batches.push(self.batch_id.clone());
        let mut entities = BTreeMap::new();
        let mut places = BTreeMap::new();
        let mut observations = HashSet::new();
        for item in &self.dataset.entities {
            ensure!(
                !item.key.trim().is_empty() && !entities.contains_key(&item.key),
                "Empty or duplicate entity key: {}",
                item.key
            );
            let mut e = Entity::new(&item.name);
            e.kind = item.kind.clone();
            e.aliases = item.aliases.clone();
            e.tags = item.tags.clone();
            e.context = item.context.clone();
            e.source = item.source.clone();
            e.attributes = item.attributes.clone();
            entities.insert(item.key.clone(), e.id.clone());
            d.entities.push(e);
        }
        for item in &self.dataset.places {
            ensure!(
                !item.key.trim().is_empty() && !places.contains_key(&item.key),
                "Empty or duplicate place key: {}",
                item.key
            );
            let mut p = Place::new(&item.name);
            p.latitude = item.latitude;
            p.longitude = item.longitude;
            p.elevation_m = item.elevation_m;
            p.timezone = item.timezone.clone();
            p.context = item.context.clone();
            p.source = item.source.clone();
            p.attributes = item.attributes.clone();
            places.insert(item.key.clone(), p.id.clone());
            d.places.push(p);
        }
        for (e, item) in d.entities.iter_mut().zip(&self.dataset.entities) {
            e.place_id = item
                .place_key
                .as_ref()
                .map(|k| {
                    places.get(k).cloned().with_context(|| {
                        format!("Unknown home/default place_key {k} in {}", item.key)
                    })
                })
                .transpose()?;
        }
        for item in &self.dataset.observations {
            ensure!(
                !item.key.trim().is_empty() && observations.insert(&item.key),
                "Empty or duplicate observation key: {}",
                item.key
            );
            let mut o = Observation::unlinked(
                temporal::parse_date(&item.date).with_context(|| format!("Date {}", item.key))?,
            );
            o.label = item.label.clone();
            o.time = item
                .time
                .as_deref()
                .map(|t| {
                    NaiveTime::parse_from_str(t, "%H:%M:%S%.f")
                        .with_context(|| format!("Time {} must be HH:MM:SS or null", item.key))
                })
                .transpose()?;
            o.timezone = item.timezone.clone();
            o.entity_id = item
                .entity_key
                .as_ref()
                .map(|k| {
                    entities
                        .get(k)
                        .cloned()
                        .with_context(|| format!("Unknown entity_key {k} in {}", item.key))
                })
                .transpose()?;
            o.place_id = item
                .place_key
                .as_ref()
                .map(|k| {
                    places
                        .get(k)
                        .cloned()
                        .with_context(|| format!("Unknown place_key {k} in {}", item.key))
                })
                .transpose()?;
            o.context = item.context.clone();
            o.source = item.source.clone();
            o.attributes = item.attributes.clone();
            d.observations.push(o);
        }
        let s = Session::new(&d.id, chrono::Utc::now().date_naive());
        Workspace {
            schema_version: 7,
            datasets: vec![d.clone()],
            sessions: vec![s],
        }
        .validate()?;
        Ok(d)
    }
    /// None creates a dataset and session. Some appends, with references local to this file.
    /// A rejected import leaves the workspace byte-for-byte unchanged.
    pub fn apply(&self, workspace: &mut Workspace, target: Option<&str>) -> Result<String> {
        if workspace
            .datasets
            .iter()
            .any(|d| d.import_batches.contains(&self.batch_id))
        {
            bail!(
                "Batch {} was already imported; use Undo to remove it before retrying",
                self.batch_id
            );
        }
        let imported = self.dataset()?;
        let mut candidate = workspace.clone();
        let result;
        if let Some(id) = target {
            let d = candidate
                .datasets
                .iter_mut()
                .find(|d| d.id == id)
                .context("Target dataset no longer exists")?;
            result = d.id.clone();
            d.entities.extend(imported.entities);
            d.places.extend(imported.places);
            d.observations.extend(imported.observations);
            d.import_batches.extend(imported.import_batches);
            if !imported.context.is_empty() {
                d.context.push_str(&format!(
                    "\n\nImported {}: {}",
                    self.batch_id, imported.context
                ));
            }
        } else {
            result = imported.id.clone();
            let mut s = Session::new(&result, chrono::Utc::now().date_naive());
            s.name = format!("{} · imported", imported.name);
            s.dates.focus = imported.observations.first().map(|o| o.id.clone());
            candidate.sessions.push(s);
            candidate.datasets.push(imported);
        }
        candidate.upgrade()?;
        *workspace = candidate;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> ImportFile {
        ImportFile::parse(include_str!("../../../examples/import-demo.json")).unwrap()
    }
    #[test]
    fn linked_import_is_atomic_and_repeat_batches_are_rejected() {
        let mut w = Workspace::demo();
        let original = serde_json::to_string(&w).unwrap();
        let mut file = sample();
        file.dataset.observations[0].entity_key = Some("missing".into());
        assert!(file.apply(&mut w, None).is_err());
        assert_eq!(serde_json::to_string(&w).unwrap(), original);
        let file = sample();
        let id = file.apply(&mut w, None).unwrap();
        let d = w.datasets.iter().find(|d| d.id == id).unwrap();
        assert_eq!(
            d.observations[0].entity_id.as_ref(),
            Some(&d.entities[0].id)
        );
        assert_eq!(d.observations[0].place_id.as_ref(), Some(&d.places[0].id));
        assert!(d.observations[0].time.is_none());
        assert_eq!(d.entities[0].attributes["score_points"], 56);
        let after = serde_json::to_string(&w).unwrap();
        assert!(file.apply(&mut w, None).is_err());
        assert_eq!(serde_json::to_string(&w).unwrap(), after);
    }
    #[test]
    fn invalid_coordinates_dst_unknown_fields_and_keys_are_rejected() {
        let mut file = sample();
        file.dataset.places[0].latitude = 91.;
        assert!(file.dataset().is_err());
        let mut file = sample();
        file.dataset.observations[0].date = "2026-03-08".into();
        file.dataset.observations[0].time = Some("02:30:00".into());
        assert!(file.dataset().is_err());
        let mut file = sample();
        file.dataset.entities.push(file.dataset.entities[0].clone());
        assert!(file.dataset().is_err());
        let mut json = serde_json::to_value(sample()).unwrap();
        json["dataset"]["entities"][0]["typo"] = true.into();
        assert!(ImportFile::parse(&json.to_string()).is_err());
    }
    #[test]
    fn append_preserves_existing_records_and_accepts_utf8_bom() {
        let file = sample();
        let mut w = Workspace::demo();
        let target = w.datasets[0].id.clone();
        file.apply(&mut w, Some(&target)).unwrap();
        assert_eq!(w.datasets.len(), 1);
        assert_eq!(w.datasets[0].entities[0].name, "Will");
        w.validate().unwrap();
        assert!(ImportFile::parse(&format!(
            "\u{feff}{}",
            serde_json::to_string(&file).unwrap()
        ))
        .is_ok());
    }
}
