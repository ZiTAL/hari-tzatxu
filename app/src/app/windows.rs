//! Help and about windows.

use super::App;
use crate::lang::t;
use crate::theme::{kv, section, DIM, MUTED, TEXT};
use crate::APP_TITLE;
use eframe::egui;
use egui::{Align2, Color32, RichText, Vec2};

impl App {
    // ------------------------------------------------------------- help window

    pub(crate) fn help_window(&mut self, ctx: &egui::Context) {
        let mut open = self.help_open;
        egui::Window::new(t("help.title"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(480.0)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(520.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.label(RichText::new(t("help.heading")).size(15.0).color(TEXT));
                        ui.add_space(6.0);
                        for line in [t("help.p1"), t("help.p2")] {
                            ui.label(RichText::new(line).size(11.5).color(MUTED));
                            ui.add_space(4.0);
                        }
                        ui.add_space(6.0);
                        section(ui, &t("menu.file.example"));
                        ui.label(RichText::new(t("help.example")).size(11.5).color(MUTED));
                        ui.add_space(8.0);
                        section(ui, &t("help.tools"));
                        for (k, v) in [
                            (t("tool.select"), t("help.tool.select")),
                            (t("tool.rect"), t("help.tool.rect")),
                            (t("tool.ellipse"), t("help.tool.ellipse")),
                            (t("tool.run"), t("help.tool.run")),
                            (t("tool.satin"), t("help.tool.satin")),
                            (t("tool.pan"), t("help.tool.pan")),
                        ] {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(k)
                                        .size(11.0)
                                        .color(Color32::from_rgb(0x9d, 0xd8, 0xc7)),
                                );
                                ui.label(RichText::new(v).size(11.0).color(MUTED));
                            });
                        }
                        ui.add_space(8.0);
                        section(ui, &t("help.rulers.heading"));
                        ui.label(RichText::new(t("help.rulers")).size(11.5).color(MUTED));
                        ui.add_space(8.0);
                        section(ui, &t("help.sizes.heading"));
                        ui.label(RichText::new(t("help.sizes")).size(11.5).color(MUTED));
                        ui.add_space(8.0);
                        section(ui, &t("help.files"));
                        ui.label(RichText::new(t("help.files.text")).size(11.5).color(MUTED));
                        ui.add_space(8.0);
                        section(ui, &t("help.built"));
                        ui.label(RichText::new(t("help.built.text")).size(11.5).color(MUTED));
                        ui.add_space(8.0);
                        section(ui, &t("menu.language"));
                        ui.label(RichText::new(t("help.langs")).size(11.5).color(MUTED));
                    });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(t("help.close")).clicked() {
                        self.help_open = false;
                    }
                    if ui.button(t("help.more")).clicked() {
                        self.help_open = false;
                        self.about_open = true;
                    }
                });
            });
        self.help_open = open;
    }

    pub(crate) fn about_window(&mut self, ctx: &egui::Context) {
        let mut open = self.about_open;
        egui::Window::new(t("about.title"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(470.0)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(RichText::new(APP_TITLE).size(16.0).color(TEXT));
                ui.label(RichText::new(t("about.subtitle")).size(11.0).color(DIM));
                ui.add_space(10.0);
                section(ui, &t("about.tech"));
                kv(ui, &t("about.lang"), "Rust");
                kv(ui, &t("about.ui"), &t("about.value.ui"));
                kv(ui, &t("about.drawing"), &t("about.value.drawing"));
                kv(ui, &t("about.files"), &t("about.value.designs"));
                kv(ui, &t("about.wire"), &t("about.wire.value"));
                ui.add_space(10.0);
                section(ui, &t("about.meaning"));
                for line in [t("about.p1"), t("about.p2"), t("about.p3"), t("about.p4")] {
                    ui.label(RichText::new(line).size(11.5).color(MUTED));
                    ui.add_space(4.0);
                }
            });
        self.about_open = open;
    }
}
