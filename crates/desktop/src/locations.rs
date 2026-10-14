use super::*;

fn metric(ui: &mut egui::Ui, f: &Feature, s: &Session) {
    let color = if f.matches(&s.query, &s.settings) {
        HIT
    } else {
        Color32::WHITE
    };
    let value = if f.unit == "degrees" {
        format!("{}°", f.displayed(&s.settings))
    } else {
        format!("{} {}", f.displayed(&s.settings), f.unit)
    };
    ui.label(RichText::new(value).color(color))
        .on_hover_text(format!("Exact: {} {}\n{}", f.raw, f.unit, f.explanation));
}

impl AtlasApp {
    pub(super) fn select_map_location(&mut self, id: String) {
        let s = &mut self.workspace.sessions[self.session];
        match self.map_view.click_target {
            1 => {
                s.locations.a = Some(id);
                s.locations.include_a = true;
                self.status = "Location A selected".into();
            }
            2 => {
                s.locations.b = Some(id);
                s.locations.include_b = true;
                self.status = "Location B selected".into();
            }
            _ => {
                s.active_place = Some(id);
                self.status = "Active observer updated · Sky uses this location".into();
            }
        }
    }
    pub(super) fn location_scope_summary(&self) -> String {
        let d = &self.workspace.datasets[self.dataset_index()];
        let s = &self.workspace.sessions[self.session];
        let n = analysis::location_places(d, s, &self.scope).len();
        let total = atlas_core::filter::pairs(n);
        format!("Location analysis: {} · {n} locations · {} of {total} pairs. Overview shows number hits; Comparison uses these calculated values. Change this scope in Earth.",
            if s.locations.all_locations { "all filtered locations" } else { "enabled A/B selections" }, total.min(s.settings.pair_limit as usize))
    }

    pub(super) fn location_controls(&mut self, ui: &mut egui::Ui) {
        let di = self.dataset_index();
        let d = &self.workspace.datasets[di];
        let s = &mut self.workspace.sessions[self.session];
        let mut edit = None;
        let mut remove = None;
        let mut observer = None;
        ui.columns(2, |columns| {
            for (i, ui) in columns.iter_mut().enumerate() {
                let letter = if i == 0 { "A" } else { "B" };
                let (id, enabled) = if i == 0 {
                    (&mut s.locations.a, &mut s.locations.include_a)
                } else {
                    (&mut s.locations.b, &mut s.locations.include_b)
                };
                ui.horizontal(|ui| {
                    ui.checkbox(enabled, format!("Include {letter}")).on_hover_text("Show this point in the focused pair. Selected A/B analysis searches enabled selections; All filtered locations searches the broader set.");
                    let name = d
                        .places
                        .iter()
                        .find(|p| Some(&p.id) == id.as_ref())
                        .map_or("Choose location", |p| p.name.as_str());
                    egui::ComboBox::from_id_salt(("compare_place", i))
                        .width(220.)
                        .selected_text(name)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(id, None, "Choose location");
                            let mut places = d.places.iter().collect::<Vec<_>>();
                            places.sort_by_key(|p| p.name.to_lowercase());
                            for p in places {
                                let label = if self.scope.places.contains(&p.id) {
                                    p.name.clone()
                                } else {
                                    format!("{} (filtered out)", p.name)
                                };
                                ui.selectable_value(id, Some(p.id.clone()), label);
                            }
                        });
                });
                if let Some(p) = d.places.iter().find(|p| Some(&p.id) == id.as_ref()) {
                    ui.horizontal(|ui| {
                        if ui
                            .button(format!("Observe {letter}"))
                            .on_hover_text(
                                "Set the sky observer and its timezone; retain the active instant.",
                            )
                            .clicked()
                        {
                            observer = Some(p.clone());
                        }
                        if ui.button(format!("Edit {letter}")).clicked() {
                            edit = Some(p.id.clone());
                        }
                        if ui.button(format!("Remove {letter}")).clicked() {
                            remove = Some(p.id.clone());
                        }
                    });
                    if !self.scope.places.contains(&p.id) {
                        ui.colored_label(HIT, "Hidden by data filters; excluded from analysis.");
                    }
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Location analysis");
            ui.selectable_value(&mut s.locations.all_locations, false, "Selected A/B only");
            ui.selectable_value(
                &mut s.locations.all_locations,
                true,
                "All filtered locations",
            );
            ui.weak("Applies to Overview, number search and Comparison.");
        });
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut s.settings.kilometers, "Kilometers");
            ui.checkbox(&mut s.settings.miles, "Miles");
            ui.checkbox(&mut s.settings.meters, "Meters");
            ui.checkbox(&mut s.settings.feet, "Feet");
            ui.label("Map click selects");
            ui.selectable_value(&mut self.map_view.click_target, 0, "Observer");
            ui.selectable_value(&mut self.map_view.click_target, 1, "A");
            ui.selectable_value(&mut self.map_view.click_target, 2, "B");
        });
        if let Some(p) = observer {
            let old = self.effective_session();
            match temporal::utc_instant(old.active_date, old.active_time, &old.timezone) {
                Ok(utc) => {
                    if let Ok(tz) = p.timezone.parse::<chrono_tz::Tz>() {
                        let local = utc.with_timezone(&tz);
                        let s = &mut self.workspace.sessions[self.session];
                        s.active_place = Some(p.id);
                        s.timezone = p.timezone;
                        s.active_date = local.date_naive();
                        s.active_time = local.time();
                        self.sync_active();
                    }
                }
                Err(e) => self.status = format!("Cannot switch observer: {e}"),
            }
        }
        if let Some(id) = edit {
            self.focus_place = Some(id);
            self.scroll_to_place = true;
            self.tab = "Records".into();
        }
        if let Some(id) = remove {
            self.remove_place(&id);
        }
    }

    pub(super) fn location_details(&mut self, ui: &mut egui::Ui) {
        let d = &self.workspace.datasets[self.dataset_index()];
        let mut s = self.effective_session();
        s.locations.all_locations = false;
        let places = analysis::location_places(d, &s, &self.scope);
        let fs = analysis::geographic_features(&places, &s);
        self.map_view.comparison = [
            s.locations
                .a
                .clone()
                .filter(|id| s.locations.include_a && self.scope.places.contains(id)),
            s.locations
                .b
                .clone()
                .filter(|id| s.locations.include_b && self.scope.places.contains(id)),
        ];
        self.map_view.visible_places = Some(
            if self.workspace.sessions[self.session]
                .locations
                .all_locations
            {
                self.scope.places.clone()
            } else {
                places.iter().map(|p| p.id.clone()).collect()
            },
        );
        ui.columns(2, |columns| {
            for (i, ui) in columns.iter_mut().enumerate() {
                let id = &self.map_view.comparison[i];
                if let Some(p) = places.iter().find(|p| Some(&p.id) == id.as_ref()) {
                    ui.group(|ui| {
                        ui.strong(format!("{} · {}", if i == 0 { "A" } else { "B" }, p.name));
                        ui.horizontal_wrapped(|ui| {
                            for f in fs
                                .iter()
                                .filter(|f| f.owner_id == p.id && f.owner == p.name)
                            {
                                ui.label(&f.calculation);
                                metric(ui, f, &s);
                            }
                        });
                        ui.weak(format!("{} · elevation {} m", p.timezone, p.elevation_m));
                        egui::CollapsingHeader::new("Context and source")
                            .id_salt(("place_context", i))
                            .show(ui, |ui| {
                                ui.label(&p.context);
                                ui.weak(&p.source);
                                for (key, value) in &p.attributes {
                                    ui.label(format!("{key}: {value}"));
                                }
                            });
                    });
                } else {
                    ui.weak(format!(
                        "{} · select and include a location",
                        if i == 0 { "A" } else { "B" }
                    ));
                }
            }
        });
        if places.len() == 2 {
            ui.strong(format!("A ↔ B · {} ↔ {}", places[0].name, places[1].name));
            ui.horizontal_wrapped(|ui| {
                ui.label("Great-circle distance");
                for f in fs
                    .iter()
                    .filter(|f| f.calculation == "Great-circle distance")
                {
                    metric(ui, f, &s);
                }
            });
            ui.columns(3, |columns| {
                for (ui, labels) in columns.iter_mut().zip([
                    ["Central angle", "Shortest longitude separation"],
                    ["Signed latitude delta (B−A)", "Absolute latitude delta"],
                    [
                        "Signed longitude delta (B−A)",
                        "Absolute raw longitude delta",
                    ],
                ]) {
                    ui.group(|ui| {
                        for label in labels {
                            if let Some(f) = fs.iter().find(|f| f.calculation == label) {
                                let caption = match label {
                                    "Shortest longitude separation" => "Shortest longitude gap",
                                    "Signed latitude delta (B−A)" => "Latitude B − A",
                                    "Absolute latitude delta" => "Latitude difference",
                                    "Signed longitude delta (B−A)" => "Longitude B − A",
                                    "Absolute raw longitude delta" => "Raw longitude difference",
                                    _ => label,
                                };
                                ui.weak(caption).on_hover_text(label);
                                metric(ui, f, &s);
                            }
                        }
                    });
                }
            });
            ui.weak("Spherical surface distance; values use toolbar rounding. Hover a value for its exact result. The map line follows the same great circle.");
            if s.settings.pair_limit == 0 {
                ui.colored_label(
                    HIT,
                    "Pair limit is 0; raise it in Filter dataset scope to calculate this pair.",
                );
            }
        } else if s.locations.a.is_some()
            && s.locations.a == s.locations.b
            && s.locations.include_a
            && s.locations.include_b
        {
            ui.weak("A and B are the same location. Choose a different B to compare two points.");
        } else {
            ui.weak(
                "Select and include two locations to see their comparison and connecting line.",
            );
        }
    }
}
