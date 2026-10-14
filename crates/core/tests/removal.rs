use atlas_core::model::*;

#[test]
fn deleting_entity_removes_its_dates_and_selections_only() {
    let mut w = Workspace::demo();
    let did = w.datasets[0].id.clone();
    let eid = w.datasets[0].entities[0].id.clone();
    w.sessions[0].selected_entities = vec![eid.clone()];
    let other = Entity::new("Another person");
    let date = w.sessions[0].active_date;
    w.datasets[0]
        .observations
        .push(Observation::new(&other.id, date));
    w.datasets[0].entities.push(other.clone());
    w.remove_entity(&did, &eid).unwrap();
    w.validate().unwrap();
    assert_eq!(w.datasets[0].entities[0].id, other.id);
    assert_eq!(w.datasets[0].observations.len(), 1);
    assert!(w.sessions[0].selected_entities.is_empty());
    assert_eq!(w.datasets[0].places.len(), 2);
}
#[test]
fn deleting_place_keeps_dates_and_clears_references_in_all_sessions() {
    let mut w = Workspace::demo();
    let did = w.datasets[0].id.clone();
    let pid = w.datasets[0].places[0].id.clone();
    w.datasets[0].observations[0].place_id = Some(pid.clone());
    let mut s = w.sessions[0].clone();
    s.id = id();
    w.sessions.push(s);
    w.remove_place(&did, &pid).unwrap();
    w.validate().unwrap();
    assert_eq!(w.datasets[0].observations.len(), 1);
    assert!(w.datasets[0].observations[0].place_id.is_none());
    assert!(w.sessions.iter().all(|s| s.active_place.is_none()));
    assert_eq!(w.datasets[0].places.len(), 1);
}
#[test]
fn session_removal_preserves_dataset_and_final_session() {
    let mut w = Workspace::demo();
    let sid = w.sessions[0].id.clone();
    assert!(w.remove_session(&sid).is_err());
    let mut s = w.sessions[0].clone();
    s.id = id();
    w.sessions.push(s);
    w.remove_session(&sid).unwrap();
    w.validate().unwrap();
    assert_eq!(w.sessions.len(), 1);
    assert_eq!(w.datasets[0].entities.len(), 1);
}
