use atlas_core::{analysis, model::*, temporal};
fn date(s: &str) -> chrono::NaiveDate {
    temporal::parse_date(s).unwrap()
}

#[test]
fn football_example_has_three_original_relationships_and_bracketing_anniversary_gaps() {
    let s = Settings::default();
    let active = date("2026-10-18");
    let a = date("1974-06-04");
    let b = date("1920-09-17");
    let seattle = temporal::relation(a, active, &s);
    let arizona = temporal::relation(b, active, &s);
    assert_eq!(seattle.original.total_days, 19129);
    assert_eq!(arizona.original.total_days, 38747);
    assert_eq!(temporal::span(a, b, &s).total_days, 19618);
    assert_eq!(seattle.previous, Some(date("2026-06-04")));
    assert_eq!(seattle.next, date("2027-06-04"));
    assert_eq!(seattle.since.unwrap().total_days, 136);
    assert_eq!(seattle.until.total_days, 229);
    assert_eq!(arizona.previous, Some(date("2026-09-17")));
    assert_eq!(arizona.next, date("2027-09-17"));
    assert_eq!(arizona.since.unwrap().total_days, 31);
    assert_eq!(arizona.until.total_days, 334);
    let pair = temporal::anniversary_pair(a, b, active, &s);
    assert_eq!(pair.previous.unwrap().total_days, 105);
    assert_eq!(pair.next.total_days, 105);
    let reversed = temporal::anniversary_pair(b, a, active, &s);
    assert_eq!(reversed.previous.unwrap().total_days, 105);
    assert_eq!(reversed.next.total_days, 105);
}

#[test]
fn anniversaries_handle_today_future_origins_leap_policies_and_supported_boundaries() {
    let mut s = Settings {
        include_end: true,
        ..Default::default()
    };
    let today = temporal::relation(date("2000-06-04"), date("2026-06-04"), &s);
    assert_eq!(today.previous, Some(date("2026-06-04")));
    assert_eq!(today.next, date("2026-06-04"));
    assert_eq!(today.since.unwrap().total_days, 0);
    assert_eq!(today.until.total_days, 0);
    let future = temporal::relation(date("2030-06-04"), date("2026-06-04"), &s);
    assert!(future.previous.is_none());
    assert!(future.since.is_none());
    assert_eq!(future.next, date("2030-06-04"));
    assert_eq!(future.original.sign, -1);
    assert!(temporal::anniversary_pair(
        date("2030-06-04"),
        date("2000-01-01"),
        date("2026-06-04"),
        &s
    )
    .previous
    .is_none());
    let leap = date("2020-02-29");
    assert_eq!(
        temporal::relation(leap, date("2025-02-28"), &s)
            .until
            .total_days,
        0
    );
    s.leap_day = LeapDay::March1;
    let r = temporal::relation(leap, date("2025-02-28"), &s);
    assert_eq!(r.previous, Some(date("2024-02-29")));
    assert_eq!(r.next, date("2025-03-01"));
    assert_eq!(r.until.total_days, 1);
    assert!(
        temporal::previous_anniversary(date("0001-12-31"), date("0001-01-01"), s.leap_day)
            .is_none()
    );
    assert_eq!(
        temporal::next_anniversary(date("0001-12-31"), date("9998-12-31"), s.leap_day),
        date("9998-12-31")
    );
    assert_eq!(
        temporal::next_anniversary(date("0001-01-01"), date("9998-12-31"), s.leap_day).to_string(),
        "9999-01-01"
    );
}

#[test]
fn unit_toggles_recompose_without_losing_days_and_inclusion_does_not_change_countdowns() {
    let mut s = Settings::default();
    let event = date("1974-06-04");
    let active = date("2026-10-18");
    let r = temporal::relation(event, active, &s);
    assert_eq!(
        (r.original.years, r.original.months, r.original.days),
        (52, 4, 14)
    );
    s.years = false;
    assert_eq!(temporal::relation(event, active, &s).original.months, 628);
    s.months = false;
    s.weeks = true;
    let r = temporal::relation(event, active, &s);
    assert_eq!(r.original.weeks * 7 + r.original.days, 19129);
    assert_eq!(
        r.since.as_ref().unwrap().weeks * 7 + r.since.as_ref().unwrap().days,
        136
    );
    s.include_end = true;
    let included = temporal::relation(event, active, &s);
    assert_eq!(included.original.total_days, 19130);
    assert_eq!(included.since.unwrap().total_days, 136);
    assert_eq!(included.until.total_days, 229);
}

#[test]
fn selected_date_scope_limits_global_features_includes_anniversaries_and_honors_filters() {
    let mut w = Workspace::demo();
    let d = &mut w.datasets[0];
    d.observations[0].date = date("1974-06-04");
    let mut b = Observation::unlinked(date("1920-09-17"));
    b.label = "Cardinals franchise award".into();
    let b_id = b.id.clone();
    d.observations.push(b);
    let mut c = Observation::unlinked(date("1990-01-01"));
    c.attributes.insert("unselected_value".into(), 56.into());
    let c_id = c.id.clone();
    d.observations.push(c);
    let s = &mut w.sessions[0];
    s.active_date = date("2026-10-18");
    s.dates.all_dates = false;
    s.dates.comparison = Some(b_id);
    let fs = analysis::features(d, s);
    assert!(!fs.iter().any(|f| f.owner_id == c_id));
    assert_eq!(
        fs.iter()
            .filter(|f| f.calculation == "Pairwise date span")
            .count(),
        1
    );
    assert!(fs
        .iter()
        .any(|f| f.calculation == "Previous-anniversary pair span" && f.raw == 105.));
    assert!(fs
        .iter()
        .any(|f| f.calculation == "Next-anniversary pair span" && f.raw == 105.));
    assert!(fs
        .iter()
        .any(|f| f.calculation == "Days since previous anniversary" && f.raw == 136.));
    assert!(fs
        .iter()
        .any(|f| f.calculation == "Days to next anniversary" && f.raw == 334.));
    s.dates.comparison = s.dates.focus.clone();
    assert_eq!(
        analysis::features(d, s)
            .iter()
            .filter(|f| f.calculation == "Pairwise date span")
            .count(),
        0
    );
    s.dates.all_dates = true;
    assert_eq!(
        analysis::features(d, s)
            .iter()
            .filter(|f| f.calculation == "Pairwise date span")
            .count(),
        3
    );
    s.settings.pair_limit = 0;
    assert!(!analysis::features(d, s)
        .iter()
        .any(|f| f.calculation.ends_with("pair span") || f.calculation == "Pairwise date span"));
    s.dates.all_dates = false;
    s.filter.from = "2001-01-01".into();
    assert!(!analysis::features(d, s)
        .iter()
        .any(|f| f.domain == "Dates" && f.owner_id != s.id));
    s.filter = Default::default();
    s.settings.include_end = true;
    d.observations[2].date = date("2026-10-19");
    s.dates.focus = Some(c_id);
    s.dates.comparison = None;
    assert!(analysis::features(d, s)
        .iter()
        .any(|f| f.calculation == "Signed span total days" && f.raw == -2.));
}

#[test]
fn legacy_upgrade_and_entity_removal_preserve_and_clean_saved_date_selections() {
    let mut value = serde_json::to_value(Workspace::demo()).unwrap();
    value["schema_version"] = 5.into();
    value["sessions"][0]
        .as_object_mut()
        .unwrap()
        .remove("dates");
    let mut w: Workspace = serde_json::from_value(value).unwrap();
    w.upgrade().unwrap();
    assert_eq!(w.schema_version, 7);
    assert!(w.sessions[0].dates.all_dates);
    assert_eq!(
        w.sessions[0].dates.focus.as_deref(),
        Some(w.datasets[0].observations[0].id.as_str())
    );
    let did = w.datasets[0].id.clone();
    let eid = w.datasets[0].entities[0].id.clone();
    w.sessions[0].dates.comparison = w.sessions[0].dates.focus.clone();
    let mut other = w.sessions[0].clone();
    other.id = id();
    w.sessions.push(other);
    w.remove_entity(&did, &eid).unwrap();
    w.validate().unwrap();
    assert!(w
        .sessions
        .iter()
        .all(|s| s.dates.focus.is_none() && s.dates.comparison.is_none()));
    w.sessions[0].dates.focus = Some(id());
    assert!(w.validate().is_err());
}
