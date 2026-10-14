use atlas_core::model::Workspace;
use atlas_storage::Store;

#[test]
fn archived_workspace_can_be_recovered_after_reopening() {
    let path =
        std::env::temp_dir().join(format!("atlas-history-{}.sqlite", atlas_core::model::id()));
    let mut original = Workspace::demo();
    original.datasets[0].entities[0].aliases = vec!["Recover me".into()];
    {
        let mut store = Store::open(&path).unwrap();
        let r = store.save(&original, 0).unwrap();
        let mut removed = original.clone();
        removed.datasets[0].entities[0].aliases.clear();
        store.save(&removed, r).unwrap();
    }
    {
        let mut reopened = Store::open(&path).unwrap();
        let revisions = reopened.revisions().unwrap();
        assert_eq!(revisions.len(), 2);
        let recovered = reopened.load_revision(1).unwrap();
        assert_eq!(
            recovered.datasets[0].entities[0].aliases,
            vec!["Recover me"]
        );
        assert_eq!(reopened.load().unwrap().unwrap().1, 2);
        assert_eq!(reopened.save(&recovered, 2).unwrap(), 3);
    }
    std::fs::remove_file(path).unwrap();
}
#[test]
fn saved_history_retains_only_latest_fifty_revisions() {
    let mut store = Store::memory().unwrap();
    let mut w = Workspace::demo();
    for revision in 0..51 {
        w.sessions[0].name = format!("Revision {}", revision + 1);
        store.save(&w, revision).unwrap();
    }
    let rows = store.revisions().unwrap();
    assert_eq!(rows.len(), 50);
    assert_eq!(rows.first().unwrap().revision, 51);
    assert_eq!(rows.last().unwrap().revision, 2);
    assert!(store.load_revision(1).is_err());
    assert_eq!(
        store.load_revision(2).unwrap().sessions[0].name,
        "Revision 2"
    );
}

#[test]
fn roundtrip_conflict_and_invalid_save_preserve_saved_data() {
    let mut store = Store::memory().unwrap();
    assert!(store.load().unwrap().is_none());
    let w = Workspace::demo();
    let revision = store.save(&w, 0).unwrap();
    let (loaded, r) = store.load().unwrap().unwrap();
    assert_eq!(r, revision);
    assert_eq!(
        loaded.datasets[0].entities[0].id,
        w.datasets[0].entities[0].id
    );
    assert!(store.save(&w, 0).is_err());
    let mut invalid = w.clone();
    invalid.datasets[0].places[0].latitude = 91.;
    assert!(store.save(&invalid, revision).is_err());
    assert_eq!(store.load().unwrap().unwrap().1, revision);
    assert_eq!(store.save(&w, revision).unwrap(), revision + 1);
}
