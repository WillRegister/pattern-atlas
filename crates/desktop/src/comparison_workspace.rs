use super::*;

impl AtlasApp {
    pub(super) fn comparisons(&mut self, ui: &mut egui::Ui) {
        ui.heading("Shared numerical properties");
        let s = &mut self.workspace.sessions[self.session];
        ui.checkbox(
            &mut s.settings.include_zero,
            "Include zero-valued collisions",
        );
        ui.label("Each value appears once. Expand its row to inspect matching properties. Counts show the participating domains under current rounding; units remain visible in the details.");
        let groups = analysis::groups(&self.feature_cache, &s.settings);
        let mut ordered = groups.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|(_, fs)| !fs.iter().any(|f| f.matches(&s.query, &s.settings)));
        if ordered.is_empty() {
            ui.weak("No repeated numeric values in the current analysis scope.");
            return;
        }
        let domains = ["Gematria", "Dates", "Geography", "Sky", "Data"];
        ui.horizontal(|ui| {
            ui.add_space(ui.spacing().indent);
            for (title, width) in [
                ("Value", 110.),
                ("Gematria", 95.),
                ("Dates", 95.),
                ("Geography", 95.),
                ("Sky", 95.),
                ("Data", 95.),
                ("Domains", 80.),
                ("Properties", 90.),
            ] {
                ui.add_sized(
                    [width, 20.],
                    egui::Label::new(RichText::new(title).strong()).truncate(),
                );
            }
        });
        for &(value, fs) in ordered.iter().take(200) {
            let counts = domains.map(|domain| fs.iter().filter(|f| f.domain == domain).count());
            let domain_count = counts.iter().filter(|n| **n > 0).count();
            let queried = fs.iter().any(|f| f.matches(&s.query, &s.settings));
            let color = if queried {
                HIT
            } else if domain_count > 1 {
                ACCENT
            } else {
                Color32::WHITE
            };
            let id = ui.make_persistent_id(("comparison_group", &s.id, value));
            let default_open = cfg!(debug_assertions)
                && queried
                && std::env::var("ATLAS_PREVIEW_VARIANT").as_deref() == Ok("inspect");
            let state = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                id,
                default_open,
            );
            let mut clicked = false;
            let mut header = state.show_header(ui, |ui| {
                clicked = ui
                    .add_sized(
                        [110., 20.],
                        egui::Button::new(RichText::new(value).strong().color(color)).frame(false),
                    )
                    .on_hover_text("Expand or collapse this value's matching properties")
                    .clicked();
                for (domain, count) in domains.iter().zip(counts) {
                    ui.add_sized([95., 20.], egui::Label::new(count.to_string()))
                        .on_hover_text(
                            fs.iter()
                                .filter(|f| f.domain == *domain)
                                .map(|f| {
                                    format!(
                                        "{} · {} = {} {} (raw {})",
                                        f.owner,
                                        f.calculation,
                                        f.displayed(&s.settings),
                                        f.unit,
                                        f.raw
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
                        );
                }
                ui.add_sized([80., 20.], egui::Label::new(domain_count.to_string()));
                ui.add_sized([90., 20.], egui::Label::new(fs.len().to_string()));
            });
            if clicked {
                header.toggle();
            }
            header.body(|ui| {
                let owned = fs.iter().map(|f| (*f).clone()).collect::<Vec<_>>();
                feature_table(
                    ui,
                    &owned,
                    &s.settings,
                    &s.query,
                    &format!("comparison_properties_{value}"),
                );
            });
        }
        if groups.len() > 200 {
            ui.weak(format!("Showing 200 of {} groups, with number-query matches first. Every shown group can be expanded; narrow scope to see more groups. CLI --analyze exports all scoped properties.",groups.len()));
        }
    }
}
