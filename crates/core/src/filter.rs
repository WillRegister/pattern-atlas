//! One scope for UI browsing and all numerical domains; canonical records stay intact.
use crate::{model::*, temporal};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DataFilter {
    pub text: String,
    pub kind: String,
    pub tag: String,
    pub conference: String,
    pub division: String,
    pub place_id: Option<String>,
    pub from: String,
    pub until: String,
    pub sourced_only: bool,
    pub link: DateLink,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateLink {
    #[default]
    All,
    Linked,
    Unlinked,
}
impl DataFilter {
    pub fn validate(&self) -> Result<()> {
        let from = (!self.from.trim().is_empty())
            .then(|| temporal::parse_date(self.from.trim()))
            .transpose()?;
        let until = (!self.until.trim().is_empty())
            .then(|| temporal::parse_date(self.until.trim()))
            .transpose()?;
        ensure!(
            from.zip(until).is_none_or(|(a, b)| a <= b),
            "Filter start date must be on or before end date"
        );
        terms(&self.text)?;
        Ok(())
    }
    pub fn active(&self) -> bool {
        *self != Self::default()
    }
    fn facets(&self) -> bool {
        !self.kind.is_empty()
            || !self.tag.is_empty()
            || !self.conference.is_empty()
            || !self.division.is_empty()
    }
    fn dates(&self) -> bool {
        !self.from.trim().is_empty() || !self.until.trim().is_empty() || self.link != DateLink::All
    }
}
#[derive(Default, Clone, Debug)]
pub struct Scope {
    pub entities: HashSet<String>,
    pub observations: HashSet<String>,
    pub places: HashSet<String>,
    pub error: Option<String>,
}
#[derive(Debug)]
struct Term {
    text: String,
    excluded: bool,
}
fn terms(query: &str) -> Result<Vec<Term>> {
    let mut out = vec![];
    let mut current = String::new();
    let mut quote = false;
    for c in query.chars().chain(std::iter::once(' ')) {
        if c == '"' {
            quote = !quote;
        } else if c.is_whitespace() && !quote {
            if !current.is_empty() {
                let excluded = current.starts_with('-') && current.len() > 1;
                out.push(Term {
                    text: current
                        .trim_start_matches(if excluded { '-' } else { '\0' })
                        .to_lowercase(),
                    excluded,
                });
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    ensure!(!quote, "Close the quoted phrase in Search data");
    Ok(out)
}
pub fn text_matches(query: &str, haystack: &str) -> bool {
    let Ok(terms) = terms(query) else {
        return false;
    };
    matches_terms(&terms, haystack)
}
fn matches_terms(terms: &[Term], haystack: &str) -> bool {
    let haystack = haystack.to_lowercase();
    terms
        .iter()
        .all(|term| haystack.contains(&term.text) != term.excluded)
}
pub fn entity_text(e: &Entity) -> String {
    format!(
        "{} {} {} {} {} {} {}",
        e.name,
        e.kind,
        e.aliases.join(" "),
        e.tags.join(" "),
        e.context,
        e.source,
        serde_json::to_string(&e.attributes).unwrap_or_default()
    )
}
pub fn place_text(p: &Place) -> String {
    format!(
        "{} {} {} {} {} {} {}",
        p.name,
        p.latitude,
        p.longitude,
        p.timezone,
        p.context,
        p.source,
        serde_json::to_string(&p.attributes).unwrap_or_default()
    )
}
fn linked_place_text(p: &Place) -> String {
    // A shared place may cite another team's website. Its provenance must not
    // make an exclusion like -Chargers hide the Rams at the same home place.
    format!(
        "{} {} {} {} {}",
        p.name,
        p.latitude,
        p.longitude,
        p.timezone,
        serde_json::to_string(&p.attributes).unwrap_or_default()
    )
}
pub fn date_text(o: &Observation) -> String {
    format!(
        "{} {} {:?} {} {} {} {}",
        o.label,
        o.date,
        o.time,
        o.timezone,
        o.context,
        o.source,
        serde_json::to_string(&o.attributes).unwrap_or_default()
    )
}
fn attribute_is(e: &Entity, key: &str, value: &str) -> bool {
    value.is_empty()
        || e.attributes
            .get(key)
            .and_then(|v| v.as_str())
            .is_some_and(|v| v.eq_ignore_ascii_case(value))
}
pub fn scope(d: &Dataset, s: &Session) -> Scope {
    let f = &s.filter;
    if let Err(e) = f.validate() {
        return Scope {
            error: Some(e.to_string()),
            ..Default::default()
        };
    }
    let terms = terms(&f.text).unwrap();
    let from = temporal::parse_date(f.from.trim()).ok();
    let until = temporal::parse_date(f.until.trim()).ok();
    let places = d
        .places
        .iter()
        .map(|p| (p.id.as_str(), linked_place_text(p)))
        .collect::<std::collections::HashMap<_, _>>();
    let entities = d
        .entities
        .iter()
        .map(|e| (e.id.as_str(), e))
        .collect::<std::collections::HashMap<_, _>>();
    let candidate = |e: &Entity| {
        (s.selected_entities.is_empty() || s.selected_entities.contains(&e.id))
            && (f.kind.is_empty() || f.kind.eq_ignore_ascii_case(&e.kind))
            && (f.tag.is_empty() || e.tags.iter().any(|t| t.eq_ignore_ascii_case(&f.tag)))
            && attribute_is(e, "conference", &f.conference)
            && attribute_is(e, "division", &f.division)
    };
    let text = d
        .entities
        .iter()
        .map(|e| {
            let home = e
                .place_id
                .as_deref()
                .and_then(|id| places.get(id))
                .map_or("", String::as_str);
            (e.id.as_str(), format!("{} {home}", entity_text(e)))
        })
        .collect::<std::collections::HashMap<_, _>>();
    let mut result = Scope::default();
    for o in &d.observations {
        let parent = o
            .entity_id
            .as_deref()
            .and_then(|id| entities.get(id))
            .copied();
        let parent_ok = parent.map_or(!f.facets() && s.selected_entities.is_empty(), candidate);
        let home = parent.and_then(|e| e.place_id.as_ref());
        if !parent_ok
            || (f.sourced_only && o.source.trim().is_empty())
            || from.is_some_and(|v| o.date < v)
            || until.is_some_and(|v| o.date > v)
            || (f.link == DateLink::Linked && parent.is_none())
            || (f.link == DateLink::Unlinked && parent.is_some())
            || f.place_id
                .as_ref()
                .is_some_and(|id| o.place_id.as_ref() != Some(id) && home != Some(id))
        {
            continue;
        }
        let related = o
            .place_id
            .as_deref()
            .and_then(|id| places.get(id))
            .map_or("", String::as_str);
        let parent_text = parent
            .and_then(|e| text.get(e.id.as_str()))
            .map_or("", String::as_str);
        if matches_terms(&terms, &format!("{} {related} {parent_text}", date_text(o))) {
            result.observations.insert(o.id.clone());
        }
    }
    let parents = d
        .observations
        .iter()
        .filter(|o| result.observations.contains(&o.id))
        .filter_map(|o| o.entity_id.as_ref())
        .collect::<HashSet<_>>();
    for e in &d.entities {
        if !candidate(e) || (f.sourced_only && e.source.trim().is_empty()) {
            continue;
        }
        if f.dates() && !parents.contains(&e.id) {
            continue;
        }
        if f.place_id
            .as_ref()
            .is_some_and(|id| e.place_id.as_ref() != Some(id))
            && !parents.contains(&e.id)
        {
            continue;
        }
        if matches_terms(&terms, &text[e.id.as_str()]) || parents.contains(&e.id) {
            result.entities.insert(e.id.clone());
        }
    }
    let related = d
        .entities
        .iter()
        .filter(|e| result.entities.contains(&e.id))
        .filter_map(|e| e.place_id.as_ref())
        .chain(
            d.observations
                .iter()
                .filter(|o| result.observations.contains(&o.id))
                .filter_map(|o| o.place_id.as_ref()),
        )
        .collect::<HashSet<_>>();
    let restricted = f.facets() || f.dates() || !s.selected_entities.is_empty();
    for p in &d.places {
        if (f.sourced_only && p.source.trim().is_empty())
            || f.place_id.as_ref().is_some_and(|id| p.id != *id)
        {
            continue;
        }
        if (restricted && related.contains(&p.id))
            || (!restricted && (matches_terms(&terms, &place_text(p)) || related.contains(&p.id)))
        {
            result.places.insert(p.id.clone());
        }
    }
    result
}
pub fn pairs(count: usize) -> usize {
    count.saturating_mul(count.saturating_sub(1)) / 2
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_handles_phrases_negative_terms_unicode_and_invalid_quotes() {
        assert!(text_matches("\"new york\" -jets", "New York Giants"));
        assert!(!text_matches("\"new york\" -jets", "New York Jets"));
        assert!(text_matches("CAFÉ", "café team"));
        assert!(!text_matches("\"broken", "broken"));
    }
    #[test]
    fn scope_links_home_place_and_dates_without_mutating_canonical_data() {
        let mut w = Workspace::demo();
        let d = &mut w.datasets[0];
        let e = &mut d.entities[0];
        e.place_id = Some(d.places[1].id.clone());
        e.attributes.insert("conference".into(), "NFC".into());
        e.tags.push("NFL".into());
        let s = &mut w.sessions[0];
        s.filter.conference = "NFC".into();
        s.filter.text = "Geneva".into();
        let before = serde_json::to_string(d).unwrap();
        let scope = scope(d, s);
        assert_eq!(scope.entities.len(), 1);
        assert_eq!(scope.observations.len(), 1);
        assert_eq!(scope.places.len(), 1);
        assert!(scope.places.contains(&d.places[1].id));
        assert_eq!(serde_json::to_string(d).unwrap(), before);
        s.filter.from = "2001-01-01".into();
        let scope = super::scope(d, s);
        assert!(scope.entities.is_empty());
        assert!(scope.observations.is_empty());
        assert!(scope.places.is_empty());
        s.filter.until = "1999-01-01".into();
        assert!(super::scope(d, s).error.is_some());
    }
    #[test]
    fn range_boundaries_unlinked_records_and_pair_limits_are_explicit() {
        let mut w = Workspace::demo();
        let d = &mut w.datasets[0];
        d.observations.push(Observation::unlinked(
            temporal::parse_date("2000-01-01").unwrap(),
        ));
        let s = &mut w.sessions[0];
        s.filter.from = "2000-01-01".into();
        s.filter.until = "2000-01-01".into();
        s.filter.link = DateLink::Unlinked;
        let scope = scope(d, s);
        assert_eq!(scope.observations.len(), 1);
        assert!(scope.entities.is_empty());
        s.filter = DataFilter::default();
        s.settings.pair_limit = 0;
        let features = crate::analysis::features(d, s);
        assert!(!features
            .iter()
            .any(|f| f.calculation == "Pairwise date span"
                || f.calculation == "Great-circle distance"));
        s.settings.pair_limit = 1;
        let features = crate::analysis::features(d, s);
        assert_eq!(
            features
                .iter()
                .filter(|f| f.calculation == "Pairwise date span")
                .count(),
            1
        );
        assert_eq!(pairs(100000), 4_999_950_000);
    }
    #[test]
    fn excluding_a_team_does_not_hide_another_team_at_a_shared_home_place() {
        let file = crate::bulk::ImportFile::parse(include_str!(
            "../../../datasets/nfl-teams-2026-10-13.json"
        ))
        .unwrap();
        let d = file.dataset().unwrap();
        let mut s = Session::new(&d.id, chrono::Utc::now().date_naive());
        s.filter.text = "\"Los Angeles\" -Chargers".into();
        let scope = scope(&d, &s);
        let names = d
            .entities
            .iter()
            .filter(|e| scope.entities.contains(&e.id))
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["Los Angeles Rams"]);
        assert_eq!(scope.places.len(), 1);
        assert_eq!(scope.observations.len(), 1);
    }
}
