use super::*;
use atlas_core::bulk::ImportFile;

#[derive(Default)]
pub(super) struct ImportTools {
    pub text: String,
    pub path: String,
    pub scope: String,
    pub append: bool,
    pub preview: Option<ImportFile>,
    pub message: String,
}
pub(crate) fn attributes(ui: &mut egui::Ui, values: &BTreeMap<String, serde_json::Value>) {
    if values.is_empty() {
        return;
    }
    ui.collapsing("Recorded attributes", |ui| {
        ui.weak("Numeric fields participate in number search and comparisons. Values retain their original field names.");
        for (key, value) in values { ui.horizontal_wrapped(|ui| { ui.strong(key); ui.label(value.to_string()); }); }
    });
}
impl AtlasApp {
    pub(super) fn import_data(&mut self, ui: &mut egui::Ui) {
        ui.heading("Add a dataset · words, teams, people, places and dates");
        ui.weak("Imports add records. Full workspace backups use the separate import on Overview.");
        ui.group(|ui| {
            ui.strong("Ask an AI to prepare data");
            ui.weak("Describe the scope, season or sources to collect. Copy the prompt into your AI chat, then paste its JSON below.");
            ui.add(egui::TextEdit::singleline(&mut self.data_tools.scope).id(egui::Id::new("ai_import_scope")).hint_text("e.g. NFL teams, current home cities and sourced founding dates").desired_width(660.));
            if ui.button("Copy AI import prompt").clicked() {
                let scope = if self.data_tools.scope.trim().is_empty() { "Ask me which dataset and time period to collect before researching." } else { &self.data_tools.scope };
                ui.ctx().copy_text(include_str!("../../../docs/AI-IMPORT-PROMPT.md").replace("[DATASET REQUEST]", scope));
                self.status = "AI prompt copied · paste it into an AI chat".into();
            }
            if ui.small_button("Load example JSON").clicked() {
                self.data_tools.text = include_str!("../../../examples/import-demo.json").into(); self.data_tools.preview = None;
            }
        });
        ui.add_space(8.);
        ui.horizontal_wrapped(|ui| {
            ui.label("JSON file path");
            ui.add(
                egui::TextEdit::singleline(&mut self.data_tools.path)
                    .id(egui::Id::new("bulk_import_path"))
                    .desired_width(510.),
            );
            if ui.button("Load JSON file").clicked() {
                match std::fs::read_to_string(self.data_tools.path.trim().trim_matches('"')) {
                    Ok(text) => {
                        self.data_tools.text = text;
                        self.data_tools.preview = None;
                        self.data_tools.message = "Loaded file · Preview before importing".into();
                    }
                    Err(e) => self.data_tools.message = format!("Could not read file: {e}"),
                }
            }
        });
        ui.label("Paste JSON");
        let changed = ui
            .add(
                egui::TextEdit::multiline(&mut self.data_tools.text)
                    .id(egui::Id::new("bulk_import_draft"))
                    .desired_rows(9)
                    .desired_width(f32::INFINITY)
                    .code_editor(),
            )
            .changed();
        if changed {
            self.data_tools.preview = None;
            self.data_tools.message = "JSON changed · preview again".into();
        }
        ui.horizontal_wrapped(|ui| {
            if ui.button("Preview import").clicked() {
                match ImportFile::parse(&self.data_tools.text) {
                    Ok(file) => {
                        self.data_tools.message =
                            "Valid JSON · review counts and samples below".into();
                        self.data_tools.preview = Some(file);
                    }
                    Err(e) => {
                        self.data_tools.preview = None;
                        self.data_tools.message = format!("Import needs attention: {e:#}");
                    }
                }
            }
            ui.checkbox(&mut self.data_tools.append, "Append to current dataset");
        });
        ui.label(&self.data_tools.message);
        if let Some(file) = self.data_tools.preview.clone() {
            let d = &file.dataset;
            let repeated = self
                .workspace
                .datasets
                .iter()
                .any(|d| d.import_batches.contains(&file.batch_id));
            ui.group(|ui| {
                ui.strong(format!("{} · batch {}", d.name, file.batch_id));
                ui.label(format!("{} entities · {} places · {} dates", d.entities.len(), d.places.len(), d.observations.len()));
                if !d.context.is_empty() { ui.label(&d.context); }
                for e in d.entities.iter().take(4) { ui.label(format!("Entity: {} ({}) · {}", e.name, e.kind, e.source)); }
                for p in d.places.iter().take(4) { ui.label(format!("Place: {} · {}, {} · {}", p.name, p.latitude, p.longitude, p.timezone)); }
                for o in d.observations.iter().take(4) { ui.label(format!("Date: {} · {} · {} · {}", o.label, o.date, o.time.as_deref().unwrap_or("time unknown"), o.timezone)); }
                if self.data_tools.append {
                    let current = &self.workspace.datasets[self.dataset_index()];
                    ui.label(format!("Destination: {}. Existing records are retained.", current.name));
                    let collisions = d.entities.iter().filter(|e| current.entities.iter().any(|old| old.name.eq_ignore_ascii_case(&e.name))).count();
                    if collisions > 0 { ui.colored_label(HIT, format!("{collisions} entity names already exist. Appending creates separate records; it does not merge them.")); }
                } else { ui.label("Destination: a new dataset and research session"); }
                if repeated { ui.colored_label(HIT, "This batch has already been imported. Undo its import before retrying."); }
                if d.observations.len() > 500 { ui.colored_label(HIT, "Large date collections generate many pair comparisons. Start with a smaller research scope."); }
            });
            if ui
                .add_enabled(!repeated, egui::Button::new("Import reviewed data"))
                .clicked()
            {
                let target = self
                    .data_tools
                    .append
                    .then(|| self.workspace.datasets[self.dataset_index()].id.clone());
                match file.apply(&mut self.workspace, target.as_deref()) {
                    Ok(id) => {
                        if target.is_none() {
                            self.session = self.workspace.sessions.len() - 1;
                            self.entity = 0;
                            self.records_unlinked = false;
                            self.temporal_date_id = None;
                            self.new_session_dataset = id;
                            self.sync_active();
                        }
                        self.status = format!(
                            "Imported {} · Undo recovers the previous workspace · Save to keep",
                            d.name
                        );
                        self.data_tools.message =
                            "Imported successfully. Changes are undoable.".into();
                        self.data_tools.preview = None;
                        ui.ctx().request_repaint();
                    }
                    Err(e) => self.data_tools.message = format!("Import rejected: {e:#}"),
                }
            }
        }
    }
    pub(super) fn city_search(&mut self, ui: &mut egui::Ui) {
        self.city_lookup.poll();
        let di = self.dataset_index();
        let open = std::mem::take(&mut self.open_city_search);
        if open {
            ui.scroll_to_cursor(Some(egui::Align::Min));
        }
        egui::CollapsingHeader::new("City / state lookup").id_salt("city_search").open(open.then_some(true)).show(ui, |ui| {
            ui.weak("Search sends the city/state/country to Open-Meteo. Select a result to add it or update a place.");
            ui.horizontal_wrapped(|ui| {
                ui.label("City"); ui.add(egui::TextEdit::singleline(&mut self.city_lookup.city).id(egui::Id::new("city_lookup_city")).desired_width(180.));
                ui.label("State / region"); ui.add(egui::TextEdit::singleline(&mut self.city_lookup.state).id(egui::Id::new("city_lookup_state")).desired_width(160.).hint_text("Illinois or IL"));
                ui.label("Country code"); ui.add(egui::TextEdit::singleline(&mut self.city_lookup.country).id(egui::Id::new("city_lookup_country")).desired_width(45.));
                if ui.add_enabled(!self.city_lookup.pending(), egui::Button::new("Search city")).clicked() { self.city_lookup.search(ui.ctx().clone()); }
            });
            if !self.city_lookup.message.is_empty() { ui.label(&self.city_lookup.message); }
            let target = self.geo_target.as_ref().filter(|(dataset, _)| *dataset == self.workspace.datasets[di].id).and_then(|(_, id)| self.workspace.datasets[di].places.iter().find(|p| p.id == *id)).map(|p| (p.id.clone(), p.name.clone()));
            for c in self.city_lookup.results.clone() {
                ui.group(|ui| {
                    ui.strong(c.label()); ui.label(format!("{}, {} · {} · elevation {} m", c.latitude, c.longitude, c.timezone, c.elevation.unwrap_or(0.)));
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Add as new place").clicked() {
                            match c.place() { Ok(p) => { self.workspace.datasets[di].places.push(p); self.status = "Added city · select it as the observer in the sidebar".into(); }, Err(e) => self.status = e.to_string() }
                        }
                        if let Some((id, name)) = &target {
                            if ui.button(format!("Apply to {name}")).clicked() {
                                if let Ok(found) = c.place() {
                                    let p = self.workspace.datasets[di].places.iter_mut().find(|p| p.id == *id).unwrap();
                                    p.latitude = found.latitude; p.longitude = found.longitude; p.elevation_m = found.elevation_m; p.timezone = found.timezone;
                                    if !p.source.is_empty() { p.source.push('\n'); } p.source.push_str(&found.source);
                                    if !p.context.is_empty() { p.context.push('\n'); } p.context.push_str(&found.context);
                                    self.status = format!("Updated coordinates and timezone for {name} · Undo to recover");
                                }
                            }
                        }
                    });
                });
            }
            ui.horizontal_wrapped(|ui| {
                ui.hyperlink_to("Open-Meteo / GeoNames", "https://open-meteo.com/en/docs/geocoding-api");
                ui.hyperlink_to("CC BY 4.0", "https://creativecommons.org/licenses/by/4.0/");
                ui.weak("Free lookup is for personal / noncommercial use. City coordinates are approximate.");
            });
        });
        if self.city_lookup.pending() {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}
