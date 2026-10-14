use atlas_core::{analysis, filter, geography, model::*};

#[test]
fn selections_define_global_location_values_and_pair_order_without_hiding_other_domains() {
    let mut w = Workspace::demo();
    let mut c = Place::new("Unselected point");
    c.latitude = -30.;
    c.longitude = -90.;
    c.attributes.insert("recorded_number".into(), 123.into());
    w.datasets[0].places.push(c.clone());
    let d = &w.datasets[0];
    let s = &mut w.sessions[0];
    s.locations.a = Some(d.places[1].id.clone());
    s.locations.b = Some(d.places[0].id.clone());
    let fs = analysis::features(d, s);
    assert!(!fs.iter().any(|f| f.owner_id == c.id));
    assert!(fs.iter().any(|f| f.domain == "Gematria"));
    assert!(fs.iter().any(|f| f.domain == "Dates"));
    let delta = fs
        .iter()
        .find(|f| f.calculation == "Signed latitude delta (B−A)")
        .unwrap();
    assert_eq!(delta.raw, d.places[0].latitude - d.places[1].latitude);
    assert_eq!(delta.owner_id, d.places[1].id);
    s.locations.include_b = false;
    let fs = analysis::features(d, s);
    assert_eq!(fs.iter().filter(|f| f.calculation == "Latitude").count(), 1);
    assert!(!fs.iter().any(|f| f.calculation == "Great-circle distance"));
    s.locations.all_locations = true;
    let fs = analysis::features(d, s);
    assert_eq!(fs.iter().filter(|f| f.calculation == "Latitude").count(), 3);
    assert_eq!(
        fs.iter()
            .filter(|f| f.calculation == "Central angle")
            .count(),
        3
    );
    assert!(fs.iter().any(|f| f.owner_id == c.id && f.raw == 123.));
    s.settings.pair_limit = 1;
    assert_eq!(
        analysis::features(d, s)
            .iter()
            .filter(|f| f.calculation == "Central angle")
            .count(),
        1
    );
}

#[test]
fn selected_places_respect_filters_deduplicate_and_remove_links_across_sessions() {
    let mut w = Workspace::demo();
    let did = w.datasets[0].id.clone();
    let pid = w.datasets[0].places[0].id.clone();
    w.sessions[0].locations.b = Some(pid.clone());
    let s = &w.sessions[0];
    let d = &w.datasets[0];
    assert_eq!(
        analysis::location_places(d, s, &filter::scope(d, s)).len(),
        1
    );
    w.sessions[0].filter.text = "Geneva".into();
    let s = &w.sessions[0];
    let d = &w.datasets[0];
    assert!(analysis::location_places(d, s, &filter::scope(d, s)).is_empty());
    w.sessions.push(w.sessions[0].clone());
    w.sessions[1].id = id();
    w.remove_place(&did, &pid).unwrap();
    assert!(w
        .sessions
        .iter()
        .all(|s| s.locations.a.is_none() && s.locations.b.is_none()));
    w.validate().unwrap();
    w.sessions[0].locations.b = Some(id());
    assert!(w.validate().is_err());
}

#[test]
fn version_four_upgrade_preserves_broad_analysis_and_observer_and_seeds_a() {
    let w = Workspace::demo();
    let mut json = serde_json::to_value(&w).unwrap();
    json["schema_version"] = 4.into();
    json["sessions"][0]
        .as_object_mut()
        .unwrap()
        .remove("locations");
    let mut old: Workspace = serde_json::from_value(json).unwrap();
    old.upgrade().unwrap();
    assert_eq!(old.schema_version, 7);
    assert!(old.sessions[0].locations.all_locations);
    assert_eq!(old.sessions[0].locations.a, w.sessions[0].active_place);
    assert_eq!(old.sessions[0].active_place, w.sessions[0].active_place);
    let restored: Workspace = serde_json::from_str(&serde_json::to_string(&old).unwrap()).unwrap();
    assert_eq!(
        restored.sessions[0].locations.a,
        old.sessions[0].locations.a
    );
    assert!(restored.sessions[0].locations.all_locations);
}

#[test]
fn sampled_routes_are_finite_have_exact_endpoints_and_follow_the_calculated_distance() {
    for (lat_a, lon_a, lat_b, lon_b) in [
        (47.6, -122.3, 33.5, -112.2),
        (0., 179., 0., -179.),
        (0., 0., 0., 180.),
        (90., 56., -90., 56.),
        (10., 20., 10., 20.),
        (34., 45., -34.000001, -134.999999),
    ] {
        let mut a = Place::new("A");
        a.latitude = lat_a;
        a.longitude = lon_a;
        let mut b = Place::new("B");
        b.latitude = lat_b;
        b.longitude = lon_b;
        let route = geography::route(&a, &b);
        assert_eq!(route[0], [lon_a, lat_a]);
        assert_eq!(*route.last().unwrap(), [lon_b, lat_b]);
        assert!(route
            .iter()
            .all(|p| p[0].is_finite() && p[1].is_finite() && p[1].abs() <= 90.));
        let distance = route
            .windows(2)
            .map(|p| {
                let mut a = a.clone();
                let mut b = b.clone();
                a.longitude = p[0][0];
                a.latitude = p[0][1];
                b.longitude = p[1][0];
                b.latitude = p[1][1];
                geography::between(&a, &b).kilometers
            })
            .sum::<f64>();
        assert!((distance - geography::between(&a, &b).kilometers).abs() < 0.001);
    }
}
