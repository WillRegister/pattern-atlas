use crate::{
    geography,
    model::{Dataset, Rounding, Session, Settings},
    temporal,
};
use chrono::Datelike;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
pub struct Feature {
    pub owner_id: String,
    pub owner: String,
    pub domain: String,
    pub calculation: String,
    pub unit: String,
    pub raw: f64,
    pub explanation: String,
}
impl Feature {
    pub fn value(&self, settings: &Settings) -> f64 {
        transform(self.raw, settings)
    }
    pub fn displayed(&self, settings: &Settings) -> String {
        if settings.rounding == Rounding::Raw {
            self.raw.to_string()
        } else {
            format!("{:.*}", settings.decimals as usize, self.value(settings))
        }
    }
    pub fn matches(&self, query: &str, settings: &Settings) -> bool {
        let Ok(q) = query.trim().parse::<f64>() else {
            return false;
        };
        self.matches_value(q, settings)
    }
    pub fn matches_value(&self, q: f64, settings: &Settings) -> bool {
        let v = self.value(settings);
        q.is_finite() && (v - q).abs() <= f64::EPSILON * v.abs().max(q.abs()).max(1.) * 4.
    }
}
pub fn transform(value: f64, s: &Settings) -> f64 {
    let scale = 10_f64.powi(s.decimals as i32);
    match s.rounding {
        Rounding::Raw => value,
        Rounding::Nearest => (value * scale).round() / scale,
        Rounding::Floor => (value * scale).floor() / scale,
    }
}
pub fn features(d: &Dataset, s: &Session) -> Vec<Feature> {
    let scope = crate::filter::scope(d, s);
    features_in_scope(d, s, &scope)
}
pub fn features_in_scope(d: &Dataset, s: &Session, scope: &crate::filter::Scope) -> Vec<Feature> {
    let mut out = vec![];
    let mut push = |id: &str,
                    owner: &str,
                    domain: &str,
                    calculation: &str,
                    unit: &str,
                    raw: f64,
                    explanation: String| {
        out.push(Feature {
            owner_id: id.into(),
            owner: owner.into(),
            domain: domain.into(),
            calculation: calculation.into(),
            unit: unit.into(),
            raw,
            explanation,
        });
    };
    let included = |id: &str| scope.entities.contains(id);
    for e in d.entities.iter().filter(|e| included(&e.id)) {
        for text in std::iter::once(&e.name).chain(e.aliases.iter()) {
            if let Some(number) = crate::gematria::literal_number(text) {
                push(&e.id,text,"Gematria","Literal number","value",number,
                    "Saved numeric term; used as a literal value rather than scoring its digits as letters".into());
                continue;
            }
            for cipher in &s.settings.ciphers {
                push(
                    &e.id,
                    text,
                    "Gematria",
                    cipher.label(),
                    "value",
                    cipher.total(text) as f64,
                    cipher.breakdown(text),
                );
            }
        }
    }
    let obs = date_observations(d, s, scope);
    for o in &obs {
        let entity = d
            .entities
            .iter()
            .find(|e| Some(&e.id) == o.entity_id.as_ref())
            .map_or("Unlinked date", |e| e.name.as_str());
        let owner = format!("{entity} · {}", o.label);
        let signed = (s.active_date - o.date).num_days();
        let desc = format!(
            "{} → {}; civil calendar dates, end inclusion {}",
            o.date, s.active_date, s.settings.include_end
        );
        push(
            &o.id,
            &owner,
            "Dates",
            "Signed days from event",
            "days",
            signed as f64,
            "Active date minus original date; excludes end-date adjustment".into(),
        );
        let span = temporal::span(o.date, s.active_date, &s.settings);
        push(
            &o.id,
            &owner,
            "Dates",
            "Span total days",
            "days",
            span.total_days as f64,
            desc.clone(),
        );
        push(
            &o.id,
            &owner,
            "Dates",
            "Signed span total days",
            "days",
            (span.sign * span.total_days) as f64,
            desc.clone(),
        );
        for (on, name, unit, value) in [
            (s.settings.years, "Calendar years", "years", span.years),
            (
                s.settings.months,
                "Remaining calendar months",
                "months",
                span.months,
            ),
            (s.settings.weeks, "Remaining weeks", "weeks", span.weeks),
            (s.settings.days, "Remaining days", "days", span.days),
        ] {
            if on {
                push(
                    &o.id,
                    &owner,
                    "Dates",
                    name,
                    unit,
                    value as f64,
                    desc.clone(),
                );
            }
        }
        let next = temporal::next_anniversary(o.date, s.active_date, s.settings.leap_day);
        push(
            &o.id,
            &owner,
            "Dates",
            "Days to next anniversary",
            "days",
            (next - s.active_date).num_days() as f64,
            format!(
                "Next occurrence {next}; original {}; {:?} leap policy",
                o.date, s.settings.leap_day
            ),
        );
        let relation = temporal::relation(o.date, s.active_date, &s.settings);
        if let (Some(previous), Some(since)) = (relation.previous, relation.since.as_ref()) {
            push(&o.id, &owner, "Dates", "Days since previous anniversary", "days", since.total_days as f64,
                format!("{previous} → {}; most recent occurrence on or before the active date; no occurrence before the original event; civil-day offset", s.active_date));
        }
        for (prefix, span) in [
            ("Since previous anniversary", relation.since.as_ref()),
            ("Until next anniversary", Some(&relation.until)),
        ] {
            if let Some(span) = span {
                for (on, unit, value) in [
                    (s.settings.years, "years", span.years),
                    (s.settings.months, "months", span.months),
                    (s.settings.weeks, "weeks", span.weeks),
                    (s.settings.days, "days", span.days),
                ] {
                    if on {
                        push(&o.id, &owner, "Dates", &format!("{prefix} · remaining {unit}"), unit, value as f64,
                        "Calendar decomposition under selected unit toggles; anniversary countdowns exclude end adjustment".into());
                    }
                }
            }
        }
    }
    for (a, b) in obs
        .iter()
        .enumerate()
        .flat_map(|(i, a)| obs.iter().skip(i + 1).map(move |b| (a, b)))
        .take(s.settings.pair_limit as usize)
    {
        let owner = format!("{} ({}) ↔ {} ({})", a.label, a.date, b.label, b.date);
        push(
            &a.id,
            &owner,
            "Dates",
            "Pairwise date span",
            "days",
            temporal::span(a.date, b.date, &s.settings).total_days as f64,
            "Absolute civil-date interval under end inclusion policy".into(),
        );
        let pair = temporal::anniversary_pair(a.date, b.date, s.active_date, &s.settings);
        let original = temporal::span(a.date, b.date, &s.settings);
        for (label, span) in [
            ("Original date pair", Some(&original)),
            ("Previous-anniversary pair span", pair.previous.as_ref()),
            ("Next-anniversary pair span", Some(&pair.next)),
        ] {
            if let Some(span) = span {
                if label != "Original date pair" {
                    push(&a.id, &owner, "Dates", label, "days", span.total_days as f64,
                    format!("Anniversaries for both events bracketing {}; original events never recur before their recorded dates; excludes end adjustment", s.active_date));
                }
                for (on, unit, value) in [
                    (s.settings.years, "years", span.years),
                    (s.settings.months, "months", span.months),
                    (s.settings.weeks, "weeks", span.weeks),
                    (s.settings.days, "days", span.days),
                ] {
                    if on {
                        push(&a.id,&owner,"Dates",&format!("{label} · remaining {unit}"),unit,value as f64,
                        "Calendar components of the selected pair interval; original spans follow end inclusion, anniversary gaps use civil-day offsets".into());
                    }
                }
            }
        }
    }
    let places = location_places(d, s, scope);
    // Recorded data is distinct from calculated date/geographic/gematria properties.
    for (id, name, attributes, source) in d
        .entities
        .iter()
        .filter(|e| included(&e.id))
        .map(|e| (&e.id, &e.name, &e.attributes, &e.source))
        .chain(
            places
                .iter()
                .map(|p| (&p.id, &p.name, &p.attributes, &p.source)),
        )
        .chain(
            obs.iter()
                .map(|o| (&o.id, &o.label, &o.attributes, &o.source)),
        )
    {
        for (key, value) in attributes {
            if let Some(raw) = value.as_f64().filter(|v| v.is_finite()) {
                push(
                    id,
                    name,
                    "Data",
                    key,
                    "recorded",
                    raw,
                    format!("Recorded attribute {key}. Source: {source}"),
                );
            }
        }
    }
    out.extend(active_date_features(s));
    out.extend(geographic_features(&places, s));
    #[cfg(feature = "astronomy")]
    if let Ok(sky) = crate::sky::for_session(d, s) {
        out.extend(crate::sky::features(&sky, s));
    }
    out
}
/// Viewing-date calendar facts are independent of saved-record and pair-selection filters.
pub fn active_date_features(s: &Session) -> Vec<Feature> {
    let date = s.active_date;
    let year_days = chrono::NaiveDate::from_ymd_opt(date.year(), 12, 31)
        .expect("Valid year end")
        .ordinal();
    let iso = date.iso_week();
    let owner = format!("Active date · {date}");
    [
        ("Day of year","day number",date.ordinal() as f64,"January 1 is day 1; leap years include February 29"),
        ("Days left in year","days",(year_days-date.ordinal()) as f64,"Civil days after the active date through December 31; excludes the active day and ignores interval end-inclusion settings"),
        ("Days in year","days",year_days as f64,"Calendar year length: 365 or 366 days under Gregorian leap-year rules"),
        ("ISO week number","week number",iso.week() as f64,"ISO week numbering: Monday start; week 1 contains January 4; see ISO week year near year boundaries"),
        ("Calendar year","year",date.year() as f64,"Gregorian calendar year of the active date"),
        ("Month number","month number",date.month() as f64,"January is 1; December is 12"),
        ("Day of month","day number",date.day() as f64,"One-based calendar day within the active month"),
        ("Weekday number","weekday number",date.weekday().number_from_monday() as f64,"ISO weekday: Monday 1 through Sunday 7"),
        ("ISO week year","year",iso.year() as f64,"The year belonging to the ISO week; may differ from the calendar year around New Year"),
    ].into_iter().map(|(calculation,unit,raw,explanation)|Feature {
        owner_id:s.id.clone(),owner:owner.clone(),domain:"Dates".into(),calculation:calculation.into(),unit:unit.into(),raw,explanation:explanation.into(),
    }).collect()
}
pub fn date_observations<'a>(
    d: &'a Dataset,
    s: &Session,
    scope: &crate::filter::Scope,
) -> Vec<&'a crate::model::Observation> {
    if s.dates.all_dates {
        d.observations
            .iter()
            .filter(|o| scope.observations.contains(&o.id))
            .collect()
    } else {
        let mut dates = vec![];
        for id in s.dates.focus.iter().chain(s.dates.comparison.iter()) {
            if let Some(o) = d
                .observations
                .iter()
                .find(|o| o.id == *id && scope.observations.contains(id))
            {
                if !dates
                    .iter()
                    .any(|other: &&crate::model::Observation| other.id == o.id)
                {
                    dates.push(o);
                }
            }
        }
        dates
    }
}
/// Resolve location analysis independently from entity/date selection, retaining saved filters.
pub fn location_places<'a>(
    d: &'a Dataset,
    s: &Session,
    scope: &crate::filter::Scope,
) -> Vec<&'a crate::model::Place> {
    if s.locations.all_locations {
        d.places
            .iter()
            .filter(|p| scope.places.contains(&p.id))
            .collect()
    } else {
        let mut places = vec![];
        for id in s.locations.selected_ids() {
            if let Some(p) = d
                .places
                .iter()
                .find(|p| p.id == id && scope.places.contains(id))
            {
                if !places.iter().any(|q: &&crate::model::Place| q.id == p.id) {
                    places.push(p);
                }
            }
        }
        places
    }
}
pub fn geographic_features(places: &[&crate::model::Place], s: &Session) -> Vec<Feature> {
    let mut out = vec![];
    let mut push = |id: &str,
                    owner: &str,
                    domain: &str,
                    calculation: &str,
                    unit: &str,
                    raw: f64,
                    explanation: String| {
        out.push(Feature {
            owner_id: id.into(),
            owner: owner.into(),
            domain: domain.into(),
            calculation: calculation.into(),
            unit: unit.into(),
            raw,
            explanation,
        });
    };
    for p in places {
        push(
            &p.id,
            &p.name,
            "Geography",
            "Latitude",
            "degrees",
            p.latitude,
            "Signed geographic latitude; north positive".into(),
        );
        push(
            &p.id,
            &p.name,
            "Geography",
            "Longitude",
            "degrees",
            p.longitude,
            "Signed geographic longitude; east positive".into(),
        );
    }
    for (a, b) in places
        .iter()
        .enumerate()
        .flat_map(|(i, a)| places.iter().skip(i + 1).map(move |b| (a, b)))
        .take(s.settings.pair_limit as usize)
    {
        let g = geography::between(a, b);
        let owner = format!("{} ↔ {}", a.name, b.name);
        for (on, unit, value) in [
            (s.settings.kilometers, "km", g.kilometers),
            (s.settings.miles, "mi", g.kilometers / 1.609344),
            (s.settings.meters, "m", g.kilometers * 1000.),
            (s.settings.feet, "ft", g.kilometers * 1000. / 0.3048),
        ] {
            if on {
                push(
                    &a.id,
                    &owner,
                    "Geography",
                    "Great-circle distance",
                    unit,
                    value,
                    "Spherical haversine; mean Earth radius 6371.0088 km; not road distance".into(),
                );
            }
        }
        for (label, value) in [
            ("Central angle", g.central_angle),
            ("Signed latitude delta (B−A)", g.delta_lat),
            ("Absolute latitude delta", g.delta_lat.abs()),
            ("Signed longitude delta (B−A)", g.delta_lon),
            ("Absolute raw longitude delta", g.delta_lon.abs()),
            ("Shortest longitude separation", g.shortest_lon),
        ] {
            push(
                &a.id,
                &owner,
                "Geography",
                label,
                "degrees",
                value,
                "B minus A; selected mode follows A/B order, all-locations mode follows dataset order; shortest longitude wraps across the dateline".into(),
            );
        }
    }
    out
}

pub fn groups<'a>(features: &'a [Feature], s: &Settings) -> BTreeMap<String, Vec<&'a Feature>> {
    let mut groups: BTreeMap<String, Vec<&Feature>> = BTreeMap::new();
    for f in features {
        if !s.include_zero && f.value(s) == 0. {
            continue;
        }
        groups.entry(f.displayed(s)).or_default().push(f);
    }
    groups.retain(|_, v| v.len() > 1);
    groups
}
