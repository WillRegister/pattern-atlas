use atlas_core::{
    analysis::{self, Feature},
    cipher::Cipher,
    gematria::{GematriaSearch, InputKind, SearchPlan},
    model::*,
};

fn query(input: &str, cross: bool, source: Option<Cipher>) -> GematriaSearch {
    GematriaSearch {
        input: input.into(),
        cross_cipher: cross,
        source_cipher: source,
        ..Default::default()
    }
}
#[test]
fn phrase_matching_can_follow_same_cipher_or_cross_enabled_ciphers() {
    let enabled = vec![
        Cipher::Ordinal,
        Cipher::Reduction,
        Cipher::Reverse,
        Cipher::ReverseReduction,
    ];
    let s = Settings::default();
    let same = SearchPlan::new(&query("Will", false, Some(Cipher::Ordinal)), &enabled).unwrap();
    let cross = SearchPlan::new(&query("Will", true, Some(Cipher::Ordinal)), &enabled).unwrap();
    assert_eq!(same.kind, InputKind::Phrase);
    assert_eq!(same.targets[0].value, 56.);
    assert_eq!(Cipher::Reverse.total("AHP"), 56);
    assert_eq!(Cipher::Ordinal.total("AHP"), 25);
    assert!(same.matches_cell(Some(Cipher::Ordinal), 56., &s));
    assert!(!same.matches_cell(Some(Cipher::Reverse), 56., &s));
    assert!(cross.matches_cell(Some(Cipher::Reverse), 56., &s));
    assert!(!cross.matches_cell(Some(Cipher::Primes), 56., &s));
    let same_all = SearchPlan::new(&query("Will", false, None), &enabled).unwrap();
    assert!(same_all.matches_cell(Some(Cipher::Reverse), 52., &s));
    assert!(!same_all.matches_cell(Some(Cipher::Ordinal), 52., &s));
    let cross_all = SearchPlan::new(&query("Will", true, None), &enabled).unwrap();
    assert!(cross_all.matches_cell(Some(Cipher::Ordinal), 52., &s));
    let disabled = SearchPlan::new(&query("Will", true, Some(Cipher::Primes)), &enabled).unwrap();
    assert!(disabled.targets.is_empty());
}
#[test]
fn literal_numbers_can_match_any_cipher_or_be_locked_and_never_become_zero_totals() {
    let enabled = Cipher::ALL;
    let s = Settings::default();
    let any = SearchPlan::new(&query("56", false, None), &enabled).unwrap();
    assert_eq!(any.kind, InputKind::Number(56.));
    assert_eq!(any.targets.len(), 1);
    assert!(any.matches_cell(Some(Cipher::Ordinal), 56., &s));
    assert!(any.matches_cell(Some(Cipher::Reverse), 56., &s));
    let locked = SearchPlan::new(&query("56", false, Some(Cipher::Ordinal)), &enabled).unwrap();
    assert!(!locked.matches_cell(Some(Cipher::Reverse), 56., &s));
    assert!(locked.matches_cell(None, 56., &s));
    let mut w = Workspace::demo();
    let mut number = Entity::new("56");
    number.kind = "gematria".into();
    let id = number.id.clone();
    w.datasets[0].entities.push(number);
    let fs = analysis::features(&w.datasets[0], &w.sessions[0]);
    let numeric = fs.iter().filter(|f| f.owner_id == id).collect::<Vec<_>>();
    assert_eq!(numeric.len(), 1);
    assert_eq!(numeric[0].calculation, "Literal number");
    assert_eq!(numeric[0].raw, 56.);
    w.sessions[0].settings.ciphers.clear();
    assert!(analysis::features(&w.datasets[0], &w.sessions[0])
        .iter()
        .any(|f| f.owner_id == id && f.raw == 56.));
}
#[test]
fn cross_domain_matching_uses_display_rounding_selected_scopes_and_domain_filters() {
    let plan = SearchPlan::new(&query("Will", false, Some(Cipher::Ordinal)), &Cipher::ALL).unwrap();
    let mut w = Workspace::demo();
    let s = &mut w.sessions[0];
    s.active_date = chrono::NaiveDate::from_ymd_opt(2026, 10, 14).unwrap();
    w.datasets[0].observations[0].date = s.active_date - chrono::Duration::days(56);
    let fs = analysis::features(&w.datasets[0], s);
    assert!(fs.iter().any(|f| f.domain == "Geography"
        && f.calculation == "Longitude"
        && plan.matches_feature(f, &s.settings)));
    assert!(fs.iter().any(|f| f.domain == "Dates"
        && f.calculation == "Signed days from event"
        && plan.matches_feature(f, &s.settings)));
    let mut search = query("Will", true, None);
    search.geography = false;
    assert!(!search.includes_domain("Geography"));
    assert!(search.includes_domain("Dates"));
    assert!(!search.includes_domain("Gematria"));
    let f = Feature {
        owner_id: String::new(),
        owner: String::new(),
        domain: "Geography".into(),
        calculation: "distance".into(),
        unit: "km".into(),
        raw: 56.9,
        explanation: String::new(),
    };
    let mut settings = Settings {
        rounding: Rounding::Floor,
        ..Default::default()
    };
    assert!(plan.matches_feature(&f, &settings));
    settings.rounding = Rounding::Nearest;
    assert!(!plan.matches_feature(&f, &settings));
    s.dates.all_dates = false;
    s.dates.focus = None;
    s.dates.comparison = None;
    s.locations.include_a = false;
    s.locations.include_b = false;
    assert!(!analysis::features(&w.datasets[0], s)
        .iter()
        .any(|f| (f.domain == "Dates" || f.domain == "Geography")
            && plan.matches_feature(f, &s.settings)));
}
#[test]
fn invalid_inputs_and_old_workspace_migration_preserve_records_and_search_defaults() {
    for input in ["NaN", "inf", "!!!", "你好", "1,000"] {
        assert!(
            SearchPlan::new(&query(input, true, None), &Cipher::ALL).is_err(),
            "{input}"
        );
    }
    assert!(SearchPlan::new(&query(&"W".repeat(4097), true, None), &Cipher::ALL).is_err());
    assert_eq!(
        SearchPlan::new(&query("  ", true, None), &Cipher::ALL)
            .unwrap()
            .kind,
        InputKind::Empty
    );
    assert_eq!(
        SearchPlan::new(&query("-56.5", true, None), &[])
            .unwrap()
            .kind,
        InputKind::Number(-56.5)
    );
    let original = Workspace::demo();
    let mut json = serde_json::to_value(&original).unwrap();
    json["schema_version"] = 6.into();
    json["sessions"][0]
        .as_object_mut()
        .unwrap()
        .remove("gematria");
    let mut old: Workspace = serde_json::from_value(json).unwrap();
    old.upgrade().unwrap();
    assert_eq!(old.schema_version, 7);
    assert!(old.sessions[0].gematria.input.is_empty());
    assert!(old.sessions[0].gematria.cross_cipher);
    assert_eq!(
        serde_json::to_value(&old.datasets).unwrap(),
        serde_json::to_value(&original.datasets).unwrap()
    );
    old.sessions[0].gematria = query("Will", false, Some(Cipher::Ordinal));
    let restored: Workspace = serde_json::from_str(&serde_json::to_string(&old).unwrap()).unwrap();
    assert_eq!(restored.sessions[0].gematria.input, "Will");
    assert!(!restored.sessions[0].gematria.cross_cipher);
    assert_eq!(
        restored.sessions[0].gematria.source_cipher,
        Some(Cipher::Ordinal)
    );
}
