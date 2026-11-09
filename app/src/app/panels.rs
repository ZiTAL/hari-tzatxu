//! Left design/summary/object panel and right tabbed panels.

use super::App;
use crate::lang::{self, t, tf};
use crate::model::{self, El, ShapeKind, StitchKind, HOOPS, RATE_MAX_SPS, RATE_MIN_SPS, THREADS};
use crate::theme::{
    card, hint, kv, section, segmented, ACCENT, DANGER, DIM, FIELD, LINE, MUTED, PANEL, PANEL2,
    TEXT,
};
use crate::ui::{dir_hint, fmt_num, fmt_time, num_field, shape_word};
use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Rounding, Sense, Stroke, Vec2};

impl App {
    // ------------------------------------------------------------- side panels

    pub(crate) fn left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left")
            .exact_width(262.0)
            .resizable(false)
            .frame(
                egui::Frame::none()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(12.0, 6.0)),
            )
            .show(ctx, |ui| {
                section(ui, &t("panel.design"));
                card(ui, PANEL2, |ui| {
                    let mut name = self.project.name.clone();
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut name)
                            .desired_width(f32::INFINITY)
                            .font(FontId::proportional(13.5))
                            .frame(false),
                    );
                    if r.changed() {
                        self.project.name = name;
                        self.dirty = true;
                    }
                    ui.add_space(4.0);
                    let (orig_w, orig_h) = self.project.hoop_size();
                    let mut hw = orig_w;
                    let mut hh = orig_h;
                    let mut preset = self.project.matching_preset();
                    let mut picked_preset = false;
                    ui.label(
                        RichText::new(t("hoop.preset"))
                            .size(10.0)
                            .color(DIM)
                            .extra_letter_spacing(0.8),
                    );
                    let chosen = match preset {
                        Some(i) => HOOPS[i].0.to_string(),
                        None => t("hoop.custom"),
                    };
                    egui::ComboBox::from_id_source("hoop")
                        .width(ui.available_width() - 4.0)
                        .selected_text(RichText::new(chosen).size(12.0))
                        .show_ui(ui, |ui| {
                            for (i, h) in HOOPS.iter().enumerate() {
                                if ui.selectable_label(preset == Some(i), h.0).clicked() {
                                    preset = Some(i);
                                    hw = h.1;
                                    hh = h.2;
                                    picked_preset = true;
                                }
                            }
                        });
                    ui.add_space(3.0);
                    for (label, val) in [(t("hoop.width"), &mut hw), (t("hoop.height"), &mut hh)] {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(label).size(10.5).color(MUTED));
                            let rest = (ui.available_width() - 18.0).max(60.0);
                            num_field(
                                ui,
                                val,
                                model::HOOP_MIN_MM..=model::HOOP_MAX_MM,
                                0.5,
                                rest,
                                " mm",
                                &t("hoop.tip"),
                            );
                        });
                    }
                    let resized = (hw - orig_w).abs() > 1e-4 || (hh - orig_h).abs() > 1e-4;
                    if resized {
                        if !self.hoop_drag {
                            self.push_undo();
                            self.hoop_drag = true;
                        }
                        self.project.set_hoop_size(hw, hh);
                        self.project.hoop = self.project.matching_preset().unwrap_or(0);
                        if picked_preset {
                            self.view.needs_fit = true;
                        }
                        self.rebuild();
                    }
                    if self.hoop_drag && !ui.ctx().input(|i| i.pointer.any_down()) {
                        self.hoop_drag = false;
                    }
                });

                ui.add_space(6.0);
                section(ui, &t("panel.summary"));
                let st = self.stats;
                card(ui, PANEL2, |ui| {
                    kv(
                        ui,
                        &t("summary.size"),
                        &format!("{:.1} × {:.1} mm", st.w, st.h),
                    );
                    kv(
                        ui,
                        &t("summary.size_cm"),
                        &format!("{:.1} × {:.1} cm", st.w / 10.0, st.h / 10.0),
                    );
                    kv(ui, &t("summary.stitches"), &fmt_num(st.stitches));
                    kv(ui, &t("summary.colours"), &format!("{}", st.colors));
                    kv(ui, &t("summary.time"), &fmt_time(st.minutes));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(t("summary.rate")).size(11.0).color(MUTED));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let mut rate = self.project.stitches_per_second();
                            let unit = t("unit.sps");
                            let r = num_field(
                                ui,
                                &mut rate,
                                RATE_MIN_SPS..=RATE_MAX_SPS,
                                0.1,
                                52.0,
                                &format!(" {unit}"),
                                &t("summary.rate_tip"),
                            );
                            if r.changed() {
                                self.project.set_stitches_per_second(rate);
                                self.rebuild();
                            }
                        });
                    });
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(tf(
                            "objects.header",
                            &[("n", &self.project.objects.len().to_string())],
                        ))
                        .size(10.5)
                        .color(MUTED)
                        .extra_letter_spacing(1.1),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(t("objects.delete")).size(10.5).color(DANGER),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            self.delete_selected();
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(t("objects.duplicate"))
                                        .size(10.5)
                                        .color(MUTED),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            self.duplicate();
                        }
                    });
                });
                ui.add_space(2.0);

                enum RowAction {
                    Select(u32),
                    Toggle(u32),
                    Up(u32),
                    Down(u32),
                }
                let mut action: Option<RowAction> = None;
                egui::Frame::none()
                    .fill(FIELD)
                    .rounding(Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(4.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .max_height((ui.available_height() - 96.0).max(120.0))
                            .show(ui, |ui| {
                                if self.project.objects.is_empty() {
                                    ui.add_space(14.0);
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            RichText::new(t("objects.empty.title"))
                                                .size(12.0)
                                                .color(DIM),
                                        );
                                        ui.label(
                                            RichText::new(t("objects.empty.hint"))
                                                .size(11.0)
                                                .color(DIM),
                                        );
                                        ui.add_space(6.0);
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new(t("objects.empty.example"))
                                                        .size(10.5)
                                                        .color(ACCENT),
                                                )
                                                .fill(FIELD)
                                                .stroke(Stroke::new(1.0_f32, LINE))
                                                .min_size(Vec2::new(150.0, 24.0)),
                                            )
                                            .clicked()
                                        {
                                            self.load_example();
                                        }
                                    });
                                    ui.add_space(14.0);
                                }
                                let rows: Vec<(u32, String, String, usize, bool, bool)> = self
                                    .project
                                    .objects
                                    .iter()
                                    .enumerate()
                                    .map(|(_i, o)| {
                                        (
                                            o.id,
                                            format!("{} {}", o.kind.label(), shape_word(o)),
                                            format!(
                                                "{:.0} × {:.0} mm · {}",
                                                o.w,
                                                o.h,
                                                lang::thread_name(o.color.min(THREADS.len() - 1))
                                            ),
                                            o.color,
                                            o.visible,
                                            self.sel == Some(o.id),
                                        )
                                    })
                                    .collect();
                                for (id, title, sub, color, visible, selected) in rows {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        Vec2::new(ui.available_width(), 34.0),
                                        Sense::click(),
                                    );
                                    let p = ui.painter();
                                    if selected {
                                        p.rect_filled(
                                            rect,
                                            Rounding::same(6.0),
                                            Color32::from_rgb(0x2b, 0x34, 0x3f),
                                        );
                                    } else if resp.hovered() {
                                        p.rect_filled(
                                            rect,
                                            Rounding::same(6.0),
                                            Color32::from_rgb(0x22, 0x28, 0x31),
                                        );
                                    }
                                    let sw = Rect::from_min_size(
                                        rect.left_top() + Vec2::new(8.0, 10.0),
                                        Vec2::new(13.0, 13.0),
                                    );
                                    p.rect_filled(
                                        sw,
                                        Rounding::same(3.0),
                                        model::thread_color(color),
                                    );
                                    p.rect_stroke(
                                        sw,
                                        Rounding::same(3.0),
                                        Stroke::new(1.0_f32, Color32::from_black_alpha(60)),
                                    );
                                    let tx = rect.left() + 28.0;
                                    p.text(
                                        Pos2::new(tx, rect.top() + 9.0),
                                        Align2::LEFT_CENTER,
                                        &title,
                                        FontId::proportional(12.0),
                                        if visible { TEXT } else { DIM },
                                    );
                                    p.text(
                                        Pos2::new(tx, rect.top() + 24.0),
                                        Align2::LEFT_CENTER,
                                        &sub,
                                        FontId::proportional(10.0),
                                        DIM,
                                    );
                                    // eye dot
                                    let eye = Rect::from_center_size(
                                        Pos2::new(rect.right() - 14.0, rect.center().y),
                                        Vec2::new(18.0, 18.0),
                                    );
                                    let eye_resp =
                                        ui.interact(eye, ui.id().with(("eye", id)), Sense::click());
                                    let ec = if visible { MUTED } else { DIM };
                                    p.circle_stroke(eye.center(), 4.0, Stroke::new(1.2_f32, ec));
                                    if visible {
                                        p.circle_filled(eye.center(), 1.8, ec);
                                    }
                                    if eye_resp.clicked() {
                                        action = Some(RowAction::Toggle(id));
                                    } else if resp.clicked() {
                                        action = Some(RowAction::Select(id));
                                    }
                                }
                            });
                    });
                // move buttons under the list
                ui.horizontal(|ui| {
                    let sel = self.sel;
                    if ui
                        .add_enabled(
                            sel.is_some(),
                            egui::Button::new(RichText::new(t("objects.move_up")).size(10.5))
                                .min_size(Vec2::new(78.0, 22.0)),
                        )
                        .clicked()
                    {
                        if let Some(id) = sel {
                            action = Some(RowAction::Up(id));
                        }
                    }
                    if ui
                        .add_enabled(
                            sel.is_some(),
                            egui::Button::new(RichText::new(t("objects.move_down")).size(10.5))
                                .min_size(Vec2::new(78.0, 22.0)),
                        )
                        .clicked()
                    {
                        if let Some(id) = sel {
                            action = Some(RowAction::Down(id));
                        }
                    }
                });
                match action {
                    Some(RowAction::Select(id)) => self.sel = Some(id),
                    Some(RowAction::Toggle(id)) => {
                        self.push_undo();
                        if let Some(o) = self.project.get_mut(id) {
                            o.visible = !o.visible;
                        }
                        self.rebuild();
                    }
                    Some(RowAction::Up(id)) => self.move_obj(id, -1),
                    Some(RowAction::Down(id)) => self.move_obj(id, 1),
                    None => {}
                }
            });
    }

    pub(crate) fn right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right")
            .exact_width(292.0)
            .resizable(false)
            .frame(
                egui::Frame::none()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0)),
            )
            .show(ctx, |ui| {
                let mut tab = self.tab;
                let labels = [t("tab.threads"), t("tab.settings"), t("tab.plan")];
                if segmented(
                    ui,
                    &mut tab,
                    &[labels[0].as_str(), labels[1].as_str(), labels[2].as_str()],
                ) {
                    self.tab = tab;
                }
                ui.add_space(8.0);
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.tab {
                        0 => self.threads_tab(ui),
                        1 => self.settings_tab(ui),
                        _ => self.plan_tab(ui),
                    });
            });
    }

    fn threads_tab(&mut self, ui: &mut egui::Ui) {
        let sel_color = self.selected_obj().map(|o| o.color);
        ui.horizontal(|ui| {
            ui.label(RichText::new(t("thread.colour")).size(12.0).color(MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(c) = sel_color {
                    let name = lang::thread_name(c.min(THREADS.len() - 1));
                    ui.label(RichText::new(name).size(12.5).color(TEXT));
                    let (r, _) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), Sense::hover());
                    ui.painter()
                        .rect_filled(r, Rounding::same(3.0), model::thread_color(c));
                } else {
                    ui.label(RichText::new(t("thread.none")).size(11.0).color(DIM));
                }
            });
        });
        ui.add_space(6.0);
        egui::Frame::none()
            .fill(FIELD)
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                let mut picked: Option<usize> = None;
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                    for i in 0..THREADS.len() {
                        let name = lang::thread_name(i);
                        let (r, resp) =
                            ui.allocate_exact_size(Vec2::new(28.0, 28.0), Sense::click());
                        let p = ui.painter();
                        p.rect_filled(r.shrink(1.0), Rounding::same(5.0), model::thread_color(i));
                        if sel_color == Some(i) {
                            p.rect_stroke(
                                r,
                                Rounding::same(5.0),
                                Stroke::new(2.0_f32, Color32::WHITE),
                            );
                        } else if resp.hovered() {
                            p.rect_stroke(
                                r,
                                Rounding::same(5.0),
                                Stroke::new(1.0_f32, Color32::from_white_alpha(120)),
                            );
                        }
                        if resp.on_hover_text(&name).clicked() {
                            picked = Some(i);
                        }
                    }
                });
                if let Some(c) = picked {
                    if let Some(id) = self.sel {
                        self.push_undo();
                        if let Some(o) = self.project.get_mut(id) {
                            o.color = c;
                        }
                        self.rebuild();
                    } else {
                        self.say(t("thread.pick_first"));
                    }
                }
            });

        ui.add_space(10.0);
        section(ui, &t("panel.stitch_plan"));
        let mut order: Vec<(usize, usize)> = Vec::new();
        for step in &self.seq {
            if let Some(last) = order.last_mut() {
                if last.0 == step.color {
                    last.1 += 1;
                    continue;
                }
            }
            order.push((step.color, 1));
        }
        if order.is_empty() {
            hint(ui, &t("common.no_stitches"));
        }
        for (i, (c, n)) in order.iter().enumerate() {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), Sense::hover());
                ui.painter()
                    .rect_filled(r, Rounding::same(3.0), model::thread_color(*c));
                ui.label(RichText::new(format!("{}. {}", i + 1, lang::thread_name(*c))).size(11.5));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(tf("plan.st", &[("n", &fmt_num(*n))]))
                            .size(10.5)
                            .color(DIM),
                    );
                });
            });
        }
    }

    fn settings_tab(&mut self, ui: &mut egui::Ui) {
        let Some(id) = self.sel else {
            ui.add_space(8.0);
            hint(ui, &t("settings.select"));
            ui.add_space(8.0);
            dir_hint(ui);
            return;
        };
        let (mut kind, mut density, mut slen, mut satin_w, mut rot, mut cx, mut cy, mut w, mut h) = {
            let Some(o) = self.project.get(id) else {
                hint(ui, &t("settings.gone"));
                return;
            };
            (
                o.kind,
                o.density,
                o.stitch_len,
                o.satin_w,
                o.rot,
                o.cx,
                o.cy,
                o.w,
                o.h,
            )
        };
        let imported = self.project.get(id).map(|o| o.shape) == Some(ShapeKind::Stitches);
        let is_path = matches!(
            self.project.get(id).map(|o| o.shape),
            Some(ShapeKind::Path) | Some(ShapeKind::Stitches)
        );
        let mut changed = false;
        let mut began = false;
        let mut ended = false;
        if !imported {
            card(ui, PANEL2, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(t("stitch.type")).size(11.5).color(MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for (k, label) in [
                            (StitchKind::Run, t("stitch.run")),
                            (StitchKind::Satin, t("stitch.satin")),
                            (StitchKind::Fill, t("stitch.fill")),
                        ] {
                            let on = kind == k;
                            if ui
                                .add(
                                    egui::Button::new(RichText::new(label).size(10.5).color(
                                        if on {
                                            Color32::from_rgb(0x0d, 0x27, 0x21)
                                        } else {
                                            MUTED
                                        },
                                    ))
                                    .fill(if on { ACCENT } else { FIELD })
                                    .stroke(Stroke::new(1.0_f32, if on { ACCENT } else { LINE }))
                                    .min_size(Vec2::new(44.0, 20.0)),
                                )
                                .clicked()
                                && !on
                            {
                                kind = k;
                                changed = true;
                            }
                        }
                    });
                });
            });
            ui.add_space(8.0);
            section(ui, &t("panel.placement"));
            let r = ui.add(
                egui::Slider::new(&mut rot, -3.14..=3.14)
                    .text(t("prop.rotation"))
                    .fixed_decimals(2),
            );
            began |= r.drag_started();
            ended |= r.drag_stopped();
            changed |= r.changed();
            let r = ui.add(
                egui::Slider::new(&mut cx, -130.0..=130.0)
                    .text(t("prop.across"))
                    .suffix(" mm")
                    .fixed_decimals(1),
            );
            began |= r.drag_started();
            ended |= r.drag_stopped();
            changed |= r.changed();
            let r = ui.add(
                egui::Slider::new(&mut cy, -190.0..=190.0)
                    .text(t("prop.up"))
                    .suffix(" mm")
                    .fixed_decimals(1),
            );
            began |= r.drag_started();
            ended |= r.drag_stopped();
            changed |= r.changed();
            if !is_path {
                let r = ui.add(
                    egui::Slider::new(&mut w, 3.0..=200.0)
                        .text(t("prop.width"))
                        .suffix(" mm")
                        .fixed_decimals(1),
                );
                began |= r.drag_started();
                ended |= r.drag_stopped();
                changed |= r.changed();
                let r = ui.add(
                    egui::Slider::new(&mut h, 3.0..=200.0)
                        .text(t("prop.height"))
                        .suffix(" mm")
                        .fixed_decimals(1),
                );
                began |= r.drag_started();
                ended |= r.drag_stopped();
                changed |= r.changed();
            }

            ui.add_space(8.0);
            section(ui, &t("panel.stitches"));
            let mut d = density;
            let rd = ui.add(
                egui::Slider::new(&mut d, 0.3..=1.6)
                    .text(t("prop.spacing"))
                    .suffix(" mm")
                    .fixed_decimals(2),
            );
            began |= rd.drag_started();
            ended |= rd.drag_stopped();
            if rd.changed() {
                density = d;
                changed = true;
            }
            let mut sl = slen;
            let rs = ui.add(
                egui::Slider::new(&mut sl, 1.0..=5.0)
                    .text(t("prop.stitch_len"))
                    .suffix(" mm")
                    .fixed_decimals(1),
            );
            began |= rs.drag_started();
            ended |= rs.drag_stopped();
            if rs.changed() {
                slen = sl;
                changed = true;
            }
            if kind == StitchKind::Satin {
                let mut sw = satin_w;
                let rw = ui.add(
                    egui::Slider::new(&mut sw, 1.5..=15.0)
                        .text(t("prop.column_width"))
                        .suffix(" mm")
                        .fixed_decimals(1),
                );
                began |= rw.drag_started();
                ended |= rw.drag_stopped();
                if rw.changed() {
                    satin_w = sw;
                    changed = true;
                }
            }
        } else {
            ui.add_space(8.0);
            hint(ui, &t("settings.imported"));
        }
        if began && !self.prop_drag {
            self.push_undo();
            self.prop_drag = true;
        }
        if changed {
            if !self.prop_drag {
                self.push_undo();
            }
            if let Some(o) = self.project.get_mut(id) {
                let (ocx, ocy) = (o.cx, o.cy);
                if is_path && !o.path.is_empty() {
                    let dx = cx - ocx;
                    let dy = cy - ocy;
                    if dx.abs() > 1e-5 || dy.abs() > 1e-5 {
                        for p in o.path.iter_mut() {
                            p[0] += dx;
                            p[1] += dy;
                        }
                    }
                }
                o.kind = kind;
                o.density = density;
                o.stitch_len = slen;
                o.satin_w = satin_w;
                o.rot = rot;
                o.cx = cx;
                o.cy = cy;
                o.w = w;
                o.h = h;
            }
            self.rebuild();
        }
        if ended || (self.prop_drag && !ui.ctx().input(|i| i.pointer.any_down())) {
            self.prop_drag = false;
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button(t("action.centre")).clicked() {
                self.centre_selected();
            }
            if ui.button(t("action.delete")).clicked() {
                self.delete_selected();
            }
        });
    }

    fn plan_tab(&mut self, ui: &mut egui::Ui) {
        section(ui, &t("plan.sequence"));
        card(ui, PANEL2, |ui| {
            kv(ui, &t("summary.stitches"), &fmt_num(self.stats.stitches));
            kv(ui, &t("plan.blocks"), &format!("{}", self.stats.colors));
            kv(ui, &t("plan.time"), &fmt_time(self.stats.minutes));
            let (hw, hh) = self.project.hoop_size();
            kv(
                ui,
                &t("plan.hoop"),
                &format!("{} × {} mm", model::fmt_mm(hw), model::fmt_mm(hh)),
            );
            kv(
                ui,
                &t("summary.rate"),
                &format!(
                    "{:.1} {}",
                    self.project.stitches_per_second(),
                    t("unit.sps")
                ),
            );
        });
        ui.add_space(8.0);
        section(ui, &t("plan.order"));
        let mut groups: Vec<(usize, usize, usize)> = Vec::new(); // color, stitches, first obj
        let mut last_color: Option<usize> = None;
        for st in &self.seq {
            if let El::Stitch(_) = st.el {
                if last_color == Some(st.color) {
                    if let Some(g) = groups.last_mut() {
                        g.1 += 1;
                    }
                } else {
                    groups.push((st.color, 1, st.obj as usize));
                    last_color = Some(st.color);
                }
            }
        }
        if groups.is_empty() {
            hint(ui, &t("common.no_stitches"));
        }
        for (i, (c, n, _)) in groups.iter().enumerate() {
            let (rect, resp) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), Sense::click());
            let p = ui.painter();
            if resp.hovered() {
                p.rect_filled(
                    rect,
                    Rounding::same(4.0),
                    Color32::from_rgb(0x24, 0x2a, 0x33),
                );
            }
            let sw =
                Rect::from_min_size(rect.left_top() + Vec2::new(4.0, 7.0), Vec2::new(12.0, 12.0));
            p.rect_filled(sw, Rounding::same(3.0), model::thread_color(*c));
            p.text(
                Pos2::new(rect.left() + 24.0, rect.center().y),
                Align2::LEFT_CENTER,
                format!("{}. {}", i + 1, lang::thread_name(*c)),
                FontId::proportional(11.5),
                TEXT,
            );
            p.text(
                Pos2::new(rect.right() - 6.0, rect.center().y),
                Align2::RIGHT_CENTER,
                tf("plan.st", &[("n", &fmt_num(*n))]),
                FontId::proportional(10.5),
                DIM,
            );
        }
        ui.add_space(10.0);
        if ui.button(t("plan.reduce")).clicked() {
            self.optimise_order();
        }
    }
}
