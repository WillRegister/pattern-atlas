use super::*;

fn fixture() -> AtlasApp {
    let mut w = Workspace::demo();
    w.datasets[0].entities[0].aliases = vec!["William".into(), "W".into()];
    let mut other = w.sessions[0].clone();
    other.id = id();
    other.name = "Other session".into();
    w.sessions.push(other);
    let mut store = Store::memory().unwrap();
    let revision = store.save(&w, 0).unwrap();
    AtlasApp::new(w, revision, store, PathBuf::from("unused-test-export-dir"))
}
fn frame(
    app: &mut AtlasApp,
    ctx: &egui::Context,
    time: f64,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1440., 920.),
            )),
            time: Some(time),
            events,
            ..Default::default()
        },
        |ctx| app.render(ctx),
    )
}
fn text_position(output: &egui::FullOutput, text: &str) -> egui::Pos2 {
    output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::epaint::Shape::Text(t) if t.galley.job.text == text => {
                Some(t.pos + t.galley.rect.center().to_vec2())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("Text not rendered: {text}"))
}
fn scroll_to_text(
    app: &mut AtlasApp,
    ctx: &egui::Context,
    mut out: egui::FullOutput,
    text: &str,
    time: f64,
) -> egui::FullOutput {
    for i in 0..5 {
        if out.shapes.iter().any(|s|matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text==text && t.pos.y<880.)) {return out;}
        let t = time + i as f64 * 0.1;
        let _ = frame(
            app,
            ctx,
            t,
            vec![
                egui::Event::PointerMoved(egui::pos2(1390., 700.)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -300.),
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        out = frame(app, ctx, t + 0.05, vec![]);
    }
    out
}
fn click(app: &mut AtlasApp, ctx: &egui::Context, pos: egui::Pos2, time: f64) -> egui::FullOutput {
    let _ = frame(
        app,
        ctx,
        time,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(
        app,
        ctx,
        time + 0.01,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    )
}
fn shortcut(key: egui::Key, shift: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            command: true,
            shift,
            ..Default::default()
        },
    }
}

#[test]
fn alias_remove_button_and_global_keyboard_undo_redo_work_across_panels() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Records".into();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(&mut app, &ctx, text_position(&out, "Aliases & tags"), 0.1);
    let out = frame(&mut app, &ctx, 0.2, vec![]);
    let out = click(&mut app, &ctx, text_position(&out, "Remove"), 0.3);
    assert_eq!(app.workspace.datasets[0].entities[0].aliases, vec!["W"]);
    assert!(app.history.can_undo());
    let _ = click(&mut app, &ctx, text_position(&out, "Dates"), 0.4);
    let _ = frame(&mut app, &ctx, 0.5, vec![shortcut(egui::Key::Z, false)]);
    assert_eq!(
        app.workspace.datasets[0].entities[0].aliases,
        vec!["William", "W"]
    );
    assert!(!app.dirty);
    let _ = frame(&mut app, &ctx, 0.6, vec![shortcut(egui::Key::Y, false)]);
    assert_eq!(app.workspace.datasets[0].entities[0].aliases, vec!["W"]);
    let _ = frame(&mut app, &ctx, 0.7, vec![shortcut(egui::Key::Z, false)]);
    let _ = frame(&mut app, &ctx, 0.8, vec![shortcut(egui::Key::Z, true)]);
    assert_eq!(app.workspace.datasets[0].entities[0].aliases, vec!["W"]);
}
#[test]
fn undo_after_saving_restores_linked_records_and_new_save_uses_current_revision() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    let before = serde_json::to_string(&app.workspace).unwrap();
    let eid = app.workspace.datasets[0].entities[0].id.clone();
    app.remove_entity(&eid);
    let _ = frame(&mut app, &ctx, 1., vec![]);
    app.save_data();
    let revision = app.revision;
    app.undo();
    assert_eq!(serde_json::to_string(&app.workspace).unwrap(), before);
    assert!(app.dirty);
    app.workspace.validate().unwrap();
    app.save_data();
    assert_eq!(app.revision, revision + 1);
    assert_eq!(
        serde_json::to_string(&app.store.load().unwrap().unwrap().0).unwrap(),
        before
    );
}
#[test]
fn deleting_active_session_and_undo_restores_selection_and_dataset() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    let sid = app.workspace.sessions[0].id.clone();
    app.remove_session(&sid);
    let _ = frame(&mut app, &ctx, 1., vec![]);
    assert_eq!(app.workspace.sessions.len(), 1);
    assert_eq!(app.workspace.datasets[0].entities.len(), 1);
    app.undo();
    assert_eq!(app.workspace.sessions[app.session].id, sid);
    assert_eq!(app.workspace.sessions.len(), 2);
    app.workspace.validate().unwrap();
}
#[test]
fn invalid_draft_is_undoable_and_inactive_session_deletion_keeps_it() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    let sid = app.workspace.sessions[1].id.clone();
    app.active_date_text = "2026-".into();
    let _ = frame(&mut app, &ctx, 1., vec![]);
    app.remove_session(&sid);
    let _ = frame(&mut app, &ctx, 2., vec![]);
    assert_eq!(app.active_date_text, "2026-");
    app.undo();
    assert_eq!(app.active_date_text, "2026-");
    assert!(app.input_valid().is_err());
    app.undo();
    assert!(app.input_valid().is_ok());
    assert!(!app.dirty);
}
#[test]
fn reopening_saved_revision_in_memory_can_be_undone_without_rewinding_database() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    let original = app.workspace.datasets[0].entities[0].name.clone();
    app.workspace.datasets[0].entities[0].name = "Edited".into();
    let _ = frame(&mut app, &ctx, 1., vec![]);
    app.save_data();
    let current_revision = app.revision;
    app.restore_saved_revision(1);
    let _ = frame(&mut app, &ctx, 2., vec![]);
    assert_eq!(app.workspace.datasets[0].entities[0].name, original);
    assert_eq!(app.revision, current_revision);
    assert_eq!(
        app.store.load().unwrap().unwrap().0.datasets[0].entities[0].name,
        "Edited"
    );
    app.undo();
    assert_eq!(app.workspace.datasets[0].entities[0].name, "Edited");
    assert!(!app.dirty);
}
#[test]
fn navigating_panels_and_populating_date_buffers_does_not_add_history() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    for (i, tab) in ["Records", "Dates", "Sky", "Comparison"].iter().enumerate() {
        app.tab = tab.to_string();
        let _ = frame(&mut app, &ctx, i as f64, vec![]);
    }
    app.session = 1;
    app.sync_active();
    let _ = frame(&mut app, &ctx, 5., vec![]);
    assert!(!app.history.can_undo());
    assert!(!app.dirty);
}

#[test]
fn focused_text_field_typing_is_grouped_and_global_undo_recovers_valid_date() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    let original = app.active_date_text.clone();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(&mut app, &ctx, text_position(&out, &original), 0.1);
    let _ = frame(&mut app, &ctx, 0.2, vec![egui::Event::Text("X".into())]);
    let _ = frame(&mut app, &ctx, 0.3, vec![egui::Event::Text("Y".into())]);
    assert!(app.input_valid().is_err());
    let _ = frame(&mut app, &ctx, 0.4, vec![shortcut(egui::Key::Z, false)]);
    assert_eq!(app.active_date_text, original);
    assert!(app.input_valid().is_ok());
    assert!(!app.history.can_undo());
    let _ = frame(&mut app, &ctx, 0.5, vec![shortcut(egui::Key::Y, false)]);
    assert!(app.active_date_text.contains("XY"));
    assert!(app.input_valid().is_err());
}

#[test]
fn opening_precise_timestamps_preserves_fractional_seconds_and_has_no_spurious_edit() {
    let mut app = fixture();
    let precise = NaiveTime::from_hms_milli_opt(12, 34, 56, 789).unwrap();
    app.workspace.sessions[0].active_time = precise;
    app.workspace.datasets[0].observations[0].time = Some(precise);
    app.save_data();
    let w = app.workspace.clone();
    let revision = app.revision;
    let mut reopened = AtlasApp::new(
        w,
        revision,
        app.store,
        PathBuf::from("unused-test-export-dir"),
    );
    reopened.tab = "Records".into();
    let ctx = egui::Context::default();
    let _ = frame(&mut reopened, &ctx, 1., vec![]);
    assert_eq!(reopened.workspace.sessions[0].active_time, precise);
    assert_eq!(
        reopened.workspace.datasets[0].observations[0].time,
        Some(precise)
    );
    assert!(!reopened.history.can_undo());
    assert!(!reopened.dirty);
}

#[test]
fn temporal_creation_button_makes_an_unlinked_record_that_can_be_detailed_and_undone() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Dates".into();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(&mut app, &ctx, text_position(&out, "+ New date"), 0.1);
    app.new_date.date = "2024-03-20".into();
    app.new_date.label = "Equinox observation".into();
    app.new_date.place_id = None;
    let out = frame(&mut app, &ctx, 0.2, vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Add date to dataset"),
        0.3,
    );
    let id = app.temporal_date_id.clone().unwrap();
    let o = app.workspace.datasets[0]
        .observations
        .iter()
        .find(|o| o.id == id)
        .unwrap();
    assert!(o.entity_id.is_none());
    assert!(o.time.is_none());
    assert_eq!(
        app.workspace.sessions[0].active_observation,
        Some(id.clone())
    );
    app.workspace.validate().unwrap();
    let out = frame(&mut app, &ctx, 0.4, vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Selected date · entity, location and context"),
        0.5,
    );
    let out = frame(&mut app, &ctx, 0.6, vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Details in Records"),
        0.7,
    );
    assert_eq!(app.tab, "Records");
    assert!(app.records_unlinked);
    app.undo();
    assert!(!app.workspace.datasets[0]
        .observations
        .iter()
        .any(|o| o.id == id));
    app.workspace.validate().unwrap();
    app.redo();
    assert!(app.workspace.datasets[0]
        .observations
        .iter()
        .any(|o| o.id == id));
}
#[test]
fn saved_date_controls_set_observer_time_and_preserve_unknown_source_time() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Dates".into();
    let eid = app.workspace.datasets[0].entities[0].id.clone();
    let pid = app.workspace.datasets[0].places[1].id.clone();
    let mut o = Observation::new(&eid, temporal::parse_date("2024-03-20").unwrap());
    o.label = "Later date".into();
    o.place_id = Some(pid.clone());
    let id = o.id.clone();
    app.workspace.datasets[0].observations.push(o);
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(&mut app, &ctx, text_position(&out, "Next date"), 0.1);
    assert_eq!(app.workspace.sessions[0].dates.focus, Some(id.clone()));
    assert!(app.workspace.sessions[0].active_observation.is_none());
    let out = frame(&mut app, &ctx, 0.2, vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Use focus as active"),
        0.3,
    );
    let s = &app.workspace.sessions[0];
    assert_eq!(s.active_observation, Some(id.clone()));
    assert_eq!(s.active_place, Some(pid));
    assert_eq!(s.timezone, "Europe/Zurich");
    assert!(s.time_assumed);
    assert_eq!(s.active_time.hour(), 12);
    let sky = atlas_core::sky::for_session(&app.workspace.datasets[0], s).unwrap();
    assert_eq!(sky.utc.hour(), 11);
    app.shift_time(chrono::Duration::hours(1)).unwrap();
    assert!(app.workspace.datasets[0]
        .observations
        .iter()
        .find(|o| o.id == id)
        .unwrap()
        .time
        .is_none());
}
#[test]
fn live_clock_and_map_navigation_do_not_create_undo_steps() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    let before = serde_json::to_string(&app.workspace).unwrap();
    app.live_time = true;
    let _ = frame(&mut app, &ctx, 1., vec![]);
    let _ = frame(&mut app, &ctx, 2., vec![]);
    assert!(!app.dirty);
    assert!(!app.history.can_undo());
    assert_eq!(serde_json::to_string(&app.workspace).unwrap(), before);
    app.live_time = false;
    for (i, tab) in ["Earth", "Sky"].iter().enumerate() {
        app.tab = tab.to_string();
        let _ = frame(&mut app, &ctx, 3. + i as f64, vec![]);
    }
    assert!(!app.history.can_undo());
}

#[test]
fn editing_an_active_source_date_refreshes_the_viewing_time() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    let o = app.workspace.datasets[0].observations[0].clone();
    app.activate_observation(&o);
    app.tab = "Dates".into();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Selected date · entity, location and context"),
        0.1,
    );
    app.date_text.insert(o.id.clone(), "2024-03-21".into());
    let _ = frame(&mut app, &ctx, 0.2, vec![]);
    assert_eq!(
        app.workspace.sessions[0].active_date,
        temporal::parse_date("2024-03-21").unwrap()
    );
    assert!(app.workspace.datasets[0].observations[0].time.is_none());
    app.workspace.validate().unwrap();
}
#[test]
fn importing_old_json_through_the_ui_upgrades_and_can_be_undone() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    let mut old = serde_json::to_value(&app.workspace).unwrap();
    old["schema_version"] = 1.into();
    old["datasets"][0]["entities"][0]["name"] = "Imported".into();
    app.import_text = serde_json::to_string(&old).unwrap();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Import a Pattern Atlas workspace"),
        0.1,
    );
    let out = frame(&mut app, &ctx, 0.2, vec![]);
    let out = scroll_to_text(
        &mut app,
        &ctx,
        out,
        "Load pasted JSON as workspace (unsaved)",
        0.21,
    );
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Load pasted JSON as workspace (unsaved)"),
        1.,
    );
    assert_eq!(app.workspace.schema_version, 7);
    assert_eq!(app.workspace.datasets[0].entities[0].name, "Imported");
    app.undo();
    assert_eq!(app.workspace.datasets[0].entities[0].name, "Will");
}

#[test]
fn place_rename_keeps_editor_open_and_keyboard_focus_for_multiple_letters() {
    let mut app = fixture();
    app.tab = "Records".into();
    app.records_unlinked = true;
    let ctx = egui::Context::default();
    let name = app.workspace.datasets[0].places[0].name.clone();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let header = out
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::epaint::Shape::Text(t) if t.galley.job.text == name && t.pos.x > 240. => {
                Some(t.pos + t.galley.rect.center().to_vec2())
            }
            _ => None,
        })
        .unwrap();
    let _ = click(&mut app, &ctx, header, 0.1);
    let out = frame(&mut app, &ctx, 1., vec![]);
    let field = out
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::epaint::Shape::Text(t)
                if t.galley.job.text == name && t.pos.x > 240. && t.pos.y > header.y =>
            {
                Some(t.pos + t.galley.rect.center().to_vec2())
            }
            _ => None,
        })
        .unwrap();
    let _ = click(&mut app, &ctx, field, 1.1);
    let focus = ctx
        .memory(|m| m.focused())
        .expect("Place name editor focused");
    for (i, letter) in ["X", "Y", "Z"].iter().enumerate() {
        let _ = frame(
            &mut app,
            &ctx,
            2. + i as f64,
            vec![egui::Event::Text(letter.to_string())],
        );
        let out = frame(&mut app, &ctx, 2.1 + i as f64, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), Some(focus));
        let _ = text_position(&out, "Latitude");
    }
    assert!(app.workspace.datasets[0].places[0].name.contains("XYZ"));
}

#[test]
fn adding_date_updates_saved_selection_active_clock_and_computed_features_on_next_frame() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Dates".into();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let out = click(&mut app, &ctx, text_position(&out, "+ New date"), 0.1);
    assert!(app.show_new_date);
    assert_eq!(
        out.viewport_output[&egui::ViewportId::ROOT].repaint_delay,
        std::time::Duration::ZERO
    );
    app.new_date.label = "Refresh regression".into();
    app.new_date.date = "2024-05-20".into();
    app.new_date.place_id = None;
    let out = frame(&mut app, &ctx, 1., vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Add date to dataset"),
        1.1,
    );
    let out = frame(&mut app, &ctx, 2., vec![]);
    assert!(!app.show_new_date);
    let _ = text_position(&out, "2024-05-20 · Unlinked · Refresh regression");
    assert_eq!(app.active_date_text, "2024-05-20");
    assert!(app
        .feature_cache
        .iter()
        .any(|f| Some(&f.owner_id) == app.temporal_date_id.as_ref()
            && f.calculation == "Signed days from event"
            && f.raw == 0.));
    app.undo();
    assert_eq!(app.workspace.datasets[0].observations.len(), 1);
}

#[test]
fn reviewed_bulk_import_adds_linked_data_and_numeric_attributes_with_undo_redo() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Import data".into();
    let original = serde_json::to_string(&app.workspace).unwrap();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Load example JSON"),
        0.1,
    );
    let out = frame(&mut app, &ctx, 1., vec![]);
    let _ = click(&mut app, &ctx, text_position(&out, "Preview import"), 1.1);
    let out = frame(&mut app, &ctx, 2., vec![]);
    assert!(app.data_tools.preview.is_some());
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Import reviewed data"),
        2.1,
    );
    let _ = frame(&mut app, &ctx, 3., vec![]);
    assert_eq!(app.workspace.datasets.len(), 2);
    assert_eq!(app.session, 2);
    assert!(app
        .feature_cache
        .iter()
        .any(|f| f.domain == "Data" && f.calculation == "score_points" && f.raw == 56.));
    app.workspace.validate().unwrap();
    app.undo();
    assert_eq!(serde_json::to_string(&app.workspace).unwrap(), original);
    app.redo();
    assert_eq!(app.workspace.datasets.len(), 2);
    app.save_data();
    let saved = app.store.load().unwrap().unwrap().0;
    assert_eq!(saved.datasets[1].entities[0].attributes["score_points"], 56);
}

#[test]
fn choosing_city_result_updates_coordinates_preserves_name_and_can_be_undone() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Records".into();
    app.records_unlinked = true;
    let d = &app.workspace.datasets[0];
    let id = d.places[0].id.clone();
    let name = d.places[0].name.clone();
    let latitude = d.places[0].latitude;
    app.geo_target = Some((d.id.clone(), id));
    app.open_city_search = true;
    app.city_lookup.results.push(serde_json::from_str(r#"{"name":"Springfield","admin1":"Illinois","country":"United States","latitude":39.8,"longitude":-89.6,"timezone":"America/Chicago"}"#).unwrap());
    let _ = frame(&mut app, &ctx, 0., vec![]);
    let out = frame(&mut app, &ctx, 1., vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, &format!("Apply to {name}")),
        1.1,
    );
    let p = &app.workspace.datasets[0].places[0];
    assert_eq!(p.latitude, 39.8);
    assert_eq!(p.name, name);
    assert_eq!(p.timezone, "America/Chicago");
    assert!(p.source.contains("GeoNames"));
    app.workspace.validate().unwrap();
    app.undo();
    assert_eq!(app.workspace.datasets[0].places[0].latitude, latitude);
    app.redo();
    assert_eq!(app.workspace.datasets[0].places[0].latitude, 39.8);
}

#[test]
fn map_remove_selected_button_cleans_home_event_observer_and_filter_links_with_undo() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Earth".into();
    let pid = app.workspace.datasets[0].places[0].id.clone();
    app.workspace.datasets[0].entities[0].place_id = Some(pid.clone());
    app.workspace.datasets[0].observations[0].place_id = Some(pid.clone());
    app.workspace.sessions[0].filter.place_id = Some(pid.clone());
    let _ = frame(&mut app, &ctx, 0., vec![]);
    let before = serde_json::to_string(&app.workspace).unwrap();
    let out = frame(&mut app, &ctx, 1., vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Remove selected location"),
        1.1,
    );
    assert!(!app.workspace.datasets[0].places.iter().any(|p| p.id == pid));
    assert!(app.workspace.datasets[0].entities[0].place_id.is_none());
    assert!(app.workspace.datasets[0].observations[0].place_id.is_none());
    assert!(app.workspace.sessions[0].active_place.is_none());
    assert!(app.workspace.sessions[0].locations.a.is_none());
    assert!(app.workspace.sessions[0].filter.place_id.is_none());
    app.workspace.validate().unwrap();
    app.undo();
    assert_eq!(serde_json::to_string(&app.workspace).unwrap(), before);
    app.redo();
    assert_eq!(app.workspace.datasets[0].places.len(), 1);
}

fn nfl_fixture() -> AtlasApp {
    let mut app = fixture();
    let file = atlas_core::bulk::ImportFile::parse(include_str!(
        "../../../datasets/nfl-teams-2026-10-13.json"
    ))
    .unwrap();
    let id = file.apply(&mut app.workspace, None).unwrap();
    app.session = app.workspace.sessions.len() - 1;
    app.workspace.sessions[app.session].name = "NFL".into();
    app.new_session_dataset = id;
    app.sync_active();
    app.history = History::new(app.snapshot(), 100);
    app
}
#[test]
fn nfl_scope_and_browse_preserve_full_records_clear_filters_and_recover_through_undo() {
    let mut app = nfl_fixture();
    let ctx = egui::Context::default();
    let di = app.dataset_index();
    assert_eq!(app.workspace.datasets[di].entities.len(), 32);
    app.workspace.sessions[app.session].filter.conference = "NFC".into();
    app.workspace.sessions[app.session].filter.division = "NFC West".into();
    app.tab = "Browse".into();
    let _ = frame(&mut app, &ctx, 0., vec![]);
    assert_eq!(app.scope.entities.len(), 4);
    assert_eq!(app.scope.places.len(), 4);
    let entities = &app.workspace.datasets[di].entities;
    assert!(app
        .feature_cache
        .iter()
        .filter(|f| f.domain == "Gematria")
        .all(|f| entities
            .iter()
            .find(|e| e.id == f.owner_id)
            .unwrap()
            .attributes["division"]
            == "NFC West"));
    let out = frame(&mut app, &ctx, 1., vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Clear data filters"),
        1.1,
    );
    let _ = frame(&mut app, &ctx, 2., vec![]);
    assert_eq!(app.scope.entities.len(), 32);
    assert_eq!(app.workspace.datasets[di].entities.len(), 32);
    app.undo();
    let _ = frame(&mut app, &ctx, 3., vec![]);
    assert_eq!(app.scope.entities.len(), 4);
    app.browse_all = true;
    app.browse_search = "Seahawks".into();
    let out = frame(&mut app, &ctx, 4., vec![]);
    let _ = click(&mut app, &ctx, text_position(&out, "Open record"), 4.1);
    assert_eq!(app.tab, "Records");
    assert_eq!(
        app.workspace.datasets[di].entities[app.entity].name,
        "Seattle Seahawks"
    );
}
#[test]
fn gematria_uses_cipher_headers_and_toggle_removes_column_without_changing_terms() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Gematria".into();
    let count = |out: &egui::FullOutput, text: &str| {
        out.shapes
            .iter()
            .filter(|s| matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text==text))
            .count()
    };
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = text_position(&out, "Term");
    let _ = text_position(&out, "Entity");
    let _ = text_position(&out, "William");
    assert_eq!(count(&out, "Ordinal"), 2); // checkbox and cipher column
    assert!(out.shapes.iter().any(|s|matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text=="56" && t.galley.job.sections.iter().any(|s|s.format.color==HIT))));
    let _ = click(&mut app, &ctx, text_position(&out, "Ordinal"), 0.1);
    let out = frame(&mut app, &ctx, 1., vec![]);
    assert_eq!(count(&out, "Ordinal"), 1);
    let _ = text_position(&out, "William");
    assert!(!app.workspace.sessions[0]
        .settings
        .ciphers
        .contains(&Cipher::Ordinal));
    app.undo();
    let out = frame(&mut app, &ctx, 2., vec![]);
    assert_eq!(count(&out, "Ordinal"), 2);
}

#[test]
fn marker_context_menu_retains_target_when_pointer_moves_into_menu() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Earth".into();
    let pid = app.workspace.datasets[0].places[0].id.clone();
    let name = app.workspace.datasets[0].places[0].name.clone();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let pos = out
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::epaint::Shape::Circle(c) if (c.radius - 10.).abs() < 0.01 => Some(c.center),
            _ => None,
        })
        .expect("Active marker painted");
    let _ = frame(
        &mut app,
        &ctx,
        0.1,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Secondary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    let _ = frame(
        &mut app,
        &ctx,
        0.2,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Secondary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    let out = frame(&mut app, &ctx, 1., vec![]);
    let target = text_position(&out, &format!("Remove {name}"));
    let _ = click(&mut app, &ctx, target, 1.1);
    assert!(!app.workspace.datasets[0].places.iter().any(|p| p.id == pid));
    app.undo();
    assert!(app.workspace.datasets[0].places.iter().any(|p| p.id == pid));
}
#[test]
fn double_click_on_existing_marker_does_not_add_a_duplicate_and_fit_locations_zoom_in() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Earth".into();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let pos = out
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::epaint::Shape::Circle(c) if (c.radius - 10.).abs() < 0.01 => Some(c.center),
            _ => None,
        })
        .unwrap();
    let _ = click(&mut app, &ctx, pos, 0.1);
    let _ = click(&mut app, &ctx, pos, 0.2);
    assert_eq!(app.workspace.datasets[0].places.len(), 2);
    let mut app = nfl_fixture();
    app.tab = "Earth".into();
    app.workspace.sessions[app.session].locations.all_locations = true;
    let ctx = egui::Context::default();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(&mut app, &ctx, text_position(&out, "Fit locations"), 0.1);
    assert!(app.map_view.zoom > 1.);
    assert!((-125. ..-70.).contains(&app.map_view.longitude));
}

#[test]
fn location_dropdown_enable_scope_and_undo_refresh_global_features_and_map_connection() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Earth".into();
    let mut p = Place::new("Third selectable location");
    p.latitude = -12.;
    p.longitude = 87.;
    let third = p.id.clone();
    app.workspace.datasets[0].places.push(p);
    let out = frame(&mut app, &ctx, 0., vec![]);
    let b_name = app.workspace.datasets[0].places[1].name.clone();
    let _ = click(&mut app, &ctx, text_position(&out, &b_name), 0.1);
    let out = frame(&mut app, &ctx, 0.3, vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Third selectable location"),
        0.4,
    );
    let out = frame(&mut app, &ctx, 0.6, vec![]);
    assert_eq!(
        app.workspace.sessions[0].locations.b.as_deref(),
        Some(third.as_str())
    );
    assert!(app
        .feature_cache
        .iter()
        .any(|f| f.owner_id == third && f.calculation == "Longitude"));
    let _ = text_position(
        &out,
        "A ↔ B · Illustrative point at 56°E ↔ Third selectable location",
    );
    assert_eq!(app.map_view.comparison[1].as_deref(), Some(third.as_str()));
    assert!(out.shapes.iter().any(|s| matches!(&s.shape, egui::epaint::Shape::LineSegment { stroke, .. } if (stroke.width - 2.5).abs() < 0.01)));
    let _ = click(&mut app, &ctx, text_position(&out, "Include B"), 0.7);
    let out = frame(&mut app, &ctx, 0.9, vec![]);
    assert!(app.map_view.comparison[1].is_none());
    assert!(!app
        .feature_cache
        .iter()
        .any(|f| f.calculation == "Great-circle distance" || f.owner_id == third));
    assert!(!out.shapes.iter().any(|s| matches!(&s.shape, egui::epaint::Shape::LineSegment { stroke, .. } if (stroke.width - 2.5).abs() < 0.01)));
    app.undo();
    let _ = frame(&mut app, &ctx, 1.1, vec![]);
    assert!(app.workspace.sessions[0].locations.include_b);
    app.save_data();
    let saved = app.store.load().unwrap().unwrap().0;
    assert_eq!(
        saved.sessions[0].locations.b.as_deref(),
        Some(third.as_str())
    );
    let out = frame(&mut app, &ctx, 1.3, vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "All filtered locations"),
        1.4,
    );
    let _ = frame(&mut app, &ctx, 1.6, vec![]);
    assert_eq!(
        app.feature_cache
            .iter()
            .filter(|f| f.calculation == "Central angle")
            .count(),
        3
    );
    assert_eq!(app.map_view.visible_places.as_ref().unwrap().len(), 3);
    app.undo();
    let out = frame(&mut app, &ctx, 1.8, vec![]);
    assert_eq!(
        app.feature_cache
            .iter()
            .filter(|f| f.calculation == "Central angle")
            .count(),
        1
    );
    let _ = click(&mut app, &ctx, text_position(&out, "Globe"), 1.9);
    let out = frame(&mut app, &ctx, 2.1, vec![]);
    assert!(app.map_view.globe);
    assert!((56. ..87.).contains(&app.map_view.longitude));
    assert!(out.shapes.iter().any(|s| matches!(&s.shape, egui::epaint::Shape::LineSegment { stroke, .. } if (stroke.width - 2.5).abs() < 0.01)));
}

#[test]
fn observe_b_retains_instant_and_map_pair_selection_preserves_observer_with_undo() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Earth".into();
    let before = app.workspace.sessions[0].clone();
    let utc =
        temporal::utc_instant(before.active_date, before.active_time, &before.timezone).unwrap();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(&mut app, &ctx, text_position(&out, "Observe B"), 0.1);
    let _ = frame(&mut app, &ctx, 0.3, vec![]);
    let s = &app.workspace.sessions[0];
    assert_eq!(s.active_place, before.locations.b);
    assert_eq!(s.timezone, "Europe/Zurich");
    assert_eq!(
        temporal::utc_instant(s.active_date, s.active_time, &s.timezone).unwrap(),
        utc
    );
    app.map_view.click_target = 2;
    let observer = s.active_place.clone();
    app.select_map_location(before.locations.a.clone().unwrap());
    let _ = frame(&mut app, &ctx, 0.5, vec![]);
    assert_eq!(app.workspace.sessions[0].active_place, observer);
    assert_eq!(app.workspace.sessions[0].locations.b, before.locations.a);
    app.undo();
    assert_eq!(app.workspace.sessions[0].locations.b, before.locations.b);
}

fn football_dates_fixture() -> (AtlasApp, String, String) {
    let mut app = nfl_fixture();
    let di = app.dataset_index();
    let a = app.workspace.datasets[di]
        .observations
        .iter()
        .find(|o| o.label == "Seattle Seahawks · franchise awarded")
        .unwrap()
        .id
        .clone();
    let b = app.workspace.datasets[di]
        .observations
        .iter()
        .find(|o| o.label == "Arizona Cardinals · franchise awarded")
        .unwrap()
        .id
        .clone();
    let s = &mut app.workspace.sessions[app.session];
    s.active_date = temporal::parse_date("2026-10-18").unwrap();
    s.query = "105".into();
    s.dates.all_dates = false;
    app.sync_active();
    app.tab = "Dates".into();
    (app, a, b)
}

#[test]
fn focus_and_comparison_pickers_keep_game_day_and_show_bracketing_anniversaries_with_undo_and_save()
{
    let (mut app, a, b) = football_dates_fixture();
    let ctx = egui::Context::default();
    let before = app.effective_session();
    app.date_search = "Seahawks".into();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let out = click(
        &mut app,
        &ctx,
        text_position(&out, "1920-09-17 · Arizona Cardinals · franchise awarded"),
        0.1,
    );
    let _ = out;
    let out = frame(&mut app, &ctx, 0.15, vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "1974-06-04 · Seattle Seahawks · franchise awarded"),
        0.2,
    );
    let out = frame(&mut app, &ctx, 0.4, vec![]);
    assert_eq!(
        app.workspace.sessions[app.session].dates.focus.as_deref(),
        Some(a.as_str())
    );
    app.comparison_date_search = "Cardinals".into();
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Choose saved date"),
        0.5,
    );
    let out = frame(&mut app, &ctx, 0.7, vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "1920-09-17 · Arizona Cardinals · franchise awarded"),
        0.8,
    );
    let out = frame(&mut app, &ctx, 1., vec![]);
    let s = &app.workspace.sessions[app.session];
    assert_eq!(s.dates.comparison.as_deref(), Some(b.as_str()));
    assert_eq!(s.active_date, before.active_date);
    assert_eq!(s.active_time, before.active_time);
    assert_eq!(s.active_place, before.active_place);
    for text in [
        "136 days",
        "229 days",
        "31 days",
        "334 days",
        "19618 days",
        "Previous-anniversary gap",
        "Next-anniversary gap",
    ] {
        let _ = text_position(&out, text);
    }
    assert!(out.shapes.iter().filter(|s|matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text=="105 days" && t.galley.job.sections.iter().any(|s|s.format.color==HIT))).count()>=2);
    let _ = click(&mut app, &ctx, text_position(&out, "Swap dates"), 1.1);
    assert_eq!(
        app.workspace.sessions[app.session].dates.focus.as_deref(),
        Some(b.as_str())
    );
    app.undo();
    let out = frame(&mut app, &ctx, 1.3, vec![]);
    assert_eq!(
        app.workspace.sessions[app.session].dates.focus.as_deref(),
        Some(a.as_str())
    );
    let _ = click(&mut app, &ctx, text_position(&out, "Years"), 1.4);
    let _ = frame(&mut app, &ctx, 1.6, vec![]);
    assert!(!app.workspace.sessions[app.session].settings.years);
    assert!(app
        .feature_cache
        .iter()
        .any(|f| f.calculation == "Remaining calendar months" && f.raw == 628.));
    app.save_data();
    let saved = app.store.load().unwrap().unwrap().0;
    assert_eq!(
        saved.sessions[app.session].dates.focus.as_deref(),
        Some(a.as_str())
    );
    assert_eq!(
        saved.sessions[app.session].dates.comparison.as_deref(),
        Some(b.as_str())
    );
    assert_eq!(saved.sessions[app.session].active_date, before.active_date);
}

#[test]
fn focus_dataset_browser_filters_numeric_relationships_and_promotes_a_date_to_compare() {
    let (mut app, a, b) = football_dates_fixture();
    let ctx = egui::Context::default();
    app.workspace.sessions[app.session].dates.focus = Some(a);
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Explore focus against the dataset"),
        0.1,
    );
    app.relation_search = "Cardinals".into();
    app.only_date_hits = true;
    let out = frame(&mut app, &ctx, 0.3, vec![]);
    let _ = text_position(
        &out,
        "1 relationships · saved-date rows point toward focus 1974-06-04",
    );
    let out = scroll_to_text(&mut app, &ctx, out, "Compare", 0.4);
    let _ = click(&mut app, &ctx, text_position(&out, "Compare"), 1.1);
    let _ = frame(&mut app, &ctx, 1.3, vec![]);
    assert_eq!(
        app.workspace.sessions[app.session]
            .dates
            .comparison
            .as_deref(),
        Some(b.as_str())
    );
    assert_eq!(
        app.workspace.datasets[app.dataset_index()]
            .observations
            .len(),
        33
    );
    app.workspace.validate().unwrap();
}

#[test]
fn deleting_a_selected_date_clears_session_references_and_undo_restores_them() {
    let (mut app, a, b) = football_dates_fixture();
    let ctx = egui::Context::default();
    let s = &mut app.workspace.sessions[app.session];
    s.dates.focus = Some(a.clone());
    s.dates.comparison = Some(b);
    let mut other = s.clone();
    other.id = id();
    other.name = "Other game".into();
    app.workspace.sessions.push(other);
    let _ = frame(&mut app, &ctx, 0., vec![]);
    let before = serde_json::to_string(&app.workspace).unwrap();
    app.remove_date(&a);
    let _ = frame(&mut app, &ctx, 0.2, vec![]);
    assert!(app
        .workspace
        .sessions
        .iter()
        .all(|s| s.dates.focus.as_deref() != Some(a.as_str())
            && s.dates.comparison.as_deref() != Some(a.as_str())));
    app.workspace.validate().unwrap();
    app.undo();
    assert_eq!(serde_json::to_string(&app.workspace).unwrap(), before);
}

#[test]
fn live_gematria_distinguishes_cross_cipher_rows_and_finds_date_and_geographic_properties() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Gematria".into();
    app.workspace.datasets[0].entities.push(Entity::new("AHP"));
    let s = &mut app.workspace.sessions[0];
    s.gematria.input = "Will".into();
    s.gematria.source_cipher = Some(Cipher::Ordinal);
    s.active_date = temporal::parse_date("2026-10-14").unwrap();
    app.workspace.datasets[0].observations[0].date = s.active_date - chrono::Duration::days(56);
    app.sync_active();
    let out = frame(&mut app, &ctx, 0., vec![]);
    assert!(out.shapes.iter().any(|s|matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text=="AHP" && t.pos.x>240.)));
    let _ = text_position(&out, "Cross-domain matches");
    assert!(app
        .feature_cache
        .iter()
        .any(|f| f.domain == "Dates" && f.raw == 56.));
    assert!(app
        .feature_cache
        .iter()
        .any(|f| f.domain == "Geography" && f.raw == 56.));
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Cross-cipher matches"),
        0.1,
    );
    let out = frame(&mut app, &ctx, 0.3, vec![]);
    assert!(!app.workspace.sessions[0].gematria.cross_cipher);
    assert!(!out
        .shapes
        .iter()
        .any(|s| matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text=="AHP" && t.pos.x>240.)));
    let _ = text_position(&out, "Longitude");
    let _ = text_position(&out, "Signed days from event");
    app.undo();
    let out = frame(&mut app, &ctx, 0.5, vec![]);
    let _ = text_position(&out, "AHP");
    app.workspace.sessions[0].gematria.input = "56".into();
    app.workspace.sessions[0].gematria.source_cipher = None;
    let out = frame(&mut app, &ctx, 0.7, vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Highlight number everywhere"),
        0.8,
    );
    assert_eq!(app.workspace.sessions[0].query, "56");
    app.save_data();
    let saved = app.store.load().unwrap().unwrap().0;
    assert_eq!(saved.sessions[0].gematria.input, "56");
    assert!(saved.sessions[0].gematria.cross_cipher);
}

#[test]
fn saving_live_phrase_is_undoable_and_inline_type_edit_survives_scope_changes() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Gematria".into();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Phrase or number, e.g. Will or 56"),
        0.1,
    );
    let _ = frame(
        &mut app,
        &ctx,
        0.3,
        vec![egui::Event::Text("Glimmer Bridge".into())],
    );
    let out = frame(&mut app, &ctx, 0.5, vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Save input as entity"),
        0.6,
    );
    let _ = frame(&mut app, &ctx, 0.8, vec![]);
    let e = app.workspace.datasets[0]
        .entities
        .iter()
        .find(|e| e.name == "Glimmer Bridge")
        .unwrap();
    let id = e.id.clone();
    assert_eq!(e.kind, "gematria");
    app.undo();
    assert!(!app.workspace.datasets[0]
        .entities
        .iter()
        .any(|e| e.id == id));
    assert_eq!(app.workspace.sessions[0].gematria.input, "Glimmer Bridge");
    app.redo();
    app.workspace.sessions[0].filter.kind = "gematria".into();
    let out = frame(&mut app, &ctx, 1., vec![]);
    let _ = click(&mut app, &ctx, text_position(&out, "gematria"), 1.1);
    let focus = ctx.memory(|m| m.focused()).unwrap();
    let _ = frame(&mut app, &ctx, 1.3, vec![shortcut(egui::Key::A, false)]);
    for (i, letter) in ["p", "e", "r", "s", "o", "n"].into_iter().enumerate() {
        let _ = frame(
            &mut app,
            &ctx,
            1.5 + i as f64,
            vec![egui::Event::Text(letter.into())],
        );
        let _ = frame(&mut app, &ctx, 1.6 + i as f64, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), Some(focus));
    }
    assert_eq!(
        app.workspace.datasets[0]
            .entities
            .iter()
            .find(|e| e.id == id)
            .unwrap()
            .kind,
        "person"
    );
    app.save_data();
    let saved = app.store.load().unwrap().unwrap().0;
    assert_eq!(
        saved.datasets[0]
            .entities
            .iter()
            .find(|e| e.id == id)
            .unwrap()
            .kind,
        "person"
    );
    let out = frame(&mut app, &ctx, 8., vec![]);
    let _ = click(&mut app, &ctx, text_position(&out, "Edit record"), 8.1);
    assert_eq!(app.tab, "Records");
    assert_eq!(app.workspace.datasets[0].entities[app.entity].id, id);
}

#[test]
fn saved_numeric_gematria_uses_literal_value_and_duplicate_names_or_aliases_do_not_overwrite() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Gematria".into();
    app.workspace.sessions[0].gematria.input = "56".into();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "Save input as entity"),
        0.1,
    );
    let out = frame(&mut app, &ctx, 0.3, vec![]);
    let _ = text_position(&out, "Number");
    let e = app.workspace.datasets[0]
        .entities
        .iter()
        .find(|e| e.name == "56")
        .unwrap();
    let id = e.id.clone();
    assert_eq!(e.kind, "gematria");
    let fs = app
        .feature_cache
        .iter()
        .filter(|f| f.owner_id == id)
        .collect::<Vec<_>>();
    assert_eq!(fs.len(), 1);
    assert_eq!(fs[0].raw, 56.);
    assert_eq!(fs[0].calculation, "Literal number");
    let count = app.workspace.datasets[0].entities.len();
    app.workspace.sessions[0].gematria.input = " 56 ".into();
    assert_eq!(app.save_gematria_input().unwrap(), id);
    assert_eq!(app.workspace.datasets[0].entities.len(), count);
    app.workspace.sessions[0].gematria.input = "william".into();
    let existing = app.workspace.datasets[0].entities[0].id.clone();
    assert_eq!(app.save_gematria_input().unwrap(), existing);
    assert_eq!(app.workspace.datasets[0].entities[0].kind, "person");
    assert_eq!(app.workspace.datasets[0].entities.len(), count);
    app.workspace.sessions[0].gematria.input = "!!!".into();
    assert!(app.save_gematria_input().is_err());
    assert_eq!(app.workspace.datasets[0].entities.len(), count);
}

#[test]
fn comparison_has_one_summary_per_value_and_expands_properties_in_that_row() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Comparison".into();
    app.workspace.sessions[0].active_date = temporal::parse_date("2026-02-25").unwrap();
    app.workspace.sessions[0].query = "56".into();
    app.sync_active();
    let out = frame(&mut app, &ctx, 0., vec![]);
    let headers = out
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::epaint::Shape::Text(t) if t.galley.job.text == "56" && t.pos.x > 240. => {
                Some(t.pos + t.galley.rect.center().to_vec2())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(headers.len(), 1);
    assert!(!out.shapes.iter().any(|s|matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text=="Inspect matching groups" || t.galley.job.text=="Record")));
    let before = serde_json::to_string(&app.workspace).unwrap();
    let _ = click(&mut app, &ctx, headers[0], 0.1);
    let out = frame(&mut app, &ctx, 0.5, vec![]);
    for text in [
        "Record",
        "Domain",
        "Calculation",
        "Day of year",
        "Longitude",
        "Ordinal",
    ] {
        let _ = text_position(&out, text);
    }
    assert_eq!(serde_json::to_string(&app.workspace).unwrap(), before);
    let _ = click(&mut app, &ctx, headers[0], 0.7);
    let out = frame(&mut app, &ctx, 1., vec![]);
    assert!(!out.shapes.iter().any(
        |s| matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text=="Day of year")
    ));
    assert_eq!(serde_json::to_string(&app.workspace).unwrap(), before);
}

#[test]
fn temporal_calendar_tiles_are_searchable_without_saved_dates_and_refresh_from_the_toolbar() {
    let mut app = fixture();
    let ctx = egui::Context::default();
    app.tab = "Dates".into();
    let saved_date = app.workspace.datasets[0].observations[0].date;
    let s = &mut app.workspace.sessions[0];
    s.active_date = temporal::parse_date("2026-02-25").unwrap();
    s.query = "56".into();
    s.dates.all_dates = false;
    s.dates.focus = None;
    s.dates.comparison = None;
    app.sync_active();
    let out = frame(&mut app, &ctx, 0., vec![]);
    for text in [
        "Active date · calendar properties",
        "Day of year",
        "Days left in year",
        "Days in year",
        "ISO week number",
    ] {
        let _ = text_position(&out, text);
    }
    assert!(out.shapes.iter().any(|s|matches!(&s.shape,egui::epaint::Shape::Text(t) if t.galley.job.text=="56" && t.pos.x>240. && t.galley.job.sections.iter().any(|s|s.format.color==HIT))));
    let _ = click(
        &mut app,
        &ctx,
        text_position(&out, "More calendar fields"),
        0.1,
    );
    let out = frame(&mut app, &ctx, 0.5, vec![]);
    for text in [
        "Calendar year",
        "Month number",
        "Day of month",
        "Weekday number",
        "ISO week year",
    ] {
        let _ = text_position(&out, text);
    }
    let _ = click(&mut app, &ctx, text_position(&out, "Days"), 0.6);
    let _ = frame(&mut app, &ctx, 0.8, vec![]);
    assert!(!app.workspace.sessions[0].settings.days);
    assert!(app
        .feature_cache
        .iter()
        .any(|f| f.calculation == "Day of year" && f.raw == 56.));
    // Editing the actual toolbar buffer updates the active calendar, while preserving the source record.
    app.active_date_text = "2026-12-31".into();
    let _ = frame(&mut app, &ctx, 1., vec![]);
    let _ = frame(&mut app, &ctx, 1.1, vec![]);
    assert!(app
        .feature_cache
        .iter()
        .any(|f| f.calculation == "Day of year" && f.raw == 365.));
    assert!(app
        .feature_cache
        .iter()
        .any(|f| f.calculation == "Days left in year" && f.raw == 0.));
    assert_eq!(app.workspace.datasets[0].observations[0].date, saved_date);
    assert!(app.workspace.sessions[0].dates.focus.is_none());
}
