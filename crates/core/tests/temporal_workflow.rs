use atlas_core::{analysis, model::*, temporal};

#[test]
fn old_workspace_upgrades_without_losing_entity_links_or_unknown_times() {
    let w = Workspace::demo();
    let mut value = serde_json::to_value(&w).unwrap();
    value["schema_version"] = 1.into();
    value["sessions"][0]
        .as_object_mut()
        .unwrap()
        .remove("active_observation");
    value["sessions"][0]["settings"]
        .as_object_mut()
        .unwrap()
        .remove("atmospheric_refraction");
    let mut old: Workspace = serde_json::from_value(value).unwrap();
    old.upgrade().unwrap();
    assert_eq!(old.schema_version, 7);
    assert_eq!(
        old.datasets[0].observations[0].entity_id,
        Some(w.datasets[0].entities[0].id.clone())
    );
    assert!(old.datasets[0].observations[0].time.is_none());
    assert!(old.sessions[0].active_observation.is_none());
}
#[test]
fn unlinked_date_can_be_created_linked_exported_and_recovered() {
    let mut w = Workspace::demo();
    let date = temporal::parse_date("2024-03-20").unwrap();
    let o = Observation::unlinked(date);
    let id = o.id.clone();
    w.datasets[0].observations.push(o);
    w.validate().unwrap();
    let features = analysis::features(&w.datasets[0], &w.sessions[0]);
    assert!(features
        .iter()
        .any(|f| f.owner_id == id && f.domain == "Dates"));
    let eid = w.datasets[0].entities[0].id.clone();
    let pid = w.datasets[0].places[0].id.clone();
    w.datasets[0].observations[1].entity_id = Some(eid);
    w.datasets[0].observations[1].place_id = Some(pid);
    w.datasets[0].observations[1].context = "A sourced date".into();
    w.sessions[0].active_observation = Some(id.clone());
    w.validate().unwrap();
    let json = serde_json::to_string(&w).unwrap();
    let restored: Workspace = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.datasets[0].observations[1].id, id);
    assert!(restored.datasets[0].observations[1].time.is_none());
}
#[test]
fn entity_deletion_keeps_unlinked_dates_and_clears_active_date_references() {
    let mut w = Workspace::demo();
    let eid = w.datasets[0].entities[0].id.clone();
    let did = w.datasets[0].id.clone();
    w.sessions[0].active_observation = Some(w.datasets[0].observations[0].id.clone());
    let o = Observation::unlinked(w.sessions[0].active_date);
    let id = o.id.clone();
    w.datasets[0].observations.push(o);
    w.remove_entity(&did, &eid).unwrap();
    w.validate().unwrap();
    assert_eq!(w.datasets[0].observations[0].id, id);
    assert!(w.sessions[0].active_observation.is_none());
}
