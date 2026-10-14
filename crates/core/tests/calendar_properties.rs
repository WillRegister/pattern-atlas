use atlas_core::{
    analysis,
    cipher::Cipher,
    gematria::{GematriaSearch, SearchPlan},
    model::*,
    temporal,
};

#[test]
fn calendar_properties_handle_year_boundaries_and_gregorian_leap_rules() {
    let mut w = Workspace::demo();
    let s = &mut w.sessions[0];
    for (date, ordinal, left, length) in [
        ("2026-01-01", 1., 364., 365.),
        ("2026-12-31", 365., 0., 365.),
        ("2024-01-01", 1., 365., 366.),
        ("2024-12-31", 366., 0., 366.),
        ("2024-02-29", 60., 306., 366.),
        ("1900-03-01", 60., 305., 365.),
        ("2000-03-01", 61., 305., 366.),
        ("2100-03-01", 60., 305., 365.),
        ("0001-01-01", 1., 364., 365.),
        ("9998-12-31", 365., 0., 365.),
    ] {
        s.active_date = temporal::parse_date(date).unwrap();
        let facts = analysis::active_date_features(s);
        let value = |name| facts.iter().find(|f| f.calculation == name).unwrap().raw;
        assert_eq!(value("Day of year"), ordinal, "{date}");
        assert_eq!(value("Days left in year"), left, "{date}");
        assert_eq!(value("Days in year"), length, "{date}");
        assert_eq!(ordinal + left, length);
    }
    s.active_date = temporal::parse_date("2021-01-01").unwrap();
    let facts = analysis::active_date_features(s);
    let value = |name| facts.iter().find(|f| f.calculation == name).unwrap().raw;
    assert_eq!(value("ISO week number"), 53.);
    assert_eq!(value("ISO week year"), 2020.);
    assert_eq!(value("Calendar year"), 2021.);
    assert_eq!(value("Weekday number"), 5.);
    assert_eq!(value("Month number"), 1.);
    assert_eq!(value("Day of month"), 1.);
}

#[test]
fn active_calendar_values_survive_empty_record_scope_and_participate_in_all_numeric_matching() {
    let mut w = Workspace::demo();
    let s = &mut w.sessions[0];
    let d = &w.datasets[0];
    s.active_date = temporal::parse_date("2026-02-25").unwrap();
    s.query = "56".into();
    s.dates.all_dates = false;
    s.dates.focus = None;
    s.dates.comparison = None;
    s.filter.text = "no saved records have this name".into();
    let fs = analysis::features(d, s);
    let day = fs.iter().find(|f| f.calculation == "Day of year").unwrap();
    assert_eq!(day.owner_id, s.id);
    assert_eq!(day.domain, "Dates");
    assert!(day.matches(&s.query, &s.settings));
    let search = GematriaSearch {
        input: "Will".into(),
        source_cipher: Some(Cipher::Ordinal),
        ..Default::default()
    };
    let plan = SearchPlan::new(&search, &s.settings.ciphers).unwrap();
    assert!(plan.matches_feature(day, &s.settings));
    s.filter = Default::default();
    let fs = analysis::features(d, s);
    let groups = analysis::groups(&fs, &s.settings);
    assert!(groups["56"]
        .iter()
        .any(|f| f.owner_id == s.id && f.calculation == "Day of year"));
    assert!(groups["56"].iter().any(|f| f.domain == "Gematria"));
    s.settings.include_end = true;
    s.settings.days = false;
    s.settings.years = false;
    s.settings.months = false;
    s.settings.weeks = false;
    let facts = analysis::active_date_features(s);
    assert_eq!(
        facts
            .iter()
            .find(|f| f.calculation == "Days left in year")
            .unwrap()
            .raw,
        309.
    );
    s.active_date = temporal::parse_date("2026-11-05").unwrap();
    let facts = analysis::active_date_features(s);
    assert_eq!(
        facts
            .iter()
            .find(|f| f.calculation == "Days left in year")
            .unwrap()
            .raw,
        56.
    );
}
