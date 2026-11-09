//! Top bar, menus, toolbar and bottom transport/status bars.

use super::{App, Tool};
use crate::lang::{self, t, tf};
use crate::model::{RATE_MAX_SPS, RATE_MIN_SPS};
use crate::theme::{
    icon_button, Icon, ACCENT, ACCENT_DIM, BG, DIM, LINE, LINE_SOFT, MUTED, OK, PANEL, PANEL2,
    TEXT, WARN,
};
use crate::ui::{fmt_num, num_field, slug};
use crate::APP_TITLE;
use eframe::egui;
use egui::{Color32, Rect, RichText, Rounding, Sense, Stroke, Vec2};

impl App {
    // ------------------------------------------------------------------ shell

    pub(crate) fn chrome(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("chrome")
            .frame(
                egui::Frame::none()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(12.0, 0.0))
                    .stroke(Stroke::new(0.0_f32, LINE_SOFT)),
            )
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(Vec2::new(30.0, 30.0), Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(r, Rounding::same(8.0), ACCENT_DIM);
                    p.circle_filled(r.center(), 3.0, Color32::from_rgb(0xe9, 0xfb, 0xf3));
                    p.line_segment(
                        [
                            r.center() + Vec2::new(6.0, -6.0),
                            r.center() + Vec2::new(-6.0, 9.0),
                        ],
                        Stroke::new(1.6_f32, Color32::from_rgb(0xe9, 0xfb, 0xf3)),
                    );
                    ui.add_space(2.0);
                    ui.vertical(|ui| {
                        ui.add_space(1.0);
                        ui.label(RichText::new(APP_TITLE).size(14.5).color(TEXT));
                        ui.label(RichText::new(t("app.tagline")).size(8.5).color(DIM));
                    });
                    ui.add_space(18.0);
                    self.menu_bar(ui);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        self.language_picker(ui, ctx);
                    });
                });
                ui.add_space(5.0);
                self.toolbar(ui);
                ui.add_space(6.0);
            });
    }

    fn language_picker(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let current = lang::code();
        let broken = lang::load_error();
        let shown = if broken.is_some() {
            RichText::new(format!("{} !", lang::name()))
                .size(11.5)
                .color(WARN)
        } else {
            RichText::new(lang::name()).size(11.5).color(TEXT)
        };
        let combo = egui::ComboBox::from_id_source("language")
            .width(108.0)
            .selected_text(shown)
            .show_ui(ui, |ui| {
                for (code, name) in lang::available() {
                    let picked = ui
                        .selectable_label(current == *code, name.as_str())
                        .clicked();
                    if picked && current != *code {
                        lang::set(code);
                        lang::save_language(code);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                            "{APP_TITLE} — {}",
                            t("app.window")
                        )));
                    }
                }
            });
        if let Some(err) = broken {
            combo.response.on_hover_text(err);
        }
        ui.label(RichText::new(t("menu.language")).size(11.0).color(MUTED));
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button(t("menu.file"), |ui| {
                if ui.button(t("menu.file.new")).clicked() {
                    self.new_project();
                    ui.close_menu();
                }
                if ui.button(t("menu.file.example")).clicked() {
                    self.load_example();
                    ui.close_menu();
                }
                if ui.button(t("menu.file.open")).clicked() {
                    self.ask_open_dialog(&t("title.open_design"), "json", &t("filter.designs"));
                    ui.close_menu();
                }
                if ui.button(t("menu.file.import")).clicked() {
                    self.ask_open_dialog(&t("title.import"), "dst", &t("filter.dst"));
                    ui.close_menu();
                }
                if ui.button(t("menu.file.save")).clicked() {
                    self.save_as();
                    ui.close_menu();
                }
                if ui.button(t("menu.file.save_as")).clicked() {
                    self.ask_save_dialog(
                        &t("title.save_design"),
                        "json",
                        &t("filter.designs"),
                        &format!("{}.json", slug(&self.project.name)),
                    );
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t("menu.file.export_dst")).clicked() {
                    self.ask_save_dialog(
                        &t("title.export_dst"),
                        "dst",
                        &t("filter.dst"),
                        &format!("{}.dst", slug(&self.project.name)),
                    );
                    ui.close_menu();
                }
                if ui.button(t("menu.file.export_svg")).clicked() {
                    self.ask_save_dialog(
                        &t("title.export_svg"),
                        "svg",
                        &t("filter.svg"),
                        &format!("{}.svg", slug(&self.project.name)),
                    );
                    ui.close_menu();
                }
                if ui.button(t("menu.file.export_csv")).clicked() {
                    self.ask_save_dialog(
                        &t("title.export_csv"),
                        "csv",
                        &t("filter.csv"),
                        &format!("{}-stitches.csv", slug(&self.project.name)),
                    );
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t("menu.file.quit")).clicked() {
                    self.quit = true;
                    ui.close_menu();
                }
            });
            ui.menu_button(t("menu.edit"), |ui| {
                if ui.button(t("menu.edit.undo")).clicked() {
                    self.do_undo();
                    ui.close_menu();
                }
                if ui.button(t("menu.edit.redo")).clicked() {
                    self.do_redo();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t("menu.edit.duplicate")).clicked() {
                    self.duplicate();
                    ui.close_menu();
                }
                if ui.button(t("menu.edit.delete")).clicked() {
                    self.delete_selected();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t("menu.edit.centre")).clicked() {
                    self.centre_selected();
                    ui.close_menu();
                }
            });
            ui.menu_button(t("menu.view"), |ui| {
                ui.checkbox(&mut self.show_stitches, t("menu.view.stitches"));
                ui.checkbox(&mut self.show_grid, t("menu.view.grid"));
                ui.checkbox(&mut self.show_rulers, t("menu.view.rulers"));
                ui.checkbox(&mut self.snap, t("menu.view.snap"));
                ui.separator();
                if ui.button(t("menu.view.fit")).clicked() {
                    self.view.needs_fit = true;
                    ui.close_menu();
                }
            });
            ui.menu_button(t("menu.stitch"), |ui| {
                let play = if self.sim.playing {
                    t("menu.stitch.pause")
                } else {
                    t("menu.stitch.play")
                };
                if ui.button(play).clicked() {
                    self.sim.playing = !self.sim.playing;
                    ui.close_menu();
                }
                if ui.button(t("menu.stitch.reset")).clicked() {
                    self.sim.idx = 0;
                    self.sim.playing = false;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t("menu.stitch.reduce")).clicked() {
                    self.optimise_order();
                    ui.close_menu();
                }
            });
            ui.menu_button(t("menu.help"), |ui| {
                if ui.button(t("menu.help.how")).clicked() {
                    self.help_open = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t("menu.help.about")).clicked() {
                    self.about_open = true;
                    ui.close_menu();
                }
            });
        });
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let tools = [
                (Tool::Select, Icon::Select, t("tool.select")),
                (Tool::FillRect, Icon::Rect, t("tool.rect")),
                (Tool::FillEllipse, Icon::Ellipse, t("tool.ellipse")),
                (Tool::RunPath, Icon::Path, t("tool.run")),
                (Tool::SatinPath, Icon::Satin, t("tool.satin")),
                (Tool::Pan, Icon::Pan, t("tool.pan")),
            ];
            for (t, icon, tip) in tools {
                if icon_button(ui, icon, &tip, self.tool == t, true).clicked() {
                    self.tool = t;
                }
            }
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(2.0);
            let can_undo = !self.undo.is_empty();
            let can_redo = !self.redo.is_empty();
            if icon_button(ui, Icon::Undo, &t("toolbar.undo"), false, can_undo).clicked() {
                self.do_undo();
            }
            if icon_button(ui, Icon::Redo, &t("toolbar.redo"), false, can_redo).clicked() {
                self.do_redo();
            }
            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);
            if icon_button(ui, Icon::Fit, &t("toolbar.fit"), false, true).clicked() {
                self.view.needs_fit = true;
            }
            if icon_button(ui, Icon::Grid, &t("toolbar.grid"), self.show_grid, true).clicked() {
                self.show_grid = !self.show_grid;
            }
            if icon_button(
                ui,
                Icon::Ruler,
                &t("toolbar.rulers"),
                self.show_rulers,
                true,
            )
            .clicked()
            {
                self.show_rulers = !self.show_rulers;
            }
            if icon_button(
                ui,
                Icon::Needle,
                &t("toolbar.stitches"),
                self.show_stitches,
                true,
            )
            .clicked()
            {
                self.show_stitches = !self.show_stitches;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let z = self.view.zoom;
                ui.label(
                    RichText::new(format!("{:.0}%", z / 3.78 * 100.0))
                        .size(11.5)
                        .color(MUTED),
                );
                if ui
                    .small_button("+")
                    .on_hover_text(t("toolbar.zoom_in"))
                    .clicked()
                {
                    self.view.zoom = (self.view.zoom * 1.2).min(40.0);
                }
                if ui
                    .small_button("−")
                    .on_hover_text(t("toolbar.zoom_out"))
                    .clicked()
                {
                    self.view.zoom = (self.view.zoom / 1.2).max(0.5);
                }
                ui.add_space(10.0);
                let fits = self.fits_hoop();
                let (txt, col) = if fits {
                    (t("toolbar.fits"), OK)
                } else {
                    (t("toolbar.overflows"), WARN)
                };
                ui.label(RichText::new(txt).size(11.0).color(col));
            });
        });
    }

    pub(crate) fn fits_hoop(&self) -> bool {
        let (hw, hh) = self.project.hoop_size();
        match self.project.bounds() {
            Some(b) => b.0 >= -hw / 2.0 && b.2 <= hw / 2.0 && b.1 >= -hh / 2.0 && b.3 <= hh / 2.0,
            None => true,
        }
    }

    // --------------------------------------------------------------- bottom bar

    pub(crate) fn bottom(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("transport")
            .exact_height(54.0)
            .frame(
                egui::Frame::none()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0))
                    .stroke(Stroke::new(1.0_f32, LINE_SOFT)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let playing = self.sim.playing;
                    let (r, resp) = ui.allocate_exact_size(Vec2::new(30.0, 30.0), Sense::click());
                    let p = ui.painter();
                    p.circle_filled(
                        r.center(),
                        14.0,
                        if playing {
                            ACCENT
                        } else {
                            Color32::from_rgb(0x2f, 0x37, 0x42)
                        },
                    );
                    if playing {
                        p.rect_filled(
                            Rect::from_center_size(
                                r.center() + Vec2::new(-4.0, 0.0),
                                Vec2::new(3.0, 12.0),
                            ),
                            Rounding::ZERO,
                            Color32::from_rgb(0x0d, 0x27, 0x21),
                        );
                        p.rect_filled(
                            Rect::from_center_size(
                                r.center() + Vec2::new(3.0, 0.0),
                                Vec2::new(3.0, 12.0),
                            ),
                            Rounding::ZERO,
                            Color32::from_rgb(0x0d, 0x27, 0x21),
                        );
                    } else {
                        p.add(egui::Shape::convex_polygon(
                            vec![
                                r.center() + Vec2::new(-4.0, -7.0),
                                r.center() + Vec2::new(7.0, 0.0),
                                r.center() + Vec2::new(-4.0, 7.0),
                            ],
                            Color32::from_rgb(0xd6, 0xe2, 0xea),
                            Stroke::NONE,
                        ));
                    }
                    if resp.clicked() {
                        if self.sim.idx >= self.seq.len() {
                            self.sim.idx = 0;
                        }
                        self.sim.playing = !self.sim.playing;
                    }
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(t("sim.reset")).size(10.5).color(MUTED),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        self.sim.idx = 0;
                        self.sim.playing = false;
                    }
                    ui.add_space(8.0);
                    let len = self.seq.len().max(1);
                    let mut idx = self.sim.idx.min(len);
                    let avail = (ui.available_width() - 400.0).max(120.0);
                    let resp = ui.add_sized(
                        Vec2::new(avail, 22.0),
                        egui::Slider::new(&mut idx, 0..=len).show_value(false),
                    );
                    if resp.changed() {
                        self.sim.idx = idx;
                        self.sim.playing = false;
                    }
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(format!(
                            "{} / {}",
                            fmt_num(self.sim.idx),
                            fmt_num(self.seq.len())
                        ))
                        .size(11.0)
                        .color(MUTED)
                        .monospace(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(t("sim.speed")).size(10.0).color(DIM));
                        for (label, value, tip) in [
                            (t("sim.max"), 80.0_f32, t("sim.max_tip")),
                            ("10×".to_string(), 10.0, t("sim.x10_tip")),
                            ("1×".to_string(), 1.0, t("sim.x1_tip")),
                        ] {
                            let active = (self.sim.mult - value).abs() < 0.05;
                            let btn = egui::Button::new(RichText::new(label).size(10.0).color(
                                if active {
                                    Color32::from_rgb(0x0d, 0x27, 0x21)
                                } else {
                                    MUTED
                                },
                            ))
                            .min_size(Vec2::new(30.0, 20.0))
                            .fill(if active { ACCENT } else { PANEL2 })
                            .stroke(Stroke::new(1.0_f32, if active { ACCENT } else { LINE }));
                            if ui.add(btn).on_hover_text(tip).clicked() {
                                self.sim.mult = value;
                                self.sim.acc = 0.0;
                            }
                        }
                        ui.add_space(6.0);
                        let mut rate = self.project.stitches_per_second();
                        let unit = t("unit.sps");
                        let r = num_field(
                            ui,
                            &mut rate,
                            RATE_MIN_SPS..=RATE_MAX_SPS,
                            0.1,
                            50.0,
                            &format!(" {unit}"),
                            &t("summary.rate_tip"),
                        );
                        if r.changed() {
                            self.project.set_stitches_per_second(rate);
                            self.rebuild();
                        }
                        ui.label(RichText::new(t("summary.rate")).size(10.0).color(MUTED));
                    });
                });
            });

        egui::TopBottomPanel::bottom("status")
            .exact_height(24.0)
            .frame(
                egui::Frame::none()
                    .fill(BG)
                    .inner_margin(egui::Margin::symmetric(16.0, 0.0))
                    .stroke(Stroke::new(1.0_f32, LINE_SOFT)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(self.tool.hint()).size(10.5).color(DIM));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let hoop = self.project.hoop_name();
                        ui.label(RichText::new(hoop).size(10.0).color(DIM));
                        ui.separator();
                        ui.label(
                            RichText::new(tf(
                                "status.objects",
                                &[("n", &self.project.objects.len().to_string())],
                            ))
                            .size(10.0)
                            .color(DIM),
                        );
                        ui.separator();
                        let (dot, _dot_resp) =
                            ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                        ui.painter().circle_filled(dot.center(), 3.0, OK);
                        ui.label(RichText::new(t("status.offline")).size(10.0).color(MUTED))
                            .on_hover_text(t("status.offline_tip"));
                    });
                });
            });
    }
}
