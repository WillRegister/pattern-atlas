use crate::history::History;
use atlas_core::{
    analysis::{self, Feature},
    cipher::Cipher,
    model::*,
    temporal,
};
use atlas_storage::Store;
use chrono::{NaiveTime, Timelike, Utc};
use eframe::egui::{self, Color32, RichText};
use std::{collections::BTreeMap, path::PathBuf};

pub(crate) const ACCENT: Color32 = Color32::from_rgb(110, 218, 207);
pub(crate) const HIT: Color32 = Color32::from_rgb(255, 210, 105);
#[path = "browsing.rs"]
mod browsing;
#[path = "comparison_workspace.rs"]
mod comparison_workspace;
#[path = "data_tools.rs"]
pub(crate) mod data_tools;
#[path = "gematria_workspace.rs"]
mod gematria_workspace;
#[path = "locations.rs"]
mod locations;
#[path = "temporal_workspace.rs"]
mod temporal_workspace;
#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
#[path = "workbench.rs"]
mod workbench;

#[derive(Clone)]
struct EditorSnapshot {
    workspace_json: String,
    date_drafts: BTreeMap<String, String>,
    time_drafts: BTreeMap<String, String>,
    session_id: String,
    entity_id: Option<String>,
}
impl PartialEq for EditorSnapshot {
    fn eq(&self, other: &Self) -> bool {
        // Navigation is restored with an edit but does not add undo steps itself.
        self.workspace_json == other.workspace_json
            && self.date_drafts == other.date_drafts
            && self.time_drafts == other.time_drafts
    }
}

pub struct AtlasApp {
    workspace: Workspace,
    revision: i64,
    store: Store,
    data_dir: PathBuf,
    session: usize,
    entity: usize,
    tab: String,
    status: String,
    dirty: bool,
    date_text: BTreeMap<String, String>,
    time_text: BTreeMap<String, String>,
    active_date_text: String,
    active_time_text: String,
    import_text: String,
    feature_cache: Vec<Feature>,
    previous_json: String,
    saved_json: String,
    history: History<EditorSnapshot>,
    new_session_dataset: String,
    reset_focus: bool,
    records_unlinked: bool,
    temporal_date_id: Option<String>,
    focus_record: Option<String>,
    scroll_to_record: bool,
    new_date: workbench::NewDate,
    show_new_date: bool,
    map_view: crate::views::MapView,
    ecliptic_view: bool,
    live_time: bool,
    sky_cache_key: String,
    sky_cache: Option<Result<atlas_core::sky::SkySnapshot, String>>,
    features_key: String,
    data_tools: data_tools::ImportTools,
    city_lookup: crate::geocoding::Lookup,
    geo_target: Option<(String, String)>,
    open_city_search: bool,
    scope: atlas_core::filter::Scope,
    browse_search: String,
    browse_kind: String,
    browse_all: bool,
    entity_search: String,
    place_search: String,
    date_search: String,
    comparison_date_search: String,
    relation_search: String,
    only_date_hits: bool,
    focus_place: Option<String>,
    scroll_to_place: bool,
    #[cfg(debug_assertions)]
    preview_requested: bool,
}
impl AtlasApp {
    pub fn open_session(&mut self, name: &str) {
        if let Some(index) = self.workspace.sessions.iter().position(|s| s.name == name) {
            self.session = index;
            self.entity = 0;
            self.new_session_dataset = self.workspace.sessions[index].dataset_id.clone();
            self.sync_active();
            self.scope = atlas_core::filter::scope(
                &self.workspace.datasets[self.dataset_index()],
                &self.workspace.sessions[index],
            );
            self.history = History::new(self.snapshot(), 100);
        } else {
            self.status =
                format!("Session {name} not found; select a saved session in the sidebar");
        }
    }
    pub fn new(workspace: Workspace, revision: i64, store: Store, data_dir: PathBuf) -> Self {
        let city_lookup = crate::geocoding::Lookup::new(data_dir.clone());
        let date = workspace.sessions[0].active_date.to_string();
        let time = workspace.sessions[0]
            .active_time
            .format("%H:%M:%S%.f")
            .to_string();
        let previous_json = serde_json::to_string(&workspace).unwrap();
        let temporal_date_id = workspace.sessions[0].dates.focus.clone();
        let initial = EditorSnapshot {
            workspace_json: previous_json.clone(),
            date_drafts: BTreeMap::new(),
            time_drafts: BTreeMap::new(),
            session_id: workspace.sessions[0].id.clone(),
            entity_id: workspace
                .datasets
                .iter()
                .find(|d| d.id == workspace.sessions[0].dataset_id)
                .and_then(|d| d.entities.first())
                .map(|e| e.id.clone()),
        };
        let new_session_dataset = workspace.sessions[0].dataset_id.clone();
        let scope = atlas_core::filter::scope(
            workspace
                .datasets
                .iter()
                .find(|d| d.id == new_session_dataset)
                .unwrap(),
            &workspace.sessions[0],
        );
        Self {
            workspace,
            revision,
            store,
            data_dir,
            session: 0,
            entity: 0,
            tab: if cfg!(debug_assertions) {
                std::env::var("ATLAS_PREVIEW_TAB").unwrap_or_else(|_| "Overview".into())
            } else {
                "Overview".into()
            },
            status: "Ready · workspace saved locally".into(),
            dirty: false,
            date_text: BTreeMap::new(),
            time_text: BTreeMap::new(),
            active_date_text: date,
            active_time_text: time,
            import_text: String::new(),
            feature_cache: vec![],
            saved_json: previous_json.clone(),
            previous_json,
            history: History::new(initial, 100),
            new_session_dataset,
            reset_focus: false,
            records_unlinked: false,
            temporal_date_id,
            focus_record: None,
            scroll_to_record: false,
            new_date: workbench::NewDate::default(),
            show_new_date: false,
            map_view: crate::views::MapView {
                visible_places: Some(scope.places.clone()),
                globe: cfg!(debug_assertions)
                    && std::env::var("ATLAS_PREVIEW_VARIANT").as_deref() == Ok("globe"),
                ..Default::default()
            },
            ecliptic_view: cfg!(debug_assertions)
                && std::env::var("ATLAS_PREVIEW_VARIANT").as_deref() == Ok("ecliptic"),
            live_time: false,
            sky_cache_key: String::new(),
            sky_cache: None,
            features_key: String::new(),
            data_tools: data_tools::ImportTools::default(),
            city_lookup,
            geo_target: None,
            open_city_search: false,
            scope,
            browse_search: String::new(),
            browse_kind: "All records".into(),
            browse_all: false,
            entity_search: String::new(),
            place_search: String::new(),
            date_search: String::new(),
            comparison_date_search: String::new(),
            relation_search: String::new(),
            only_date_hits: false,
            focus_place: None,
            scroll_to_place: false,
            #[cfg(debug_assertions)]
            preview_requested: false,
        }
    }
    fn snapshot(&self) -> EditorSnapshot {
        let mut date_drafts = BTreeMap::new();
        let mut time_drafts = BTreeMap::new();
        for o in self.workspace.datasets.iter().flat_map(|d| &d.observations) {
            if let Some(text) = self
                .date_text
                .get(&o.id)
                .filter(|text| **text != o.date.to_string())
            {
                date_drafts.insert(o.id.clone(), text.clone());
            }
            let canonical = o
                .time
                .map(|t| t.format("%H:%M:%S%.f").to_string())
                .unwrap_or_default();
            if let Some(text) = self.time_text.get(&o.id).filter(|text| **text != canonical) {
                time_drafts.insert(o.id.clone(), text.clone());
            }
        }
        let s = &self.workspace.sessions[self.session];
        if self.active_date_text != s.active_date.to_string() {
            date_drafts.insert(s.id.clone(), self.active_date_text.clone());
        }
        if self.active_time_text != s.active_time.format("%H:%M:%S%.f").to_string() {
            time_drafts.insert(s.id.clone(), self.active_time_text.clone());
        }
        EditorSnapshot {
            workspace_json: serde_json::to_string(&self.workspace).unwrap(),
            date_drafts,
            time_drafts,
            session_id: s.id.clone(),
            entity_id: self.workspace.datasets[self.dataset_index()]
                .entities
                .get(self.entity)
                .map(|e| e.id.clone()),
        }
    }
    fn restore_snapshot(&mut self, snapshot: EditorSnapshot) {
        self.live_time = false;
        self.workspace = serde_json::from_str(&snapshot.workspace_json)
            .expect("Editor snapshots are serialized in memory");
        self.session = self
            .workspace
            .sessions
            .iter()
            .position(|s| s.id == snapshot.session_id)
            .unwrap_or(0);
        self.entity = self.workspace.datasets[self.dataset_index()]
            .entities
            .iter()
            .position(|e| Some(&e.id) == snapshot.entity_id.as_ref())
            .unwrap_or(0);
        self.date_text = snapshot.date_drafts;
        self.time_text = snapshot.time_drafts;
        self.sync_active();
        let sid = &self.workspace.sessions[self.session].id;
        if let Some(text) = self.date_text.remove(sid) {
            self.active_date_text = text;
        }
        if let Some(text) = self.time_text.remove(sid) {
            self.active_time_text = text;
        }
        self.previous_json.clear();
        self.reset_focus = true;
        self.dirty = snapshot.workspace_json != self.saved_json || self.input_valid().is_err();
    }
    fn undo(&mut self) {
        if let Some(snapshot) = self.history.undo() {
            self.restore_snapshot(snapshot);
            self.status = "Undid workspace edit · Redo is available".into();
        }
    }
    fn redo(&mut self) {
        if let Some(snapshot) = self.history.redo() {
            self.restore_snapshot(snapshot);
            self.status = "Redid workspace edit".into();
        }
    }
    fn prune_drafts(&mut self) {
        let ids = self
            .workspace
            .datasets
            .iter()
            .flat_map(|d| &d.observations)
            .map(|o| o.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        self.date_text.retain(|id, _| ids.contains(id.as_str()));
        self.time_text.retain(|id, _| ids.contains(id.as_str()));
    }
    fn remove_session(&mut self, sid: &str) {
        let active = self.workspace.sessions[self.session].id.clone();
        match self.workspace.remove_session(sid) {
            Ok(()) => {
                self.session = self
                    .workspace
                    .sessions
                    .iter()
                    .position(|s| s.id == active)
                    .unwrap_or_else(|| self.session.min(self.workspace.sessions.len() - 1));
                if active == sid {
                    self.entity = 0;
                    self.sync_active();
                } else {
                    self.feature_cache.clear();
                }
                self.reset_focus = true;
                self.status = "Removed session · dataset retained · Undo to recover".into();
            }
            Err(e) => self.status = e.to_string(),
        }
    }
    fn remove_entity(&mut self, eid: &str) {
        let did = self.workspace.sessions[self.session].dataset_id.clone();
        match self.workspace.remove_entity(&did, eid) {
            Ok(()) => {
                self.entity = 0;
                self.prune_drafts();
                self.feature_cache.clear();
                self.reset_focus = true;
                self.status = "Removed entity and its labeled dates · Undo to recover".into();
            }
            Err(e) => self.status = e.to_string(),
        }
    }
    fn remove_place(&mut self, pid: &str) {
        let did = self.workspace.sessions[self.session].dataset_id.clone();
        match self.workspace.remove_place(&did, pid) {
            Ok(()) => {
                self.feature_cache.clear();
                self.reset_focus = true;
                self.status =
                    "Removed place and cleared its links · dates retained · Undo to recover".into();
            }
            Err(e) => self.status = e.to_string(),
        }
    }
    fn restore_saved_revision(&mut self, revision: i64) {
        match self.store.load_revision(revision) {
            Ok(w) => {
                self.workspace = w;
                self.session = 0;
                self.entity = 0;
                self.date_text.clear();
                self.time_text.clear();
                self.sync_active();
                self.reset_focus = true;
                self.status=format!("Restored saved revision {revision} in memory · Save to commit, or Undo to return");
            }
            Err(e) => self.status = format!("Recovery failed: {e}"),
        }
    }
    fn dataset_index(&self) -> usize {
        let id = &self.workspace.sessions[self.session].dataset_id;
        self.workspace
            .datasets
            .iter()
            .position(|d| d.id == *id)
            .unwrap()
    }
    fn sync_active(&mut self) {
        let s = &self.workspace.sessions[self.session];
        self.active_date_text = s.active_date.to_string();
        self.active_time_text = s.active_time.format("%H:%M:%S%.f").to_string();
        self.temporal_date_id = s.dates.focus.clone();
        self.feature_cache.clear();
    }
    fn input_valid(&self) -> anyhow::Result<()> {
        temporal::parse_date(&self.active_date_text)?;
        NaiveTime::parse_from_str(&self.active_time_text, "%H:%M:%S%.f")?;
        for v in self.date_text.values() {
            temporal::parse_date(v)?;
        }
        for v in self.time_text.values().filter(|s| !s.trim().is_empty()) {
            NaiveTime::parse_from_str(v, "%H:%M:%S%.f")?;
        }
        self.workspace.validate()
    }
    fn save_data(&mut self) {
        let result = self
            .input_valid()
            .and_then(|_| self.store.save(&self.workspace, self.revision));
        match result {
            Ok(r) => {
                self.revision = r;
                self.dirty = false;
                self.previous_json = serde_json::to_string(&self.workspace).unwrap();
                self.saved_json = self.previous_json.clone();
                self.feature_cache.clear();
                self.status = format!("Saved revision {r} · {}", self.data_dir.display());
            }
            Err(e) => self.status = format!("Save blocked: {e:#}"),
        }
    }
    fn export(&mut self) {
        if let Err(e) = self.input_valid() {
            self.status = format!("Export blocked: {e}");
            return;
        }
        let path = self.data_dir.join(format!(
            "workspace-{}-{}.json",
            Utc::now().format("%Y%m%d-%H%M%S"),
            &id()[..8]
        ));
        match serde_json::to_string_pretty(&self.workspace)
            .map_err(anyhow::Error::from)
            .and_then(|s| crate::write_new(&path, &s))
        {
            Ok(()) => self.status = format!("Exported {}", path.display()),
            Err(e) => self.status = format!("Export failed: {e}"),
        }
    }
    fn top(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading(RichText::new("PATTERN ATLAS").color(ACCENT));
                ui.label(
                    RichText::new(concat!(
                        "native research workbench · ",
                        env!("CARGO_PKG_VERSION")
                    ))
                    .weak(),
                );
                ui.separator();
                if ui
                    .add_enabled(self.history.can_undo(), egui::Button::new("Undo"))
                    .on_hover_text("Undo workspace edit · Ctrl+Z")
                    .clicked()
                {
                    self.undo();
                }
                if ui
                    .add_enabled(self.history.can_redo(), egui::Button::new("Redo"))
                    .on_hover_text("Redo workspace edit · Ctrl+Y / Ctrl+Shift+Z")
                    .clicked()
                {
                    self.redo();
                }
                if ui.button("Save workspace").clicked() {
                    self.save_data();
                }
                if ui.button("Export JSON").clicked() {
                    self.export();
                }
                if ui.button("Discard edits / Reload saved").clicked() {
                    match self.store.load() {
                        Ok(Some((w, r))) => {
                            self.workspace = w;
                            self.revision = r;
                            self.session = 0;
                            self.entity = 0;
                            self.date_text.clear();
                            self.time_text.clear();
                            self.sync_active();
                            self.previous_json = serde_json::to_string(&self.workspace).unwrap();
                            self.saved_json = self.previous_json.clone();
                            self.reset_focus = true;
                            self.dirty = false;
                            self.status = "Reloaded saved workspace".into();
                        }
                        Ok(None) => self.status = "No saved workspace".into(),
                        Err(e) => self.status = format!("Reload failed: {e}"),
                    }
                }
                ui.label(if self.dirty {
                    "● Unsaved edits"
                } else {
                    "Saved"
                });
            });
            ui.horizontal_wrapped(|ui| {
                let was_live = self.live_time;
                let mut live = self.live_time;
                if ui.checkbox(&mut live, "Live now").changed() {
                    if was_live && !live {
                        self.pause_live();
                    }
                    self.live_time = live;
                }
                let effective = self.effective_session();
                let s = &mut self.workspace.sessions[self.session];
                ui.label("Find number");
                ui.add(
                    egui::TextEdit::singleline(&mut s.query)
                        .desired_width(95.)
                        .hint_text("e.g. 56"),
                );
                egui::ComboBox::from_id_salt("rounding")
                    .selected_text(format!("{:?}", s.settings.rounding))
                    .show_ui(ui, |ui| {
                        for v in [Rounding::Raw, Rounding::Nearest, Rounding::Floor] {
                            ui.selectable_value(&mut s.settings.rounding, v, format!("{v:?}"));
                        }
                    });
                ui.label("Decimals");
                ui.add(egui::DragValue::new(&mut s.settings.decimals).range(0..=6));
                ui.separator();
                ui.label("Active date");
                if self.live_time {
                    ui.label(effective.active_date.to_string());
                } else {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.active_date_text).desired_width(95.),
                    );
                    if let Ok(d) = temporal::parse_date(&self.active_date_text) {
                        s.active_date = d;
                        if response.changed() {
                            s.active_observation = None;
                        }
                    }
                }
                ui.label("Time");
                if self.live_time {
                    ui.label(effective.active_time.to_string());
                } else {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.active_time_text).desired_width(90.),
                    );
                    if let Ok(t) = NaiveTime::parse_from_str(&self.active_time_text, "%H:%M:%S%.f")
                    {
                        s.active_time = t;
                        if response.changed() {
                            s.active_observation = None;
                        }
                    }
                }
                ui.add(egui::TextEdit::singleline(&mut s.timezone).desired_width(135.));
                crate::timezones::presets(ui, &mut s.timezone, "active_timezone_presets");
                ui.checkbox(&mut s.time_assumed, "Assumed time");
                if ui.button("Now UTC").clicked() {
                    let now = Utc::now();
                    s.active_date = now.date_naive();
                    s.active_time = now.time().with_nanosecond(0).unwrap();
                    s.timezone = "UTC".into();
                    s.time_assumed = false;
                    s.active_observation = None;
                    self.live_time = false;
                    self.sync_active();
                }
            });
            ui.horizontal(|ui| {
                for tab in [
                    "Overview",
                    "Records",
                    "Browse",
                    "Gematria",
                    "Dates",
                    "Earth",
                    "Sky",
                    "Comparison",
                    "Import data",
                ] {
                    ui.selectable_value(&mut self.tab, tab.into(), tab);
                }
            });
        });
    }
    fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigator").resizable(true).default_width(235.).show(ctx,|ui| {
            egui::ScrollArea::vertical().show(ui,|ui| {
            ui.heading("Sessions"); let mut next=None; let mut remove=None;
            for (i,s) in self.workspace.sessions.iter().enumerate() {
                ui.push_id(&s.id,|ui|{ui.horizontal(|ui|{
                    if ui.selectable_label(i==self.session,&s.name).clicked(){next=Some(s.id.clone());}
                    if ui.add_enabled(self.workspace.sessions.len()>1,egui::Button::new("×")).on_hover_text(if self.workspace.sessions.len()>1 {"Remove session; dataset retained. Undo to recover."} else {"Keep one session. Create another to remove this one."}).clicked(){remove=Some(s.id.clone());}
                });});
            }
            if let Some(id)=remove {self.remove_session(&id);}
            if let Some(id)=next {if let Some(i)=self.workspace.sessions.iter().position(|s|s.id==id){self.session=i;self.entity=0;self.sync_active();}}
            if !self.workspace.datasets.iter().any(|d|d.id==self.new_session_dataset){self.new_session_dataset=self.workspace.sessions[self.session].dataset_id.clone();}
            ui.label("New session for dataset");
            let label=self.workspace.datasets.iter().find(|d|d.id==self.new_session_dataset).map(|d|d.name.clone()).unwrap_or_default();
            egui::ComboBox::from_id_salt("session_dataset").selected_text(label).show_ui(ui,|ui|{for d in &self.workspace.datasets{ui.selectable_value(&mut self.new_session_dataset,d.id.clone(),&d.name);}});
            if ui.button("+ Session on selected dataset").clicked() {
                let current=&self.workspace.sessions[self.session];
                let mut s=if current.dataset_id==self.new_session_dataset{current.clone()}else{Session::new(&self.new_session_dataset,current.active_date)};
                s.id=id();s.name=format!("Session {}",self.workspace.sessions.len()+1);
                self.workspace.sessions.push(s);self.session=self.workspace.sessions.len()-1;self.entity=0;self.sync_active();
            }
            if ui.button("+ Empty dataset & session").clicked() {
                let d=Dataset::empty(&format!("Dataset {}",self.workspace.datasets.len()+1));
                let s=Session::new(&d.id,self.workspace.sessions[self.session].active_date);
                self.new_session_dataset=d.id.clone();self.workspace.datasets.push(d); self.workspace.sessions.push(s); self.session=self.workspace.sessions.len()-1; self.entity=0; self.sync_active();
            }
            let dindex=self.dataset_index(); let s=&mut self.workspace.sessions[self.session];
            ui.separator(); ui.label("Session name"); ui.text_edit_singleline(&mut s.name);
            ui.label("Session context"); egui::ScrollArea::vertical().id_salt("session_context_editor").max_height(75.).show(ui,|ui| {ui.add(egui::TextEdit::multiline(&mut s.context).desired_rows(3));});
            ui.separator(); ui.heading("Dataset"); ui.text_edit_singleline(&mut self.workspace.datasets[dindex].name);
            ui.label("Dataset context"); egui::ScrollArea::vertical().id_salt("dataset_context_editor").max_height(85.).show(ui,|ui| {ui.add(egui::TextEdit::multiline(&mut self.workspace.datasets[dindex].context).desired_rows(3));});
            let d=&self.workspace.datasets[dindex]; ui.separator(); ui.heading("Entity scope");
            if ui.selectable_label(s.selected_entities.is_empty(),"All entities").clicked() { s.selected_entities.clear(); }
            egui::ScrollArea::vertical().id_salt("scope_entity_list").max_height(160.).show(ui,|ui| {for e in &d.entities {
                if (self.scope.entities.contains(&e.id) || s.selected_entities.contains(&e.id)) && ui.selectable_label(s.selected_entities.contains(&e.id),&e.name).clicked() {
                    s.selected_entities=vec![e.id.clone()];
                }
            }});
            ui.separator(); ui.heading("Active observer");
            let current=d.places.iter().find(|p|Some(&p.id)==s.active_place.as_ref()).map_or("No location",|p|p.name.as_str());
            egui::ComboBox::from_id_salt("active_place").selected_text(current).show_ui(ui,|ui| {
                ui.selectable_value(&mut s.active_place,None,"No location"); for p in &d.places { ui.selectable_value(&mut s.active_place,Some(p.id.clone()),&p.name); }
            });
            if let Some(p)=d.places.iter().find(|p|Some(&p.id)==s.active_place.as_ref()) {
                ui.label(format!("Latitude {:.4}°, longitude {:.4}°",p.latitude,p.longitude));
                if ui.button("Use observer timezone").clicked() { s.timezone=p.timezone.clone(); }
            }
            ui.separator(); ui.small("Coordinates and source dates stay in the dataset. Display settings stay in this session.");
            });
        });
    }
    fn overview(&mut self, ui: &mut egui::Ui) {
        let dindex = self.dataset_index();
        let d = &self.workspace.datasets[dindex];
        let s = &self.workspace.sessions[self.session];
        ui.heading("A connected dataset. Several ways to explore it.");
        ui.label("Link named entities to labeled dates and places; search their numerical properties across sections.");
        ui.add_space(14.);
        ui.horizontal(|ui| {
            for (count, label) in [
                (d.entities.len(), "entities"),
                (d.observations.len(), "labeled dates"),
                (d.places.len(), "places"),
                (self.feature_cache.len(), "numeric features"),
            ] {
                ui.group(|ui| {
                    ui.label(RichText::new(count.to_string()).size(30.).color(ACCENT));
                    ui.label(label);
                });
            }
        });
        ui.add_space(14.);
        ui.weak(self.location_scope_summary());
        ui.weak(self.date_scope_summary());
        ui.heading("Start here");
        ui.label(
            "1. Create an empty dataset, or replace the illustrative starter records in Records.",
        );
        ui.label(
            "2. Add context, sources, labeled dates and coordinates. Dates can stay date-only.",
        );
        ui.label(
            "3. Choose ciphers and units. Search a number, then inspect the Comparison matrix.",
        );
        ui.label("4. Save the workspace. Export JSON to share it with another interface.");
        ui.add_space(12.);
        ui.heading(format!(
            "Number search: {}",
            if s.query.is_empty() { "none" } else { &s.query }
        ));
        let hits = self
            .feature_cache
            .iter()
            .filter(|f| f.matches(&s.query, &s.settings))
            .cloned()
            .collect::<Vec<_>>();
        feature_table(ui, &hits, &s.settings, &s.query, "overview_hits");
        ui.separator();
        ui.label("Dates, maps and sky views share the active time and observer. Date-only sources keep their unknown time; viewing assumptions stay in the session.");
        ui.collapsing("Saved history · recover earlier work",|ui| {
            ui.label("Last 50 saved workspaces. Timestamps are UTC. Restore loads a revision into memory; Save commits it as a new revision. Undo can return to your current edits.");
            match self.store.revisions(){
                Ok(revisions)=>{for saved in revisions{ui.horizontal(|ui|{
                    ui.label(format!("Revision {} · {} UTC",saved.revision,saved.saved_at));
                    if ui.button("Restore in memory").clicked(){self.restore_saved_revision(saved.revision);}
                });}},
                Err(e)=>{ui.colored_label(HIT,format!("Cannot list saved history: {e}"));},
            }
        });
        ui.collapsing("Import a Pattern Atlas workspace",|ui| {
            ui.label("Paste a full workspace JSON export. Loading replaces the in-memory workspace; save to commit it.");
            ui.add(egui::TextEdit::multiline(&mut self.import_text).id(egui::Id::new("workspace_import_draft")).desired_rows(6).desired_width(f32::INFINITY));
            if ui.button("Load pasted JSON as workspace (unsaved)").clicked() {
                let result=serde_json::from_str::<Workspace>(&self.import_text).map_err(anyhow::Error::from).and_then(|mut w|{w.upgrade()?;Ok(w)});
                match result { Ok(w)=>{self.workspace=w;self.session=0;self.entity=0;self.date_text.clear();self.time_text.clear();self.sync_active();self.status="Imported into memory. Save to commit; export first if you need a separate copy.".into();}, Err(e)=>self.status=format!("Import rejected: {e:#}") }
            }
        });
    }
}
impl AtlasApp {
    fn render(&mut self, ctx: &egui::Context) {
        let before_view = (
            self.tab.clone(),
            self.show_new_date,
            self.temporal_date_id.clone(),
            self.status.clone(),
            self.session,
            self.entity,
            self.records_unlinked,
            self.focus_place.clone(),
        );
        // Pasted import text is a scratch buffer until loaded; retain its local text undo.
        let import_focused = ctx.memory(|m| m.focused()).is_some_and(|id| {
            [
                "workspace_import_draft",
                "new_date_label",
                "new_date_value",
                "new_date_time",
                "new_date_timezone",
                "bulk_import_draft",
                "bulk_import_path",
                "ai_import_scope",
                "city_lookup_city",
                "city_lookup_state",
                "city_lookup_country",
                "browse_record_search",
                "editor_entity_search",
                "editor_place_search",
                "date_picker_search",
            ]
            .iter()
            .any(|v| id == egui::Id::new(v))
                || ["focus_date", "comparison_date", "sky_saved_date"]
                    .iter()
                    .any(|salt| id == egui::Id::new((*salt, "search")))
                || id == egui::Id::new("relation_search")
        });
        let redo = !import_focused
            && ctx.input_mut(|i| {
                i.consume_key(
                    egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                    egui::Key::Z,
                ) || i.consume_key(egui::Modifiers::COMMAND, egui::Key::Y)
            });
        let undo = !import_focused
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Z));
        if redo {
            self.redo();
        } else if undo {
            self.undo();
        }
        if self.reset_focus {
            ctx.memory_mut(|m| {
                if let Some(id) = m.focused() {
                    m.surrender_focus(id);
                }
            });
            self.reset_focus = false;
        }
        self.top(ctx);
        self.sidebar(ctx);
        let json = serde_json::to_string(&self.workspace).unwrap_or_default();
        let effective = self.effective_session();
        let key = format!("{}:{}", json, serde_json::to_string(&effective).unwrap());
        if key != self.features_key || self.feature_cache.is_empty() {
            self.features_key = key;
            self.previous_json = json;
            self.scope = atlas_core::filter::scope(
                &self.workspace.datasets[self.dataset_index()],
                &effective,
            );
            self.map_view.visible_places = Some(self.scope.places.clone());
            self.feature_cache = analysis::features_in_scope(
                &self.workspace.datasets[self.dataset_index()],
                &effective,
                &self.scope,
            );
        }
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status);
            if let Err(e) = self.input_valid() {
                ui.colored_label(HIT, format!("Input needs attention: {e}"));
            }
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                self.filters(ui);
                match self.tab.as_str() {
                    "Overview" => self.overview(ui),
                    "Records" => self.records(ui),
                    "Browse" => self.browse(ui),
                    "Gematria" => self.gematria(ui),
                    "Dates" => self.dates(ui),
                    "Earth" => self.earth(ui),
                    "Sky" => self.sky(ui),
                    "Import data" => self.import_data(ui),
                    _ => self.comparisons(ui),
                }
            });
        });
        let after = serde_json::to_string(&self.workspace).unwrap_or_default();
        let snapshot = self.snapshot();
        self.dirty = after != self.saved_json || self.input_valid().is_err();
        let (typing, time) = ctx.input(|i| {
            let typing = !i.pointer.any_pressed()
                && !i.pointer.any_released()
                && i.events.iter().any(|e| {
                    matches!(
                        e,
                        egui::Event::Text(_)
                            | egui::Event::Paste(_)
                            | egui::Event::Cut
                            | egui::Event::Key {
                                key: egui::Key::Backspace | egui::Key::Delete,
                                pressed: true,
                                ..
                            }
                    )
                });
            (typing, i.time)
        });
        let group = if typing {
            ctx.memory(|m| m.focused())
                .map(|id| format!("{}:{id:?}", self.workspace.sessions[self.session].id))
        } else {
            None
        };
        self.history.record(snapshot, group, time);
        if self.live_time {
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }
        if after != self.previous_json {
            ctx.request_repaint();
        }
        if before_view
            != (
                self.tab.clone(),
                self.show_new_date,
                self.temporal_date_id.clone(),
                self.status.clone(),
                self.session,
                self.entity,
                self.records_unlinked,
                self.focus_place.clone(),
            )
        {
            ctx.request_repaint();
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty {
            self.save_data();
            if self.dirty {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            }
        }
        #[cfg(debug_assertions)]
        if let Ok(path) = std::env::var("ATLAS_PREVIEW_PATH") {
            if let Some(screenshot) = ctx.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            }) {
                let bytes = screenshot
                    .pixels
                    .iter()
                    .flat_map(|c| c.to_array())
                    .collect::<Vec<_>>();
                match image::save_buffer(
                    &path,
                    &bytes,
                    screenshot.width() as u32,
                    screenshot.height() as u32,
                    image::ColorType::Rgba8,
                ) {
                    Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                    Err(e) => self.status = format!("Preview capture failed: {e}"),
                }
            } else if !self.preview_requested && ctx.cumulative_pass_nr() >= 3 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                self.preview_requested = true;
            } else {
                ctx.request_repaint();
            }
        }
    }
}
impl eframe::App for AtlasApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.render(ctx);
    }
}
fn feature_table(ui: &mut egui::Ui, fs: &[Feature], settings: &Settings, query: &str, salt: &str) {
    if fs.is_empty() {
        ui.label("No properties in this selection.");
        return;
    }
    egui::Grid::new(salt)
        .striped(true)
        .min_col_width(75.)
        .show(ui, |ui| {
            for label in ["Record", "Domain", "Calculation", "Value", "Unit"] {
                ui.strong(label);
            }
            ui.end_row();
            for f in fs
                .iter()
                .filter(|f| f.matches(query, settings))
                .chain(fs.iter().filter(|f| !f.matches(query, settings)))
                .take(500)
            {
                let color = if f.matches(query, settings) {
                    HIT
                } else {
                    Color32::WHITE
                };
                ui.label(RichText::new(&f.owner).color(color));
                ui.label(&f.domain);
                ui.label(&f.calculation);
                ui.label(RichText::new(f.displayed(settings)).color(color).strong())
                    .on_hover_text(format!("Raw: {}\n{}", f.raw, f.explanation));
                ui.label(&f.unit);
                ui.end_row();
            }
        });
    if fs.len() > 500 {
        ui.label(format!(
            "Showing 500 of {} properties. CLI analysis exports all properties.",
            fs.len()
        ));
    }
}
