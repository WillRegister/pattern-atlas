use super::*;
use atlas_core::gematria::{self, InputKind, SearchPlan};

struct TermRow {
    id: String,
    entity: String,
    text: String,
    kind: String,
    number: Option<Feature>,
    values: Vec<(Cipher, Feature)>,
}
fn value_feature(text: &str, cipher: Option<Cipher>, raw: f64) -> Feature {
    Feature {
        owner_id: String::new(),
        owner: text.into(),
        domain: "Gematria".into(),
        calculation: cipher.map_or("Literal number", Cipher::label).into(),
        unit: "value".into(),
        raw,
        explanation: cipher.map_or_else(
            || "Literal numeric input; digits are not scored as letters".into(),
            |c| c.breakdown(text),
        ),
    }
}
fn existing_entity(d: &Dataset, text: &str) -> Option<usize> {
    let key = text.trim().to_lowercase();
    if key.is_empty() {
        return None;
    }
    d.entities
        .iter()
        .position(|e| e.name.trim().to_lowercase() == key)
        .or_else(|| {
            d.entities.iter().position(|e| {
                e.aliases
                    .iter()
                    .any(|alias| alias.trim().to_lowercase() == key)
            })
        })
}
fn term_rows(d: &Dataset, scope: &atlas_core::filter::Scope, ciphers: &[Cipher]) -> Vec<TermRow> {
    d.entities
        .iter()
        .filter(|e| scope.entities.contains(&e.id))
        .flat_map(|e| {
            std::iter::once(&e.name)
                .chain(e.aliases.iter())
                .map(move |text| {
                    let number =
                        gematria::literal_number(text).map(|n| value_feature(text, None, n));
                    let values = if number.is_some() {
                        vec![]
                    } else {
                        ciphers
                            .iter()
                            .map(|c| (*c, value_feature(text, Some(*c), c.total(text) as f64)))
                            .collect()
                    };
                    TermRow {
                        id: e.id.clone(),
                        entity: e.name.clone(),
                        text: text.clone(),
                        kind: e.kind.clone(),
                        number,
                        values,
                    }
                })
        })
        .collect()
}
fn row_hit(row: &TermRow, plan: Option<&SearchPlan>, s: &Session) -> bool {
    let hit = |c, f: &Feature| {
        plan.map_or_else(
            || f.matches(&s.query, &s.settings),
            |p| p.matches_cell(c, f.raw, &s.settings),
        )
    };
    row.number.as_ref().is_some_and(|f| hit(None, f))
        || row.values.iter().any(|(c, f)| hit(Some(*c), f))
}
fn cell(
    ui: &mut egui::Ui,
    f: &Feature,
    cipher: Option<Cipher>,
    plan: Option<&SearchPlan>,
    s: &Session,
    width: f32,
) {
    let hit = plan.map_or_else(
        || f.matches(&s.query, &s.settings),
        |p| p.matches_cell(cipher, f.raw, &s.settings),
    );
    ui.add_sized(
        [width, 20.],
        egui::Label::new(
            RichText::new(f.displayed(&s.settings))
                .strong()
                .color(if hit { HIT } else { Color32::WHITE }),
        )
        .truncate(),
    )
    .on_hover_text(format!("Raw: {}\n{}", f.raw, f.explanation));
}
fn cipher_width(c: Cipher) -> f32 {
    (c.label().chars().count() as f32 * 7. + 12.).max(84.)
}
struct TermTable<'a> {
    rows: &'a [TermRow],
    ciphers: &'a [Cipher],
    plan: Option<&'a SearchPlan>,
    session: &'a Session,
    salt: &'a str,
}
impl TermTable<'_> {
    fn show(&self, ui: &mut egui::Ui) -> Option<String> {
        if self.rows.is_empty() {
            ui.weak("No matching saved terms in the current data scope.");
            return None;
        }
        let literals = self.rows.iter().any(|row| row.number.is_some());
        let width = 190.
            + 180.
            + 85.
            + self
                .ciphers
                .iter()
                .map(|c| cipher_width(*c) + 8.)
                .sum::<f32>()
            + if literals { 92. } else { 0. };
        let mut edit = None;
        egui::ScrollArea::horizontal()
            .id_salt((self.salt, "columns"))
            .show(ui, |ui| {
                ui.set_min_width(width);
                ui.horizontal(|ui| {
                    for (title, w) in [("Term", 190.), ("Entity", 180.), ("Type", 85.)] {
                        ui.add_sized(
                            [w - 8., 20.],
                            egui::Label::new(RichText::new(title).strong()).truncate(),
                        );
                    }
                    if literals {
                        ui.add_sized(
                            [84., 20.],
                            egui::Label::new(RichText::new("Number").strong()),
                        );
                    }
                    for c in self.ciphers {
                        ui.add_sized(
                            [cipher_width(*c), 20.],
                            egui::Label::new(RichText::new(c.label()).strong()).truncate(),
                        )
                        .on_hover_text(c.label());
                    }
                });
                egui::ScrollArea::vertical()
                    .id_salt((self.salt, "rows"))
                    .max_height(210.)
                    .show_rows(ui, 26., self.rows.len(), |ui, range| {
                        for i in range {
                            let row = &self.rows[i];
                            ui.horizontal(|ui| {
                                ui.add_sized([182., 20.], egui::Label::new(&row.text).truncate())
                                    .on_hover_text(&row.text);
                                if ui
                                    .add_sized(
                                        [172., 20.],
                                        egui::Button::new(&row.entity).frame(false),
                                    )
                                    .on_hover_text("Open this entity in Records")
                                    .clicked()
                                {
                                    edit = Some(row.id.clone());
                                }
                                ui.add_sized([77., 20.], egui::Label::new(&row.kind).truncate())
                                    .on_hover_text(&row.kind);
                                if literals {
                                    if let Some(f) = &row.number {
                                        cell(ui, f, None, self.plan, self.session, 84.);
                                    } else {
                                        ui.add_sized([84., 20.], egui::Label::new("—"));
                                    }
                                }
                                for c in self.ciphers {
                                    if let Some((_, f)) =
                                        row.values.iter().find(|(cipher, _)| cipher == c)
                                    {
                                        cell(
                                            ui,
                                            f,
                                            Some(*c),
                                            self.plan,
                                            self.session,
                                            cipher_width(*c),
                                        );
                                    } else {
                                        ui.add_sized(
                                            [cipher_width(*c), 20.],
                                            egui::Label::new("—"),
                                        )
                                        .on_hover_text(
                                            "This term is a literal number, not a letter total",
                                        );
                                    }
                                }
                            });
                        }
                    });
            });
        edit
    }
}

impl AtlasApp {
    pub(super) fn save_gematria_input(&mut self) -> anyhow::Result<String> {
        let s = &self.workspace.sessions[self.session];
        let plan = SearchPlan::new(&s.gematria, &s.settings.ciphers)?;
        anyhow::ensure!(
            plan.kind != InputKind::Empty,
            "Enter a phrase or number first"
        );
        let name = s.gematria.input.trim().to_string();
        let di = self.dataset_index();
        if let Some(index) = existing_entity(&self.workspace.datasets[di], &name) {
            self.entity = index;
            self.status = "Already saved · existing record and type retained".into();
            return Ok(self.workspace.datasets[di].entities[index].id.clone());
        }
        let mut entity = Entity::new(&name);
        entity.kind = "gematria".into();
        entity.source = "Manual gematria input".into();
        let id = entity.id.clone();
        self.workspace.datasets[di].entities.push(entity);
        self.entity = self.workspace.datasets[di].entities.len() - 1;
        self.status =
            "Saved input as a gematria entity · edit its type here or in Records · Undo to recover"
                .into();
        Ok(id)
    }
    fn open_gematria_entity(&mut self, id: &str) {
        let d = &self.workspace.datasets[self.dataset_index()];
        if let Some(index) = d.entities.iter().position(|e| e.id == id) {
            self.entity = index;
            self.records_unlinked = false;
            self.entity_search.clear();
            self.focus_record = None;
            self.tab = "Records".into();
        }
    }
    pub(super) fn gematria(&mut self, ui: &mut egui::Ui) {
        ui.heading("Gematria");
        ui.weak("Type a phrase or a number. Letter scores use ASCII A–Z, case-insensitive; other characters contribute zero. Numbers are literal values.");
        let mut save = false;
        ui.horizontal_wrapped(|ui| {
            ui.label("Live input");
            let search=&mut self.workspace.sessions[self.session].gematria;
            let response=ui.add(egui::TextEdit::singleline(&mut search.input).id(egui::Id::new("gematria_input")).desired_width(400.).hint_text("Phrase or number, e.g. Will or 56"));
            if response.lost_focus() && ui.input(|i|i.key_pressed(egui::Key::Enter)) {save=true;}
            let s=&self.workspace.sessions[self.session];let d=&self.workspace.datasets[self.dataset_index()];
            let valid=SearchPlan::new(&s.gematria,&s.settings.ciphers).is_ok_and(|p|p.kind!=InputKind::Empty);
            let exists=existing_entity(d,&s.gematria.input).is_some();
            if ui.add_enabled(valid && !exists,egui::Button::new("Save input as entity")).on_hover_text("Save a phrase or literal number with type gematria. Enter also saves. Existing names and aliases reuse their record.").clicked() {save=true;}
            if exists {ui.weak("Already saved");}
        });
        if save {
            if let Err(e) = self.save_gematria_input() {
                self.status = format!("Cannot save input: {e}");
            }
        }
        let di = self.dataset_index();
        let input = self.workspace.sessions[self.session].gematria.input.clone();
        if let Some(index) = existing_entity(&self.workspace.datasets[di], &input) {
            let mut open = None;
            ui.horizontal_wrapped(|ui| {
                let e = &mut self.workspace.datasets[di].entities[index];
                ui.weak(format!("Saved record: {}", e.name));
                ui.label("Entity type");
                ui.add(
                    egui::TextEdit::singleline(&mut e.kind)
                        .id(egui::Id::new(("gematria_type", &e.id)))
                        .desired_width(110.),
                );
                if ui.button("Edit record").clicked() {
                    open = Some(e.id.clone());
                }
                if e.name.trim().to_lowercase() != input.trim().to_lowercase() {
                    ui.weak("Input is an alias; type belongs to its parent entity.");
                }
            });
            if let Some(id) = open {
                self.open_gematria_entity(&id);
            }
        }
        let s = &mut self.workspace.sessions[self.session];
        ui.horizontal_wrapped(|ui| {
            for c in Cipher::ALL {
                let mut on = s.settings.ciphers.contains(&c);
                if ui.checkbox(&mut on, c.label()).changed() {
                    if on {
                        s.settings.ciphers.push(c);
                    } else {
                        s.settings.ciphers.retain(|x| *x != c);
                    }
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut s.gematria.cross_cipher,"Cross-cipher matches").on_hover_text("On: any enabled cipher can match a selected input value. Off: phrase values match only the same cipher. Literal saved numbers have no cipher.");
            ui.label("Match input values from");
            let selected=s.gematria.source_cipher.map_or_else(||"All enabled ciphers".into(),|c|if s.settings.ciphers.contains(&c) {c.label().to_string()} else {format!("{} (disabled)",c.label())});
            egui::ComboBox::from_id_salt("gematria_source_cipher").selected_text(selected).show_ui(ui,|ui| {
                ui.selectable_value(&mut s.gematria.source_cipher,None,"All enabled ciphers");
                for c in Cipher::ALL.into_iter().filter(|c|s.settings.ciphers.contains(c)) {ui.selectable_value(&mut s.gematria.source_cipher,Some(c),c.label());}
            });
        });
        let ciphers = Cipher::ALL
            .into_iter()
            .filter(|c| s.settings.ciphers.contains(c))
            .collect::<Vec<_>>();
        let plan = SearchPlan::new(&s.gematria, &ciphers);
        if let Err(e) = &plan {
            ui.colored_label(HIT, e.to_string());
        }
        if let Ok(plan) = &plan {
            match plan.kind {
                InputKind::Phrase => {
                    ui.weak("Input totals · click a value to use that cipher and highlight its number throughout the app");
                    let mut selected = None;
                    egui::ScrollArea::horizontal()
                        .id_salt("gematria_input_totals")
                        .show(ui, |ui| {
                            egui::Grid::new("live_gematria_totals")
                                .min_col_width(90.)
                                .show(ui, |ui| {
                                    for c in &ciphers {
                                        ui.strong(c.label());
                                    }
                                    ui.end_row();
                                    for c in &ciphers {
                                        let f = value_feature(
                                            &s.gematria.input,
                                            Some(*c),
                                            c.total(&s.gematria.input) as f64,
                                        );
                                        if ui
                                            .selectable_label(
                                                s.gematria.source_cipher == Some(*c),
                                                f.displayed(&s.settings),
                                            )
                                            .on_hover_text(c.breakdown(&s.gematria.input))
                                            .clicked()
                                        {
                                            selected = Some((*c, f.raw));
                                        }
                                    }
                                    ui.end_row();
                                });
                        });
                    if let Some((c, n)) = selected {
                        s.gematria.source_cipher = Some(c);
                        s.query = n.to_string();
                    }
                }
                InputKind::Number(number) => {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("Literal number target: {number}"));
                        if ui.button("Highlight number everywhere").clicked() {
                            s.query = number.to_string();
                        }
                    });
                    if !s.gematria.cross_cipher && s.gematria.source_cipher.is_none() {
                        ui.weak("A number has no source cipher. Choose one above to lock the search to that cipher.");
                    }
                }
                InputKind::Empty => {
                    ui.weak("Enter input for matching results. Saved terms below still highlight the toolbar number query.");
                }
            }
        }
        let s = self.effective_session();
        let plan = SearchPlan::new(&s.gematria, &ciphers).ok();
        let has_input = !s.gematria.input.trim().is_empty();
        if has_input && plan.as_ref().is_some_and(|p| p.targets.is_empty()) {
            ui.weak("Enable the selected source cipher, or choose an enabled cipher above.");
        }
        let d = &self.workspace.datasets[di];
        let scope = atlas_core::filter::scope(d, &s);
        let all = term_rows(d, &scope, &ciphers);
        let mut rows = all
            .into_iter()
            .filter(|row| !has_input || plan.as_ref().is_some_and(|p| row_hit(row, Some(p), &s)))
            .collect::<Vec<_>>();
        if !has_input {
            rows.sort_by_key(|row| !row_hit(row, None, &s));
        }
        ui.separator();
        ui.strong(if has_input {
            "Matching saved terms"
        } else {
            "Saved terms"
        });
        ui.weak(format!(
            "{} terms · {} cipher columns · aliases have their own rows",
            rows.len(),
            ciphers.len()
        ));
        let match_plan = if has_input { plan.as_ref() } else { None };
        let mut edit = TermTable {
            rows: &rows,
            ciphers: &ciphers,
            plan: match_plan,
            session: &s,
            salt: "gematria_matches",
        }
        .show(ui);
        if has_input {
            egui::CollapsingHeader::new("Browse all saved terms in scope")
                .id_salt("browse_gematria_terms")
                .show(ui, |ui| {
                    let all = term_rows(d, &scope, &ciphers);
                    if let Some(id) = (TermTable {
                        rows: &all,
                        ciphers: &ciphers,
                        plan: plan.as_ref(),
                        session: &s,
                        salt: "gematria_browse",
                    })
                    .show(ui)
                    {
                        edit = Some(id);
                    }
                });
            ui.separator();
            ui.strong("Cross-domain matches");
            let filters = &mut self.workspace.sessions[self.session].gematria;
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(&mut filters.dates, "Dates");
                ui.checkbox(&mut filters.geography, "Geography");
                ui.checkbox(&mut filters.sky, "Sky");
                ui.checkbox(&mut filters.data, "Recorded data");
            });
            ui.weak("Uses current data filters, Date/Earth analysis scopes, active sky observer and toolbar rounding.");
            let fs = self
                .feature_cache
                .iter()
                .filter(|f| {
                    filters.includes_domain(&f.domain)
                        && plan
                            .as_ref()
                            .is_some_and(|p| p.matches_feature(f, &s.settings))
                })
                .collect::<Vec<_>>();
            // Put one match from each domain first, before repeated properties in any one domain.
            let mut counts = BTreeMap::new();
            let mut ranked = fs
                .into_iter()
                .map(|f| {
                    let seen = counts.entry(f.domain.as_str()).or_insert(0);
                    let rank = *seen;
                    *seen += 1;
                    (rank, f)
                })
                .collect::<Vec<_>>();
            ranked.sort_by_key(|(rank, f)| {
                (
                    *rank,
                    match f.domain.as_str() {
                        "Geography" => 0,
                        "Dates" => 1,
                        "Sky" => 2,
                        _ => 3,
                    },
                )
            });
            let fs = ranked.into_iter().map(|(_, f)| f).collect::<Vec<_>>();
            ui.weak(format!("{} matching properties", fs.len()));
            if fs.is_empty() {
                ui.weak("No cross-domain matches in the current analysis scope.");
            } else {
                cross_domain_table(ui, &fs, &s);
            }
        }
        if let Some(id) = edit {
            self.open_gematria_entity(&id);
        }
    }
}

fn cross_domain_table(ui: &mut egui::Ui, fs: &[&Feature], s: &Session) {
    egui::ScrollArea::horizontal()
        .id_salt("gematria_cross_columns")
        .show(ui, |ui| {
            ui.set_min_width(940.);
            ui.horizontal(|ui| {
                for (title, w) in [
                    ("Record", 300.),
                    ("Domain", 95.),
                    ("Calculation", 330.),
                    ("Value", 100.),
                    ("Unit", 90.),
                ] {
                    ui.add_sized(
                        [w, 20.],
                        egui::Label::new(RichText::new(title).strong()).truncate(),
                    );
                }
            });
            egui::ScrollArea::vertical()
                .id_salt("gematria_cross_rows")
                .max_height(170.)
                .show_rows(ui, 26., fs.len(), |ui, range| {
                    for i in range {
                        let f = fs[i];
                        ui.horizontal(|ui| {
                            for (value, w) in
                                [(&f.owner, 300.), (&f.domain, 95.), (&f.calculation, 330.)]
                            {
                                ui.add_sized([w, 20.], egui::Label::new(value).truncate())
                                    .on_hover_text(value);
                            }
                            ui.add_sized(
                                [100., 20.],
                                egui::Label::new(
                                    RichText::new(f.displayed(&s.settings)).strong().color(HIT),
                                ),
                            )
                            .on_hover_text(format!("Raw: {}\n{}", f.raw, f.explanation));
                            ui.add_sized([90., 20.], egui::Label::new(&f.unit).truncate());
                        });
                    }
                });
        });
}
