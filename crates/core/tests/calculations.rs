use atlas_core::{
    analysis::{features, transform},
    cipher::Cipher,
    geography,
    model::*,
    temporal,
};
use chrono::NaiveTime;
fn date(s: &str) -> chrono::NaiveDate {
    temporal::parse_date(s).unwrap()
}

#[test]
fn cipher_identity_and_examples() {
    assert_eq!(Cipher::Ordinal.total("Will"), 56);
    assert_eq!(Cipher::Reduction.total("Will"), 20);
    assert_eq!(Cipher::Reverse.total("Will"), 52);
    for c in ['A', 'I', 'J', 'R', 'S', 'Z'] {
        assert_eq!(Cipher::Ordinal.weight(c) + Cipher::Reverse.weight(c), 27);
    }
    assert_eq!(Cipher::Standard.total("AJS"), 111);
    assert_eq!(Cipher::Primes.total("AZ"), 103);
    assert_eq!(Cipher::Ordinal.total("é 56!"), 0);
}
#[test]
fn calendar_months_are_not_thirty_days() {
    let mut s = Settings {
        years: false,
        ..Settings::default()
    };
    let v = temporal::span(date("2024-01-31"), date("2024-03-01"), &s);
    assert_eq!((v.months, v.days, v.total_days), (1, 1, 30));
    s.months = false;
    s.weeks = true;
    let v = temporal::span(date("2024-01-31"), date("2024-03-01"), &s);
    assert_eq!((v.weeks, v.days), (4, 2));
    s.include_end = true;
    assert_eq!(
        temporal::span(date("2024-01-31"), date("2024-01-31"), &s).total_days,
        1
    );
    assert_eq!(
        temporal::span(date("2024-03-01"), date("2024-01-31"), &s).sign,
        -1
    );
}
#[test]
fn anniversaries_and_leap_policy() {
    let birth = date("2000-02-29");
    let reference = date("2025-02-28");
    assert_eq!(
        temporal::next_anniversary(birth, reference, LeapDay::February28),
        reference
    );
    assert_eq!(
        temporal::next_anniversary(birth, reference, LeapDay::March1),
        date("2025-03-01")
    );
    assert_eq!(
        temporal::next_anniversary(date("2030-01-01"), date("2025-01-01"), LeapDay::February28),
        date("2030-01-01")
    );
}
#[test]
fn daylight_saving_does_not_silently_guess() {
    assert!(temporal::utc_instant(
        date("2026-03-08"),
        NaiveTime::from_hms_opt(2, 30, 0).unwrap(),
        "America/Chicago"
    )
    .is_err());
    assert!(temporal::utc_instant(
        date("2026-11-01"),
        NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
        "America/Chicago"
    )
    .is_err());
    let v = temporal::utc_instant(
        date("2026-01-01"),
        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        "America/Chicago",
    )
    .unwrap();
    assert_eq!(v.to_rfc3339(), "2026-01-01T18:00:00+00:00");
}
#[test]
fn geography_handles_dateline_and_antipodes() {
    let mut a = Place::new("A");
    let mut b = Place::new("B");
    a.longitude = 179.;
    b.longitude = -179.;
    let g = geography::between(&a, &b);
    assert_eq!(g.delta_lon, -358.);
    assert_eq!(g.shortest_lon, 2.);
    assert!((g.kilometers - 222.39016).abs() < 0.001);
    assert_eq!(geography::between(&a, &a).kilometers, 0.);
    a.longitude = 0.;
    b.longitude = 180.;
    assert!((geography::between(&a, &b).central_angle - 180.).abs() < 1e-9);
}
#[test]
fn shared_query_is_consistent_and_floor_is_signed() {
    let w = Workspace::demo();
    let s = &w.sessions[0];
    let fs = features(&w.datasets[0], s);
    let hits = fs
        .iter()
        .filter(|f| f.matches("56", &s.settings))
        .collect::<Vec<_>>();
    assert!(hits
        .iter()
        .any(|f| f.domain == "Gematria" && f.owner == "Will"));
    assert!(hits
        .iter()
        .any(|f| f.domain == "Geography" && f.calculation == "Longitude"));
    let mut settings = s.settings.clone();
    settings.rounding = Rounding::Floor;
    assert_eq!(transform(-56.1, &settings), -57.);
    settings.rounding = Rounding::Nearest;
    assert_eq!(transform(55.6, &settings), 56.);
}
#[test]
fn invalid_links_and_unknown_time_survive_interchange() {
    let w = Workspace::demo();
    let encoded = serde_json::to_string(&w).unwrap();
    let mut decoded: Workspace = serde_json::from_str(&encoded).unwrap();
    decoded.validate().unwrap();
    assert!(decoded.datasets[0].observations[0].time.is_none());
    decoded.datasets[0].observations[0].entity_id = Some(id());
    assert!(decoded.validate().is_err());
    assert!(temporal::parse_date("2025-02-30").is_err());
}
