use super::*;
use atlas_core::filter::{self, DataFilter, DateLink};
use std::collections::BTreeSet;

fn facet(ui: &mut egui::Ui, label: &str, value: &mut String, choices: BTreeSet<String>) {
    egui::ComboBox::from_id_salt(label)
        .selected_text(if value.is_empty() {
            format!("All {label}")
        } else {
            value.clone()
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(value, String::new(), format!("All {label}"));
            for choice in choices {
                ui.selectable_value(value, choice.clone(), choice);
            }
        });
}
impl AtlasApp {
    pub(super) fn filters(&mut self, ui: &mut egui::Ui) {
        let di = self.dataset_index();
        let d = &self.workspace.datasets[di];
        let s = &mut self.workspace.sessions[self.session];
        ui.horizontal_wrapped(|ui| {
            ui.label("Search data");
            ui.add(
                egui::TextEdit::singleline(&mut s.filter.text)
                    .id(egui::Id::new("dataset_filter_text"))
                    .desired_width(310.)
                    .hint_text("name, alias, context, source, metadata"),
            );
            ui.weak(format!(
                "{} / {} entities · {} / {} dates · {} / {} locations",
                self.scope.entities.len(),
                d.entities.len(),
                self.scope.observations.len(),
                d.observations.len(),
                self.scope.places.len(),
                d.places.len()
            ));
            if ui.button("Clear data filters").clicked() {
                s.filter = DataFilter::default();
                s.selected_entities.clear();
            }
        });
        egui::CollapsingHeader::new("Filter dataset scope").id_salt("dataset_facets").show(ui, |ui| {
            ui.weak("Applies to Browse, date choices, gematria, map and comparisons. Search terms use AND; quotes preserve phrases; -term excludes text. Active viewing time and observer stay unchanged.");
            ui.horizontal_wrapped(|ui| {
                facet(ui,"kinds",&mut s.filter.kind,d.entities.iter().map(|e|e.kind.clone()).filter(|v|!v.is_empty()).collect());
                facet(ui,"tags",&mut s.filter.tag,d.entities.iter().flat_map(|e|e.tags.iter().cloned()).collect());
                facet(ui,"conferences",&mut s.filter.conference,d.entities.iter().filter_map(|e|e.attributes.get("conference").and_then(|v|v.as_str()).map(str::to_owned)).collect());
                facet(ui,"divisions",&mut s.filter.division,d.entities.iter().filter_map(|e|e.attributes.get("division").and_then(|v|v.as_str()).map(str::to_owned)).collect());
                let selected=s.filter.place_id.as_ref().and_then(|id|d.places.iter().find(|p|p.id==*id)).map_or("All locations",|p|p.name.as_str());
                egui::ComboBox::from_id_salt("filter_place").selected_text(selected).show_ui(ui, |ui| {
                    ui.selectable_value(&mut s.filter.place_id,None,"All locations");
                    for p in &d.places {ui.selectable_value(&mut s.filter.place_id,Some(p.id.clone()),&p.name);}
                });
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Dates from"); ui.add(egui::TextEdit::singleline(&mut s.filter.from).id_salt("filter_from").desired_width(100.).hint_text("YYYY-MM-DD"));
                ui.label("through"); ui.add(egui::TextEdit::singleline(&mut s.filter.until).id_salt("filter_until").desired_width(100.).hint_text("YYYY-MM-DD"));
                egui::ComboBox::from_id_salt("date_link_filter").selected_text(format!("{:?} dates",s.filter.link)).show_ui(ui,|ui| {for v in [DateLink::All,DateLink::Linked,DateLink::Unlinked] {ui.selectable_value(&mut s.filter.link,v,format!("{v:?} dates"));}});
                ui.checkbox(&mut s.filter.sourced_only,"Only sourced records");
                ui.label("Pair limit per domain"); ui.add(egui::DragValue::new(&mut s.settings.pair_limit).range(0..=100000).speed(100.));
            });
            ui.weak("Date boundaries are inclusive. Entity filters include connected event/home locations. Location filtering matches event locations or an entity's home/default place. Find number in the toolbar searches computed values.");
        });
        if let Some(error) = &self.scope.error {
            ui.colored_label(
                HIT,
                format!("Filter needs attention: {error}. Clear filters to show all data."),
            );
        }
        if s.filter.active() || !s.selected_entities.is_empty() {
            ui.weak("Filtered scope · Clear data filters restores all saved records. Editing a selected record remains available even if a filter hides it.");
        }
        for (domain, n) in [
            ("date", analysis::date_observations(d, s, &self.scope).len()),
            (
                "location",
                analysis::location_places(d, s, &self.scope).len(),
            ),
        ] {
            let total = filter::pairs(n);
            if total > s.settings.pair_limit as usize {
                ui.colored_label(HIT,format!("{} of {total} {domain} pairs calculated in dataset order. Narrow filters or increase Pair limit.",s.settings.pair_limit));
            }
        }
        if s.active_place
            .as_ref()
            .is_some_and(|id| !self.scope.places.contains(id))
        {
            ui.weak("The active observer is outside the filtered scope; its marker and sky view remain visible.");
        }
        ui.separator();
    }
    pub(super) fn browse(&mut self, ui: &mut egui::Ui) {
        ui.heading("Browse saved records");
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.browse_search)
                    .id(egui::Id::new("browse_record_search"))
                    .desired_width(310.)
                    .hint_text("Search within these results"),
            );
            egui::ComboBox::from_id_salt("browse_type")
                .selected_text(&self.browse_kind)
                .show_ui(ui, |ui| {
                    for kind in ["All records", "Entities", "Dates", "Locations"] {
                        ui.selectable_value(&mut self.browse_kind, kind.into(), kind);
                    }
                });
            ui.checkbox(&mut self.browse_all, "Browse all saved records");
        });
        let d = &self.workspace.datasets[self.dataset_index()];
        let names = d
            .entities
            .iter()
            .map(|e| (e.id.as_str(), e.name.as_str()))
            .collect::<std::collections::HashMap<_, _>>();
        let mut rows = vec![];
        if self.browse_kind == "All records" || self.browse_kind == "Entities" {
            for e in &d.entities {
                let text = filter::entity_text(e);
                if (self.browse_all || self.scope.entities.contains(&e.id))
                    && filter::text_matches(&self.browse_search, &text)
                {
                    rows.push((
                        "Entity",
                        e.id.clone(),
                        e.name.clone(),
                        format!("{} · {}", e.kind, e.tags.join(", ")),
                        e.source.clone(),
                    ));
                }
            }
        }
        if self.browse_kind == "All records" || self.browse_kind == "Dates" {
            for o in &d.observations {
                let name = o
                    .entity_id
                    .as_ref()
                    .and_then(|id| names.get(id.as_str()).copied())
                    .unwrap_or("Unlinked");
                let label = format!("{} · {} · {name}", o.date, o.label);
                let text = format!("{name} {}", filter::date_text(o));
                if (self.browse_all || self.scope.observations.contains(&o.id))
                    && filter::text_matches(&self.browse_search, &text)
                {
                    rows.push((
                        "Date",
                        o.id.clone(),
                        label,
                        o.time.map_or("Time unknown".into(), |t| t.to_string()),
                        o.source.clone(),
                    ));
                }
            }
        }
        if self.browse_kind == "All records" || self.browse_kind == "Locations" {
            for p in &d.places {
                let text = filter::place_text(p);
                if (self.browse_all || self.scope.places.contains(&p.id))
                    && filter::text_matches(&self.browse_search, &text)
                {
                    rows.push((
                        "Location",
                        p.id.clone(),
                        p.name.clone(),
                        format!("{:.4}, {:.4} · {}", p.latitude, p.longitude, p.timezone),
                        p.source.clone(),
                    ));
                }
            }
        }
        rows.sort_by_cached_key(|(kind, _, label, _, _)| (*kind, label.to_lowercase()));
        ui.label(format!(
            "{} matching saved records{}",
            rows.len(),
            if self.browse_all {
                " · session filters bypassed for browsing only"
            } else {
                " · current session scope"
            }
        ));
        if rows.is_empty() {
            ui.weak("No results. Clear the local search or data filters, or enable Browse all saved records.");
        }
        let mut action = None;
        egui::ScrollArea::vertical()
            .id_salt("browse_results")
            .max_height(520.)
            .show_rows(ui, 48., rows.len(), |ui, range| {
                for i in range {
                    let (kind, id, label, detail, source) = &rows[i];
                    ui.push_id(id, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Open record").clicked() {
                                action = Some((*kind, id.clone()));
                            }
                            ui.vertical(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(format!("{kind}: {label}")).strong(),
                                    )
                                    .truncate(),
                                )
                                .on_hover_text(label);
                                ui.add(egui::Label::new(RichText::new(detail).weak()).truncate())
                                    .on_hover_text(if source.is_empty() {
                                        "No source recorded"
                                    } else {
                                        source
                                    });
                            });
                        });
                    });
                }
            });
        if let Some((kind, id)) = action {
            match kind {
                "Entity" => {
                    self.entity = d.entities.iter().position(|e| e.id == id).unwrap();
                    self.records_unlinked = false;
                    self.tab = "Records".into();
                }
                "Location" => {
                    self.focus_place = Some(id);
                    self.scroll_to_place = true;
                    self.tab = "Records".into();
                }
                _ => {
                    let o = d.observations.iter().find(|o| o.id == id).unwrap().clone();
                    self.open_record(&o);
                }
            }
            ui.ctx().request_repaint();
        }
    }
}
