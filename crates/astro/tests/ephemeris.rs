use atlas_astro::*;
use chrono::DateTime;
use serde::Deserialize;

#[derive(Deserialize)]
struct Reference {
    name: String,
    utc: String,
    azimuth: f64,
    altitude: f64,
}
#[test]
fn positions_agree_with_independent_jpl_horizons_fixtures() {
    let references: Vec<Reference> =
        serde_json::from_str(include_str!("jpl-reference.json")).unwrap();
    let observer = Observer {
        latitude: 51.4779,
        longitude: 0.,
        elevation_m: 46.,
    };
    for reference in references {
        let sky = snapshot(
            observer,
            DateTime::parse_from_rfc3339(&reference.utc)
                .unwrap()
                .to_utc(),
        )
        .unwrap();
        let b = sky
            .bodies
            .iter()
            .find(|b| b.name == reference.name)
            .unwrap();
        let daz = wrap_signed(b.azimuth - reference.azimuth).abs();
        let dalt = (b.altitude - reference.altitude).abs();
        assert!(
            daz < 0.05 && dalt < 0.05,
            "{} {}: angular errors {daz}°, {dalt}°",
            reference.name,
            reference.utc
        );
    }
}
#[test]
fn solar_shading_phase_and_observer_changes_are_consistent() {
    let utc = DateTime::parse_from_rfc3339("2024-03-20T12:00:00Z")
        .unwrap()
        .to_utc();
    let observer = Observer {
        latitude: 51.4779,
        longitude: 0.,
        elevation_m: 46.,
    };
    let sky = snapshot(observer, utc).unwrap();
    assert!(
        (solar_altitude(sky.subsolar_latitude, sky.subsolar_longitude, &sky) - 90.).abs() < 1e-5
    );
    assert!(
        (solar_altitude(-sky.subsolar_latitude, sky.subsolar_longitude + 180., &sky) + 90.).abs()
            < 1e-5
    );
    assert!(
        (solar_altitude(observer.latitude, observer.longitude, &sky) - sky.bodies[0].altitude)
            .abs()
            < 0.01
    );
    assert!((0. ..=1.).contains(&sky.moon_illuminated_fraction));
    let other = snapshot(
        Observer {
            longitude: 180.,
            ..observer
        },
        utc,
    )
    .unwrap();
    assert!(other.bodies[0].altitude < 0.);
    assert!((other.bodies[0].ecliptic_longitude - sky.bodies[0].ecliptic_longitude).abs() < 1e-10);
    assert!(snapshot(
        Observer {
            latitude: 91.,
            ..observer
        },
        utc
    )
    .is_err());
    let ancient = DateTime::parse_from_rfc3339("1600-01-01T00:00:00Z")
        .unwrap()
        .to_utc();
    assert!(snapshot(observer, ancient).is_err());
}
