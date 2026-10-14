use crate::{
    analysis::Feature,
    model::{Dataset, Session},
    temporal,
};
use anyhow::{Context, Result};
pub use atlas_astro::*;

pub fn for_session(dataset: &Dataset, session: &Session) -> Result<SkySnapshot> {
    let place = dataset
        .places
        .iter()
        .find(|p| Some(&p.id) == session.active_place.as_ref())
        .context("Choose an active observer location")?;
    let utc = temporal::utc_instant(session.active_date, session.active_time, &session.timezone)?;
    snapshot(
        Observer {
            latitude: place.latitude,
            longitude: place.longitude,
            elevation_m: place.elevation_m,
        },
        utc,
    )
}
pub fn features(sky: &SkySnapshot, session: &Session) -> Vec<Feature> {
    let mut out = vec![];
    let owner_id = session.active_place.clone().unwrap_or_default();
    for b in &sky.bodies {
        for (name, unit, value) in [
            ("Azimuth", "degrees", b.azimuth),
            ("Geometric altitude", "degrees", b.altitude),
            ("Right ascension", "degrees", b.ra_hours * 15.),
            ("Declination", "degrees", b.declination),
            (
                "Geocentric ecliptic longitude",
                "degrees",
                b.ecliptic_longitude,
            ),
            (
                "Geocentric ecliptic latitude",
                "degrees",
                b.ecliptic_latitude,
            ),
            ("Observer distance", "AU", b.distance_au),
        ] {
            out.push(Feature{owner_id:owner_id.clone(),owner:b.name.clone(),domain:"Sky".into(),calculation:name.into(),unit:unit.into(),raw:value,
                explanation:format!("Astronomy Engine · {} UTC · observer {}, {} at {} m; topocentric equatorial / geocentric ecliptic of date",sky.utc,sky.observer.latitude,sky.observer.longitude,sky.observer.elevation_m)});
        }
        if session.settings.atmospheric_refraction {
            out.push(Feature {
                owner_id: owner_id.clone(),
                owner: b.name.clone(),
                domain: "Sky".into(),
                calculation: "Refracted altitude".into(),
                unit: "degrees".into(),
                raw: b.apparent_altitude,
                explanation: "Standard atmospheric refraction model; geometry retained separately"
                    .into(),
            });
        }
        let km = b.distance_au * 149_597_870.7;
        for (on, unit, raw) in [
            (session.settings.kilometers, "km", km),
            (session.settings.miles, "mi", km / 1.609344),
            (session.settings.meters, "m", km * 1000.),
            (session.settings.feet, "ft", km * 1000. / 0.3048),
        ] {
            if on {
                out.push(Feature {
                    owner_id: owner_id.clone(),
                    owner: b.name.clone(),
                    domain: "Sky".into(),
                    calculation: "Observer distance".into(),
                    unit: unit.into(),
                    raw,
                    explanation: "Topocentric distance; 1 AU = 149597870.7 km".into(),
                });
            }
        }
    }
    for (i, a) in sky.bodies.iter().enumerate() {
        for b in sky.bodies.iter().skip(i + 1) {
            out.push(Feature {
                owner_id: owner_id.clone(),
                owner: format!("{} ↔ {}", a.name, b.name),
                domain: "Sky".into(),
                calculation: "Angular separation".into(),
                unit: "degrees".into(),
                raw: angular_separation(a, b),
                explanation: format!("Topocentric equatorial coordinates at {} UTC", sky.utc),
            });
        }
    }
    out.push(Feature {
        owner_id,
        owner: "Moon".into(),
        domain: "Sky".into(),
        calculation: "Phase angle".into(),
        unit: "degrees".into(),
        raw: sky.moon_phase,
        explanation: "Geocentric Moon−Sun ecliptic longitude, 0=new, 180=full".into(),
    });
    out
}
