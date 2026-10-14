use super::*;
use crate::{observation_editor, views};
use atlas_core::sky;
use chrono::Duration;

pub(super) struct NewDate {
    pub label: String,
    pub date: String,
    pub time: String,
    pub timezone: String,
    pub entity_id: Option<String>,
    pub place_id: Option<String>,
}
impl Default for NewDate {
    fn default() -> Self {
        Self {
            label: String::new(),
            date: String::new(),
            time: String::new(),
            timezone: "UTC".into(),
            entity_id: None,
            place_id: None,
        }
    }
}

impl AtlasApp {
    pub(super) fn effective_session(&self) -> Session {
        let mut s = self.workspace.sessions[self.session].clone();
        if self.live_time {
            if let Ok(tz) = s.timezone.parse::<chrono_tz::Tz>() {
                let local = Utc::now().with_timezone(&tz);
                s.active_date = local.date_naive();
                s.active_time = local.time().with_nanosecond(0).unwrap();
                s.time_assumed = false;
                s.active_observation = None;
            }
        }
        s
    }
    pub(super) fn pause_live(&mut self) {
        let effective = self.effective_session();
        self.workspace.sessions[self.session] = effective;
        self.live_time = false;
        self.sync_active();
    }
    fn refresh_sky(&mut self, s: &Session) {
        let d = &self.workspace.datasets[self.dataset_index()];
        let place = d
            .places
            .iter()
            .find(|p| Some(&p.id) == s.active_place.as_ref());
        let key = format!(
            "{}:{}:{}:{}:{place:?}",
            s.active_date, s.active_time, s.timezone, s.dataset_id
        );
        if self.sky_cache_key != key {
            self.sky_cache_key = key;
            self.sky_cache = Some(sky::for_session(d, s).map_err(|e| e.to_string()));
        }
    }
    pub(super) fn activate_observation(&mut self, o: &Observation) {
        let d = &self.workspace.datasets[self.dataset_index()];
        let zone = if o.time.is_none() {
            d.places
                .iter()
                .find(|p| Some(&p.id) == o.place_id.as_ref())
                .map_or(o.timezone.clone(), |p| p.timezone.clone())
        } else {
            o.timezone.clone()
        };
        let time = o
            .time
            .unwrap_or_else(|| NaiveTime::from_hms_opt(12, 0, 0).unwrap());
        if let Err(e) = temporal::utc_instant(o.date, time, &zone) {
            self.status = format!("Cannot activate date: {e}");
            return;
        }
        self.live_time = false;
        self.temporal_date_id = Some(o.id.clone());
        let s = &mut self.workspace.sessions[self.session];
        s.dates.focus = Some(o.id.clone());
        s.active_date = o.date;
        s.active_time = time;
        s.timezone = zone;
        s.time_assumed = o.time.is_none();
        s.active_observation = Some(o.id.clone());
        if o.place_id.is_some() {
            s.active_place = o.place_id.clone();
        }
        self.sync_active();
    }
    pub(super) fn remove_date(&mut self, id: &str) {
        let di = self.dataset_index();
        self.workspace.datasets[di]
            .observations
            .retain(|o| o.id != id);
        let removed = std::iter::once(id.to_string()).collect();
        for s in &mut self.workspace.sessions {
            s.dates.clear_removed(&removed);
            if s.active_observation.as_deref() == Some(id) {
                s.active_observation = None;
            }
        }
        self.date_text.remove(id);
        self.time_text.remove(id);
        self.reset_focus = true;
        self.status = "Removed labeled date · Undo to recover".into();
    }
    pub(super) fn open_record(&mut self, o: &Observation) {
        self.records_unlinked = o.entity_id.is_none();
        self.entity = self.workspace.datasets[self.dataset_index()]
            .entities
            .iter()
            .position(|e| Some(&e.id) == o.entity_id.as_ref())
            .unwrap_or(0);
        self.focus_record = Some(o.id.clone());
        self.scroll_to_record = true;
        self.tab = "Records".into();
    }
    pub(super) fn add_new_date(&mut self) -> anyhow::Result<String> {
        let date = temporal::parse_date(&self.new_date.date)?;
        let time = if self.new_date.time.trim().is_empty() {
            None
        } else {
            Some(NaiveTime::parse_from_str(
                &self.new_date.time,
                "%H:%M:%S%.f",
            )?)
        };
        self.new_date.timezone.parse::<chrono_tz::Tz>()?;
        if let Some(t) = time {
            temporal::utc_instant(date, t, &self.new_date.timezone)?;
        }
        let dindex = self.dataset_index();
        let d = &self.workspace.datasets[dindex];
        anyhow::ensure!(
            self.new_date
                .entity_id
                .as_ref()
                .is_none_or(|id| d.entities.iter().any(|e| e.id == *id)),
            "Choose an entity in this dataset"
        );
        anyhow::ensure!(
            self.new_date
                .place_id
                .as_ref()
                .is_none_or(|id| d.places.iter().any(|p| p.id == *id)),
            "Choose a location in this dataset"
        );
        let mut o = Observation::unlinked(date);
        o.entity_id = self.new_date.entity_id.clone();
        o.place_id = self.new_date.place_id.clone();
        o.time = time;
        o.timezone = self.new_date.timezone.clone();
        o.label = if self.new_date.label.trim().is_empty() {
            format!("Date {date}")
        } else {
            self.new_date.label.clone()
        };
        let id = o.id.clone();
        self.workspace.datasets[dindex].observations.push(o.clone());
        self.activate_observation(&o);
        self.show_new_date = false;
        self.status =
            "Added date to dataset · edit its links here or open Details in Records".into();
        Ok(id)
    }
    fn new_date_editor(&mut self, ui: &mut egui::Ui) {
        let d = &self.workspace.datasets[self.dataset_index()];
        ui.group(|ui| {
            ui.heading("New dataset date");
            ui.weak("Entity and location links are optional. Leave time blank when it is unknown.");
            ui.horizontal_wrapped(|ui| {
                ui.label("Label");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_date.label)
                        .id(egui::Id::new("new_date_label"))
                        .desired_width(230.),
                );
                ui.label("Date");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_date.date)
                        .id(egui::Id::new("new_date_value"))
                        .desired_width(110.),
                );
                ui.label("Time");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_date.time)
                        .id(egui::Id::new("new_date_time"))
                        .desired_width(100.)
                        .hint_text("unknown"),
                );
                ui.label("Timezone");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_date.timezone)
                        .id(egui::Id::new("new_date_timezone"))
                        .desired_width(160.),
                );
                crate::timezones::presets(ui, &mut self.new_date.timezone, "new_date_zone_presets");
            });
            ui.horizontal_wrapped(|ui| {
                let entity = d
                    .entities
                    .iter()
                    .find(|e| Some(&e.id) == self.new_date.entity_id.as_ref())
                    .map_or("Unlinked date", |e| e.name.as_str());
                egui::ComboBox::from_id_salt("new_date_entity")
                    .selected_text(entity)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.new_date.entity_id, None, "Unlinked date");
                        for e in &d.entities {
                            ui.selectable_value(
                                &mut self.new_date.entity_id,
                                Some(e.id.clone()),
                                &e.name,
                            );
                        }
                    });
                let place = d
                    .places
                    .iter()
                    .find(|p| Some(&p.id) == self.new_date.place_id.as_ref())
                    .map_or("No location", |p| p.name.as_str());
                egui::ComboBox::from_id_salt("new_date_place")
                    .selected_text(place)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.new_date.place_id, None, "No location");
                        for p in &d.places {
                            ui.selectable_value(
                                &mut self.new_date.place_id,
                                Some(p.id.clone()),
                                &p.name,
                            );
                        }
                    });
            });
        });
        if ui.button("Add date to dataset").clicked() {
            if let Err(e) = self.add_new_date() {
                self.status = format!("Date not added: {e}");
            }
        }
        if ui.small_button("Cancel new date").clicked() {
            self.show_new_date = false;
            ui.ctx().request_repaint();
        }
    }
    pub(super) fn dates(&mut self, ui: &mut egui::Ui) {
        ui.heading("Temporal workspace");
        self.temporal_controls(ui);
        self.active_calendar_properties(ui);
        if self.show_new_date {
            self.new_date_editor(ui);
        }
        self.temporal_comparison(ui);
        self.focus_dataset_browser(ui);
        let di = self.dataset_index();
        let mut action = None;
        if let Some(index) = self.workspace.datasets[di]
            .observations
            .iter()
            .position(|o| Some(&o.id) == self.temporal_date_id.as_ref())
        {
            egui::CollapsingHeader::new("Selected date · entity, location and context")
                .id_salt("temporal_focus_editor")
                .show(ui, |ui| {
                    let d = &mut self.workspace.datasets[di];
                    let o = &mut d.observations[index];
                    let result = observation_editor::edit(
                        ui,
                        o,
                        &d.entities,
                        &d.places,
                        &mut self.date_text,
                        &mut self.time_text,
                    );
                    action = Some((o.clone(), result));
                });
        }
        if let Some((o, a)) = action {
            if a.activate
                || (a.moment_changed
                    && self.workspace.sessions[self.session]
                        .active_observation
                        .as_ref()
                        == Some(&o.id))
            {
                self.activate_observation(&o);
            }
            if a.open_records {
                self.open_record(&o);
            }
            if a.remove {
                self.remove_date(&o.id);
            }
        }
        ui.weak(self.date_scope_summary());
    }
    pub(super) fn records(&mut self, ui: &mut egui::Ui) {
        ui.heading("Entities & labeled observations");
        self.city_search(ui);
        let di = self.dataset_index();
        let mut remove_entity = None;
        let d = &mut self.workspace.datasets[di];
        ui.horizontal_wrapped(|ui| {
            if ui.button("+ Entity").clicked() {
                d.entities
                    .push(Entity::new(&format!("Entity {}", d.entities.len() + 1)));
                self.entity = d.entities.len() - 1;
                self.records_unlinked = false;
            }
            if !d.entities.is_empty() {
                self.entity = self.entity.min(d.entities.len() - 1);
                egui::ComboBox::from_id_salt("edit_entity")
                    .selected_text(&d.entities[self.entity].name)
                    .show_ui(ui, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.entity_search)
                                .id(egui::Id::new("editor_entity_search"))
                                .hint_text("Find entity / alias"),
                        );
                        for (i, e) in d
                            .entities
                            .iter()
                            .enumerate()
                            .filter(|(_, e)| {
                                self.scope.entities.contains(&e.id)
                                    && atlas_core::filter::text_matches(
                                        &self.entity_search,
                                        &format!(
                                            "{} {} {}",
                                            e.name,
                                            e.aliases.join(" "),
                                            e.context
                                        ),
                                    )
                            })
                            .take(500)
                        {
                            if ui.selectable_value(&mut self.entity, i, &e.name).clicked() {
                                self.records_unlinked = false;
                            }
                        }
                    });
            }
            ui.selectable_value(&mut self.records_unlinked, true, "Unlinked dates");
            if !self.records_unlinked
                && !d.entities.is_empty()
                && ui.button("Remove entity & dates").clicked()
            {
                remove_entity = Some(d.entities[self.entity].id.clone());
            }
        });
        let eid = if !self.records_unlinked && !d.entities.is_empty() {
            let e = &mut d.entities[self.entity];
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut e.name);
                ui.label("Kind");
                ui.text_edit_singleline(&mut e.kind);
            });
            ui.label("Context");
            super::data_tools::attributes(ui, &e.attributes);
            ui.add(
                egui::TextEdit::multiline(&mut e.context)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );
            ui.horizontal(|ui| {
                ui.label("Source / evidence");
                ui.add(egui::TextEdit::singleline(&mut e.source).desired_width(500.));
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Home / default location");
                let selected = e
                    .place_id
                    .as_ref()
                    .and_then(|id| d.places.iter().find(|p| p.id == *id))
                    .map_or("No default location", |p| p.name.as_str());
                egui::ComboBox::from_id_salt(("entity_home", &e.id))
                    .selected_text(selected)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut e.place_id, None, "No default location");
                        for p in &d.places {
                            ui.selectable_value(&mut e.place_id, Some(p.id.clone()), &p.name);
                        }
                    });
            });
            ui.collapsing("Aliases & tags", |ui| {
                for (kind, values) in [("Alias", &mut e.aliases), ("Tag", &mut e.tags)] {
                    let mut remove = None;
                    for (i, v) in values.iter_mut().enumerate() {
                        ui.push_id((kind, i), |ui| {
                            ui.horizontal(|ui| {
                                ui.label(kind);
                                ui.text_edit_singleline(v);
                                if ui.small_button("Remove").clicked() {
                                    remove = Some(i);
                                }
                            });
                        });
                    }
                    if let Some(i) = remove {
                        values.remove(i);
                        self.reset_focus = true;
                        self.status = format!("Removed {kind} · Undo to recover");
                    }
                    if ui.button(format!("+ {kind}")).clicked() {
                        values.push(String::new());
                    }
                }
            });
            Some(e.id.clone())
        } else {
            None
        };
        ui.add_space(10.);
        ui.heading(if eid.is_some() {
            "Dates attached to this entity"
        } else {
            "Unlinked dates"
        });
        if ui.button("+ Labeled date").clicked() {
            let mut o = Observation::unlinked(self.workspace.sessions[self.session].active_date);
            o.entity_id = eid.clone();
            d.observations.push(o);
        }
        let mut actions = vec![];
        for o in d.observations.iter_mut().filter(|o| o.entity_id == eid) {
            let result = observation_editor::edit(
                ui,
                o,
                &d.entities,
                &d.places,
                &mut self.date_text,
                &mut self.time_text,
            );
            if self.scroll_to_record && self.focus_record.as_ref() == Some(&o.id) {
                ui.scroll_to_cursor(Some(egui::Align::Center));
                self.scroll_to_record = false;
            }
            actions.push((o.clone(), result));
        }
        for (o, a) in actions {
            if a.activate
                || (a.moment_changed
                    && self.workspace.sessions[self.session]
                        .active_observation
                        .as_ref()
                        == Some(&o.id))
            {
                self.activate_observation(&o);
            }
            if a.open_dates {
                self.temporal_date_id = Some(o.id.clone());
                self.workspace.sessions[self.session].dates.focus = Some(o.id.clone());
                self.tab = "Dates".into();
            }
            if a.remove {
                self.remove_date(&o.id);
            }
        }
        if let Some(id) = remove_entity {
            self.remove_entity(&id);
        }
        let d = &mut self.workspace.datasets[di];
        ui.add_space(14.);
        ui.heading("Shared places");
        ui.add(
            egui::TextEdit::singleline(&mut self.place_search)
                .id(egui::Id::new("editor_place_search"))
                .hint_text("Find location / context / source")
                .desired_width(350.),
        );
        if ui.button("+ Place").clicked() {
            let p = Place::new(&format!("Place {}", d.places.len() + 1));
            self.focus_place = Some(p.id.clone());
            self.scroll_to_place = true;
            d.places.push(p);
        }
        let mut remove = None;
        for p in &mut d.places {
            let focused = self.focus_place.as_ref() == Some(&p.id);
            if !focused
                && (!self.scope.places.contains(&p.id)
                    || !atlas_core::filter::text_matches(
                        &self.place_search,
                        &format!("{} {} {}", p.name, p.context, p.source),
                    ))
            {
                continue;
            }
            ui.push_id(&p.id, |ui| {
                if focused && self.scroll_to_place {
                    ui.scroll_to_cursor(Some(egui::Align::Min));
                }
                let response = egui::CollapsingHeader::new(p.name.clone())
                    .id_salt("place_editor")
                    .open((focused && self.scroll_to_place).then_some(true))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Name");
                            ui.add(egui::TextEdit::singleline(&mut p.name).id_salt("place_name"));
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Latitude");
                            ui.add(
                                egui::DragValue::new(&mut p.latitude)
                                    .range(-90. ..=90.)
                                    .speed(0.01)
                                    .max_decimals(6),
                            );
                            ui.label("Longitude");
                            ui.add(
                                egui::DragValue::new(&mut p.longitude)
                                    .range(-180. ..=180.)
                                    .speed(0.01)
                                    .max_decimals(6),
                            );
                            ui.label("Elevation m");
                            ui.add(egui::DragValue::new(&mut p.elevation_m));
                        });
                        ui.horizontal(|ui| {
                            ui.label("IANA timezone");
                            ui.text_edit_singleline(&mut p.timezone);
                            crate::timezones::presets(ui, &mut p.timezone, "place_zone_presets");
                        });
                        if ui
                            .small_button("Find city coordinates for this place")
                            .clicked()
                        {
                            self.geo_target = Some((d.id.clone(), p.id.clone()));
                            self.open_city_search = true;
                            self.city_lookup.city = p.name.clone();
                            self.status =
                                "Use City / state lookup above, then Apply to this place".into();
                            ui.ctx().request_repaint();
                        }
                        super::data_tools::attributes(ui, &p.attributes);
                        ui.label("Context");
                        ui.text_edit_multiline(&mut p.context);
                        ui.label("Source");
                        ui.text_edit_singleline(&mut p.source);
                        if ui.button("Remove place").clicked() {
                            remove = Some(p.id.clone());
                        }
                    });
                if response.header_response.clicked() {
                    self.focus_place = Some(p.id.clone());
                }
                if focused {
                    self.scroll_to_place = false;
                }
            });
        }
        if let Some(id) = remove {
            self.remove_place(&id);
        }
    }
    pub(super) fn earth(&mut self, ui: &mut egui::Ui) {
        ui.heading("Earth · locations and daylight");
        self.city_search(ui);
        self.location_controls(ui);
        let di = self.dataset_index();
        if let Some(p) = self.workspace.datasets[di]
            .places
            .iter()
            .find(|p| Some(&p.id) == self.workspace.sessions[self.session].active_place.as_ref())
            .cloned()
        {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Sky observer: {}",p.name));
                if ui.button("Edit selected location").clicked() {self.focus_place=Some(p.id.clone());self.scroll_to_place=true;self.tab="Records".into();}
                if ui.button("Remove selected location").on_hover_text("Clears entity/date/observer links; retains dates. Undo restores the location and its links.").clicked() {self.remove_place(&p.id);}
            });
        }
        let s = self.effective_session();
        self.refresh_sky(&s);
        let mut action = None;
        ui.columns(2, |columns| {
            self.location_details(&mut columns[0]);
            let snapshot = self.sky_cache.as_ref().and_then(|r| r.as_ref().ok());
            action = views::draw_earth(
                &mut columns[1],
                &mut self.map_view,
                &self.workspace.datasets[di],
                &s,
                &self.feature_cache,
                snapshot,
            );
        });
        match action {
            Some(views::MapAction::Remove(id)) => self.remove_place(&id),
            Some(views::MapAction::Observer(id)) => {
                self.select_map_location(id);
            }
            Some(views::MapAction::NewPoint(lat, lon)) => {
                let mut p = Place::new(&format!(
                    "Map point {}",
                    self.workspace.datasets[di].places.len() + 1
                ));
                p.latitude = lat;
                p.longitude = lon;
                p.context = "Selected on the map; coordinate precision depends on zoom".into();
                p.source = "Manual map selection".into();
                let id = p.id.clone();
                self.workspace.datasets[di].places.push(p);
                self.select_map_location(id);
                self.status =
                    "Added map location · set its name, timezone and evidence in Records".into();
            }
            None => {}
        }
        if let Some(Ok(sky)) = &self.sky_cache {
            ui.weak(format!(
                "Day/night shading at {} UTC · geometric solar terminator",
                sky.utc
            ));
        } else {
            ui.weak("Choose an observer and a valid time (1700–2200) to show day/night shading.");
        }
        ui.weak(self.location_scope_summary());
    }
    pub(super) fn shift_time(&mut self, delta: Duration) -> anyhow::Result<()> {
        let mut s = self.effective_session();
        let local = s
            .active_date
            .and_time(s.active_time)
            .checked_add_signed(delta)
            .ok_or_else(|| anyhow::anyhow!("Time shift is out of range"))?;
        temporal::utc_instant(local.date(), local.time(), &s.timezone)?;
        s.active_date = local.date();
        s.active_time = local.time();
        s.active_observation = None;
        s.time_assumed = true;
        self.workspace.sessions[self.session] = s;
        self.live_time = false;
        self.sync_active();
        Ok(())
    }
    pub(super) fn sky(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sky · active observer and time");
        self.date_choices(ui);
        if self.show_new_date {
            self.new_date_editor(ui);
        }
        let mut shift = None;
        ui.horizontal_wrapped(|ui| {
            for (label, hours) in [
                ("−1 day", -24),
                ("−1 hour", -1),
                ("+1 hour", 1),
                ("+1 day", 24),
            ] {
                if ui.button(label).clicked() {
                    shift = Some(Duration::hours(hours));
                }
            }
            if ui.button("Assume noon").clicked() {
                let t = self.effective_session().active_time;
                shift = Some(Duration::seconds(
                    12 * 3600 - t.num_seconds_from_midnight() as i64,
                ));
            }
            if ui.button("Assume midnight").clicked() {
                let t = self.effective_session().active_time;
                shift = Some(Duration::seconds(-(t.num_seconds_from_midnight() as i64)));
            }
            ui.selectable_value(&mut self.ecliptic_view, false, "Horizon");
            ui.selectable_value(&mut self.ecliptic_view, true, "Ecliptic longitude");
            ui.checkbox(
                &mut self.workspace.sessions[self.session]
                    .settings
                    .atmospheric_refraction,
                "Standard refraction",
            );
        });
        if let Some(delta) = shift {
            if let Err(e) = self.shift_time(delta) {
                self.status = format!("Time not shifted: {e}");
            }
        }
        let mut minute = self
            .effective_session()
            .active_time
            .num_seconds_from_midnight()
            / 60;
        if ui
            .add_enabled(
                !self.live_time,
                egui::Slider::new(&mut minute, 0..=1439)
                    .text("Local time of day")
                    .custom_formatter(|value, _| {
                        format!("{:02}:{:02}", value as u32 / 60, value as u32 % 60)
                    }),
            )
            .changed()
        {
            let t = self.effective_session().active_time;
            let delta =
                Duration::seconds((minute * 60) as i64 - t.num_seconds_from_midnight() as i64);
            if let Err(e) = self.shift_time(delta) {
                self.status = format!("Time not shifted: {e}");
            }
        }
        let s = self.effective_session();
        self.refresh_sky(&s);
        let snapshot = match self.sky_cache.as_ref().unwrap() {
            Ok(sky) => sky.clone(),
            Err(e) => {
                ui.colored_label(HIT, e);
                return;
            }
        };
        let observer = self.workspace.datasets[self.dataset_index()]
            .places
            .iter()
            .find(|p| Some(&p.id) == s.active_place.as_ref())
            .unwrap();
        ui.label(format!(
            "{} · {:.4}°, {:.4}° · {} m · {} {} {} / {} UTC",
            observer.name,
            observer.latitude,
            observer.longitude,
            observer.elevation_m,
            s.active_date,
            s.active_time,
            s.timezone,
            snapshot.utc
        ));
        if s.time_assumed {
            ui.colored_label(
                HIT,
                "Viewing time is an assumption / exploration input. The source date is unchanged.",
            );
        }
        let sun = &snapshot.bodies[0];
        ui.label(format!(
            "Sun {:.2}° altitude · {} · Moon {:.1}% illuminated (phase {:.2}°)",
            sun.altitude,
            if sun.altitude >= 0. {
                "above horizon"
            } else if sun.altitude >= -6. {
                "civil twilight"
            } else if sun.altitude >= -12. {
                "nautical twilight"
            } else if sun.altitude >= -18. {
                "astronomical twilight"
            } else {
                "astronomical night"
            },
            snapshot.moon_illuminated_fraction * 100.,
            snapshot.moon_phase
        ));
        views::draw_sky(ui, &snapshot, &s, &self.feature_cache, self.ecliptic_view);
        ui.weak("Horizon rings: 0°, 30°, 60°, zenith 90°. North azimuth 0°, east 90°. Above horizon does not guarantee visibility through daylight or terrain.");
        let fs = self
            .feature_cache
            .iter()
            .filter(|f| f.domain == "Sky")
            .cloned()
            .collect::<Vec<_>>();
        feature_table(ui, &fs, &s.settings, &s.query, "sky_features");
    }
}
