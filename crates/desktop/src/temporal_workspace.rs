use super::workbench::NewDate;
use super::*;

fn label(d: &Dataset, o: &Observation) -> String {
    let entity = d
        .entities
        .iter()
        .find(|e| Some(&e.id) == o.entity_id.as_ref());
    let name = entity.map_or("Unlinked", |e| e.name.as_str());
    if o.label.contains(name) {
        format!("{} · {}", o.date, o.label)
    } else {
        format!("{} · {name} · {}", o.date, o.label)
    }
}

fn picker(
    ui: &mut egui::Ui,
    d: &Dataset,
    scope: &atlas_core::filter::Scope,
    id: &mut Option<String>,
    search: &mut String,
    salt: &str,
) {
    let selected = d
        .observations
        .iter()
        .find(|o| Some(&o.id) == id.as_ref())
        .map(|o| label(d, o))
        .unwrap_or_else(|| "Choose saved date".into());
    egui::ComboBox::from_id_salt(salt)
        .width(ui.available_width().min(480.))
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(search)
                    .id(egui::Id::new((salt, "search")))
                    .hint_text("Search team, entity, label, date or context"),
            );
            ui.selectable_value(id, None, "Choose saved date");
            let mut dates = d
                .observations
                .iter()
                .filter(|o| {
                    atlas_core::filter::text_matches(
                        search,
                        &format!("{} {} {}", label(d, o), o.context, o.source),
                    )
                })
                .collect::<Vec<_>>();
            dates.sort_by_key(|o| (o.date, &o.label));
            ui.weak(format!(
                "{} matches · narrow the search to find a date",
                dates.len()
            ));
            for o in dates.into_iter().take(500) {
                let caption = if scope.observations.contains(&o.id) {
                    label(d, o)
                } else {
                    format!("{} (filtered out)", label(d, o))
                };
                ui.selectable_value(id, Some(o.id.clone()), caption);
            }
        });
}

fn number_hit(value: i64, s: &Session) -> bool {
    Feature {
        owner_id: String::new(),
        owner: String::new(),
        domain: "Dates".into(),
        calculation: String::new(),
        unit: "days".into(),
        raw: value as f64,
        explanation: String::new(),
    }
    .matches(&s.query, &s.settings)
}
fn parts_hit(span: &temporal::Span, s: &Session) -> bool {
    [
        (s.settings.years, span.years),
        (s.settings.months, span.months),
        (s.settings.weeks, span.weeks),
        (s.settings.days, span.days),
    ]
    .into_iter()
    .any(|(on, n)| on && number_hit(n, s))
}
fn span_text(span: &temporal::Span, s: &Session, signed: bool) -> String {
    let total = if signed {
        span.sign * span.total_days
    } else {
        span.total_days
    };
    if s.settings.years || s.settings.months || s.settings.weeks {
        format!("{total} days · {}", span.label(&s.settings))
    } else {
        format!("{total} days")
    }
}
fn span_cell(ui: &mut egui::Ui, span: &temporal::Span, s: &Session, signed: bool) {
    let total = if signed {
        span.sign * span.total_days
    } else {
        span.total_days
    };
    ui.label(
        RichText::new(format!("{total} days")).color(if number_hit(total, s) {
            HIT
        } else {
            Color32::WHITE
        }),
    );
    if s.settings.years || s.settings.months || s.settings.weeks {
        let mut shown = span.clone();
        if !signed {
            shown.sign = 1;
        }
        ui.label(
            RichText::new(shown.label(&s.settings))
                .small()
                .color(if parts_hit(span, s) {
                    HIT
                } else {
                    Color32::GRAY
                }),
        );
    }
}
fn relation_hit(r: &temporal::DateRelation, s: &Session) -> bool {
    [Some(&r.original), r.since.as_ref(), Some(&r.until)]
        .into_iter()
        .flatten()
        .any(|span| {
            number_hit(span.total_days, s)
                || number_hit(span.total_days * span.sign, s)
                || parts_hit(span, s)
        })
}

impl AtlasApp {
    pub(super) fn active_calendar_properties(&self, ui: &mut egui::Ui) {
        let s = self.effective_session();
        let facts = analysis::active_date_features(&s);
        ui.strong("Active date · calendar properties");
        ui.columns(4, |columns| {
            for (ui, f) in columns.iter_mut().zip(&facts[..4]) {
                ui.group(|ui| {
                    ui.weak(&f.calculation).on_hover_text(&f.explanation);
                    ui.label(RichText::new(f.displayed(&s.settings)).size(23.).color(
                        if f.matches(&s.query, &s.settings) {
                            HIT
                        } else {
                            ACCENT
                        },
                    ))
                    .on_hover_text(&f.explanation);
                });
            }
        });
        egui::CollapsingHeader::new("More calendar fields")
            .id_salt("active_calendar_details")
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for f in &facts[4..] {
                        ui.group(|ui| {
                            ui.weak(&f.calculation);
                            ui.label(RichText::new(f.displayed(&s.settings)).color(
                                if f.matches(&s.query, &s.settings) {
                                    HIT
                                } else {
                                    Color32::WHITE
                                },
                            ))
                            .on_hover_text(&f.explanation);
                        });
                    }
                });
            });
        ui.separator();
    }
    pub(super) fn date_choices(&mut self, ui: &mut egui::Ui) {
        let d = &self.workspace.datasets[self.dataset_index()];
        let mut chosen = self.workspace.sessions[self.session]
            .active_observation
            .clone();
        let before = chosen.clone();
        ui.horizontal_wrapped(|ui| {
            ui.label("Set sky time from saved date");
            picker(
                ui,
                d,
                &self.scope,
                &mut chosen,
                &mut self.date_search,
                "sky_saved_date",
            );
            let mut dates = d
                .observations
                .iter()
                .filter(|o| self.scope.observations.contains(&o.id))
                .collect::<Vec<_>>();
            dates.sort_by_key(|o| (o.date, &o.label));
            let index = dates.iter().position(|o| Some(&o.id) == chosen.as_ref());
            if ui
                .add_enabled(
                    index.is_some_and(|i| i > 0),
                    egui::Button::new("Previous date"),
                )
                .clicked()
            {
                chosen = index.map(|i| dates[i - 1].id.clone());
            }
            if ui
                .add_enabled(
                    index.map_or(!dates.is_empty(), |i| i + 1 < dates.len()),
                    egui::Button::new("Next date"),
                )
                .clicked()
            {
                chosen = dates.get(index.map_or(0, |i| i + 1)).map(|o| o.id.clone());
            }
            if ui.button("+ New date").clicked() {
                let s = &self.workspace.sessions[self.session];
                self.new_date = NewDate {
                    date: s.active_date.to_string(),
                    timezone: s.timezone.clone(),
                    place_id: s.active_place.clone(),
                    ..Default::default()
                };
                self.show_new_date = true;
            }
        });
        if chosen != before {
            if let Some(o) = d
                .observations
                .iter()
                .find(|o| Some(&o.id) == chosen.as_ref())
                .cloned()
            {
                self.activate_observation(&o);
            }
        }
    }
    pub(super) fn date_scope_summary(&self) -> String {
        let s = &self.workspace.sessions[self.session];
        let d = &self.workspace.datasets[self.dataset_index()];
        let n = analysis::date_observations(d, s, &self.scope).len();
        let pairs = atlas_core::filter::pairs(n);
        format!("Date analysis: {} · {n} dates · {} of {pairs} saved-date pairs, plus each date against the active date. Used by Overview and Comparison.", if s.dates.all_dates {"all filtered dates"} else {"selected dates"}, pairs.min(s.settings.pair_limit as usize))
    }
    pub(super) fn temporal_controls(&mut self, ui: &mut egui::Ui) {
        let di = self.dataset_index();
        let d = &self.workspace.datasets[di];
        let s = &mut self.workspace.sessions[self.session];
        ui.columns(2, |columns| {
            columns[0].strong("Focus date");
            picker(
                &mut columns[0],
                d,
                &self.scope,
                &mut s.dates.focus,
                &mut self.date_search,
                "focus_date",
            );
            columns[1].strong("Compare date");
            picker(
                &mut columns[1],
                d,
                &self.scope,
                &mut s.dates.comparison,
                &mut self.comparison_date_search,
                "comparison_date",
            );
        });
        let mut activate = None;
        ui.horizontal_wrapped(|ui| {
            let mut dates=d.observations.iter().filter(|o|self.scope.observations.contains(&o.id)).collect::<Vec<_>>(); dates.sort_by_key(|o|(o.date,&o.label));
            let index=dates.iter().position(|o|Some(&o.id)==s.dates.focus.as_ref());
            if ui.add_enabled(index.is_some_and(|i|i>0),egui::Button::new("Previous date")).clicked() {s.dates.focus=index.map(|i|dates[i-1].id.clone());}
            if ui.add_enabled(index.map_or(!dates.is_empty(),|i|i+1<dates.len()),egui::Button::new("Next date")).clicked() {s.dates.focus=dates.get(index.map_or(0,|i|i+1)).map(|o|o.id.clone());}
            if ui.button("Swap dates").clicked() {std::mem::swap(&mut s.dates.focus,&mut s.dates.comparison);}
            if ui.add_enabled(s.dates.focus.is_some(),egui::Button::new("Use focus as active")).on_hover_text("Replace the toolbar date/time and observer with this saved event. Selecting a focus alone leaves them unchanged.").clicked() {activate=d.observations.iter().find(|o|Some(&o.id)==s.dates.focus.as_ref()).cloned();}
            if ui.button("+ New date").clicked() {
                self.new_date=NewDate {date:s.active_date.to_string(), timezone:s.timezone.clone(),place_id:s.active_place.clone(),..Default::default()}; self.show_new_date=true;
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (value,name) in [(&mut s.settings.years,"Years"),(&mut s.settings.months,"Months"),(&mut s.settings.weeks,"Weeks"),(&mut s.settings.days,"Days")] {ui.checkbox(value,name);}
            ui.checkbox(&mut s.settings.include_end,"Include end date").on_hover_text("Applies to original date spans. Anniversary countdowns remain civil-day offsets, so an anniversary today is 0 days away.");
            ui.label("Leap day"); ui.selectable_value(&mut s.settings.leap_day,LeapDay::February28,"February 28"); ui.selectable_value(&mut s.settings.leap_day,LeapDay::March1,"March 1");
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("Date analysis");
            ui.selectable_value(&mut s.dates.all_dates, false, "Selected dates only");
            ui.selectable_value(&mut s.dates.all_dates, true, "All filtered dates");
        });
        self.temporal_date_id = s.dates.focus.clone();
        if let Some(o) = activate {
            self.activate_observation(&o);
        }
        let s = self.effective_session();
        ui.label(
            RichText::new(format!(
                "Active reference · {} · {}",
                s.active_date,
                s.active_date.format("%A")
            ))
            .color(ACCENT),
        );
        ui.weak("Set the active date in the toolbar. Focus and comparison choices keep that date/time unchanged. All intervals below use civil calendar dates.");
        ui.separator();
    }

    pub(super) fn temporal_comparison(&self, ui: &mut egui::Ui) {
        let d = &self.workspace.datasets[self.dataset_index()];
        let s = self.effective_session();
        let saved_focus = d
            .observations
            .iter()
            .find(|o| Some(&o.id) == s.dates.focus.as_ref());
        let saved_comparison = d
            .observations
            .iter()
            .find(|o| Some(&o.id) == s.dates.comparison.as_ref());
        for o in [saved_focus, saved_comparison].into_iter().flatten() {
            if !self.scope.observations.contains(&o.id) {
                ui.colored_label(
                    HIT,
                    format!(
                        "{} is hidden by data filters. Clear or adjust filters to compare it.",
                        label(d, o)
                    ),
                );
            }
        }
        let focus = saved_focus.filter(|o| self.scope.observations.contains(&o.id));
        let comparison = saved_comparison.filter(|o| self.scope.observations.contains(&o.id));
        let dates = [
            ("Focus", focus),
            (
                "Compare",
                comparison.filter(|b| focus.is_none_or(|a| a.id != b.id)),
            ),
        ];
        if focus.is_none() && comparison.is_none() {
            ui.label("Choose a focus date to begin. Add a comparison date for a three-date view with the active reference.");
            return;
        }
        ui.strong("Selected dates against the active reference");
        egui::Grid::new("selected_temporal_relations")
            .striped(true)
            .min_col_width(180.)
            .show(ui, |ui| {
                for name in [
                    "Saved date",
                    "Original to active",
                    "Since previous anniversary",
                    "Until next anniversary",
                ] {
                    ui.strong(name);
                }
                ui.end_row();
                for (role, o) in dates {
                    if let Some(o) = o {
                        let r = temporal::relation(o.date, s.active_date, &s.settings);
                        ui.vertical(|ui| {
                            ui.set_width(260.);
                            ui.strong(role);
                            ui.add(egui::Label::new(label(d, o)).wrap())
                                .on_hover_text(format!("{}\n{}", o.context, o.source));
                        });
                        ui.vertical(|ui| span_cell(ui, &r.original, &s, true));
                        ui.vertical(|ui| {
                            if let (Some(date), Some(span)) = (r.previous, r.since.as_ref()) {
                                ui.weak(date.to_string());
                                span_cell(ui, span, &s, false);
                            } else {
                                ui.weak("Not yet occurred");
                            }
                        });
                        ui.vertical(|ui| {
                            ui.weak(r.next.to_string());
                            span_cell(ui, &r.until, &s, false);
                        });
                        ui.end_row();
                    }
                }
            });
        if let (Some(a), Some(b)) = (
            focus,
            comparison.filter(|b| focus.is_some_and(|a| a.id != b.id)),
        ) {
            ui.add_space(8.);
            ui.strong("Between the two selected dates");
            let original = temporal::span(a.date, b.date, &s.settings);
            let anniversaries =
                temporal::anniversary_pair(a.date, b.date, s.active_date, &s.settings);
            ui.columns(3, |columns| {
                for (ui, (caption, span)) in columns.iter_mut().zip([
                    ("Original date gap", Some(&original)),
                    ("Previous-anniversary gap", anniversaries.previous.as_ref()),
                    ("Next-anniversary gap", Some(&anniversaries.next)),
                ]) {
                    ui.group(|ui| {
                        ui.weak(caption);
                        if let Some(span) = span {
                            span_cell(ui, span, &s, false);
                        } else {
                            ui.weak("No prior occurrence for both dates");
                        }
                    });
                }
            });
            ui.weak("Anniversary gaps compare the two previous occurrences, or the two next occurrences, around the active reference. Countdowns exclude end-date adjustment; no recurrence is invented before an event began.");
        }
        if saved_focus.is_some() && s.dates.focus == s.dates.comparison {
            ui.weak("Both selections are the same saved date. Choose a different comparison date for pair gaps.");
        }
    }

    pub(super) fn focus_dataset_browser(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Explore focus against the dataset").id_salt("focus_dataset_relations")
            .default_open(cfg!(debug_assertions) && std::env::var("ATLAS_PREVIEW_VARIANT").as_deref()==Ok("relations")).show(ui,|ui| {
            let di=self.dataset_index(); let d=&self.workspace.datasets[di]; let s=self.effective_session();
            let Some(focus)=d.observations.iter().find(|o|Some(&o.id)==s.dates.focus.as_ref() && self.scope.observations.contains(&o.id)) else {ui.weak("Choose a focus date within the filtered scope first.");return;};
            ui.weak("Each saved date is measured against the focus date, including its anniversaries around that focus. The first row measures the focus against the active date. Choose Compare to bring any row into the three-date view above.");
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.relation_search).id(egui::Id::new("relation_search")).hint_text("Search related dates / teams"));
                ui.checkbox(&mut self.only_date_hits,"Only number matches");
            });
            let mut rows=vec![];
            let active=temporal::relation(focus.date,s.active_date,&s.settings);
            if !self.only_date_hits || relation_hit(&active,&s) {rows.push((None,format!("Focus to active · {}",s.active_date),active));}
            for o in &d.observations {
                if o.id==focus.id || !self.scope.observations.contains(&o.id) || !atlas_core::filter::text_matches(&self.relation_search,&format!("{} {} {}",label(d,o),o.context,o.source)) {continue;}
                let r=temporal::relation(o.date,focus.date,&s.settings);
                if !self.only_date_hits || relation_hit(&r,&s) {rows.push((Some(o.id.clone()),label(d,o),r));}
            }
            ui.weak(format!("{} relationships · saved-date rows point toward focus {}",rows.len(),focus.date));
            let width=ui.available_width(); let widths=[width*0.38,width*0.22,width*0.20,width*0.20];
            ui.horizontal(|ui|{for (w,title) in widths.into_iter().zip(["Related date / direction","Original span","Since previous","Until next"]) {ui.add_sized([w-8.,20.],egui::Label::new(RichText::new(title).strong()).truncate());}});
            let mut compare=None;
            egui::ScrollArea::vertical().id_salt("date_relation_rows").max_height(240.).show_rows(ui,28.,rows.len(),|ui,range| {
                for i in range {let (id,name,r)=&rows[i];
                    ui.horizontal(|ui| {
                        ui.allocate_ui(egui::vec2(widths[0]-8.,24.),|ui| {ui.horizontal(|ui|{
                            if id.is_some() && ui.small_button("Compare").clicked() {compare=id.clone();}
                            ui.add_sized([ui.available_width(),20.],egui::Label::new(name).truncate()).on_hover_text(name);
                        });});
                        for (w,span,date,signed) in [(widths[1],Some(&r.original),None,true),(widths[2],r.since.as_ref(),r.previous,false),(widths[3],Some(&r.until),Some(r.next),false)] {
                            let text=span.map_or_else(||"Not yet occurred".into(),|sp|span_text(sp,&s,signed));
                            let hit=span.is_some_and(|sp|number_hit(if signed {sp.sign*sp.total_days} else {sp.total_days},&s)||parts_hit(sp,&s));
                            ui.add_sized([w-8.,20.],egui::Label::new(RichText::new(&text).color(if hit {HIT} else {Color32::WHITE})).truncate()).on_hover_text(format!("{text}\n{}",date.map_or(String::new(),|date|date.to_string())));
                        }
                    });
                }
            });
            if let Some(id)=compare {self.workspace.sessions[self.session].dates.comparison=Some(id);}
        });
    }
}
