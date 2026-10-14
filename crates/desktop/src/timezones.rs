use eframe::egui;

pub fn presets(ui: &mut egui::Ui, zone: &mut String, salt: impl std::hash::Hash) {
    egui::ComboBox::from_id_salt(salt).selected_text("Timezone presets").show_ui(ui, |ui| {
        for (label, id) in [
            ("US Central · CST / CDT", "America/Chicago"),
            ("US Eastern · EST / EDT", "America/New_York"),
            ("US Mountain · MST / MDT", "America/Denver"),
            ("Arizona · MST", "America/Phoenix"),
            ("US Pacific · PST / PDT", "America/Los_Angeles"),
            ("UTC · no daylight saving", "UTC"),
        ] { ui.selectable_value(zone, id.into(), label); }
    }).response.on_hover_text("America/Chicago handles Central standard and daylight time automatically. Other IANA names can be typed in the field.");
}
