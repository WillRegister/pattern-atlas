use atlas_core::{model::*, temporal};
use chrono::NaiveTime;
use eframe::egui;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct DateAction {
    pub moment_changed: bool,
    pub activate: bool,
    pub remove: bool,
    pub open_records: bool,
    pub open_dates: bool,
}
pub fn edit(
    ui: &mut egui::Ui,
    o: &mut Observation,
    entities: &[Entity],
    places: &[Place],
    date_text: &mut BTreeMap<String, String>,
    time_text: &mut BTreeMap<String, String>,
) -> DateAction {
    let mut action = DateAction::default();
    let before = (o.date, o.time, o.timezone.clone(), o.place_id.clone());
    ui.push_id(&o.id, |ui| {
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Label");
                ui.add(egui::TextEdit::singleline(&mut o.label).desired_width(215.));
                let date = date_text
                    .entry(o.id.clone())
                    .or_insert_with(|| o.date.to_string());
                ui.add(egui::TextEdit::singleline(date).desired_width(110.));
                match temporal::parse_date(date) {
                    Ok(d) => o.date = d,
                    Err(_) => {
                        ui.colored_label(super::app::HIT, "Use YYYY-MM-DD");
                    }
                }
                let time = time_text
                    .entry(o.id.clone())
                    .or_insert_with(|| o.time.map(|t| t.to_string()).unwrap_or_default());
                ui.label("Time");
                ui.add(
                    egui::TextEdit::singleline(time)
                        .desired_width(110.)
                        .hint_text("unknown / HH:MM:SS"),
                );
                if time.trim().is_empty() {
                    o.time = None;
                } else {
                    match NaiveTime::parse_from_str(time, "%H:%M:%S%.f") {
                        Ok(t) => o.time = Some(t),
                        Err(_) => {
                            ui.colored_label(super::app::HIT, "Invalid time");
                        }
                    }
                }
                ui.label("Timezone");
                ui.add(egui::TextEdit::singleline(&mut o.timezone).desired_width(155.));
                crate::timezones::presets(ui, &mut o.timezone, "date_zone_presets");
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Entity");
                let name = entities
                    .iter()
                    .find(|e| Some(&e.id) == o.entity_id.as_ref())
                    .map_or("Unlinked date", |e| e.name.as_str());
                egui::ComboBox::from_id_salt("date_entity")
                    .selected_text(name)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut o.entity_id, None, "Unlinked date");
                        for e in entities {
                            ui.selectable_value(&mut o.entity_id, Some(e.id.clone()), &e.name);
                        }
                    });
                ui.label("Location");
                let name = places
                    .iter()
                    .find(|p| Some(&p.id) == o.place_id.as_ref())
                    .map_or("No linked place", |p| p.name.as_str());
                egui::ComboBox::from_id_salt("date_place")
                    .selected_text(name)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut o.place_id, None, "No linked place");
                        for p in places {
                            ui.selectable_value(&mut o.place_id, Some(p.id.clone()), &p.name);
                        }
                    });
                if let Some(p) = places.iter().find(|p| Some(&p.id) == o.place_id.as_ref()) {
                    if ui.small_button("Use place timezone").clicked() {
                        o.timezone = p.timezone.clone();
                    }
                }
            });
            ui.label("Context");
            super::app::data_tools::attributes(ui, &o.attributes);
            ui.add(
                egui::TextEdit::multiline(&mut o.context)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY),
            );
            ui.horizontal(|ui| {
                ui.label("Source / evidence");
                ui.add(egui::TextEdit::singleline(&mut o.source).desired_width(500.));
            });
            ui.horizontal_wrapped(|ui| {
                let valid = temporal::parse_date(date_text.get(&o.id).unwrap()).is_ok()
                    && time_text.get(&o.id).is_none_or(|t| {
                        t.trim().is_empty() || NaiveTime::parse_from_str(t, "%H:%M:%S%.f").is_ok()
                    });
                action.activate = ui
                    .add_enabled(valid, egui::Button::new("Use as active date/time"))
                    .clicked();
                action.open_records = ui.button("Details in Records").clicked();
                action.open_dates = ui.button("Compare in Dates").clicked();
                action.remove = ui.button("Remove date").clicked();
                if o.time.is_none() {
                    ui.weak("Source time is unknown; sky viewing uses a labeled assumption.");
                }
            });
        });
    });
    action.moment_changed = before != (o.date, o.time, o.timezone.clone(), o.place_id.clone());
    action
}
