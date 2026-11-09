//! Stitch Studio - an embroidery workspace that needs no USB key.

mod io;
mod lang;
mod model;
mod theme;

use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Rounding, Sense, Stroke, Vec2};
use lang::{t, tf};
use model::{El, Obj, Project, ShapeKind, Stats, Step, StitchKind, HOOPS, RATE_MAX_SPS, RATE_MIN_SPS, THREADS};
use std::path::PathBuf;
use theme::*;

const APP_TITLE: &str = "Stitch Studio";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Select,
    FillRect,
    FillEllipse,
    RunPath,
    SatinPath,
    Pan,
}

impl Tool {
    fn hint(self) -> String {
        match self {
            Tool::Select => t("tool.select.hint"),
            Tool::FillRect => t("tool.rect.hint"),
            Tool::FillEllipse => t("tool.ellipse.hint"),
            Tool::RunPath => t("tool.run.hint"),
            Tool::SatinPath => t("tool.satin.hint"),
            Tool::Pan => t("tool.pan.hint"),
        }
    }

    fn creating(self) -> bool {
        matches!(self, Tool::FillRect | Tool::FillEllipse | Tool::RunPath | Tool::SatinPath)
    }
}

enum Drag {
    None,
    Create { start: [f32; 2], cur: [f32; 2], path: Vec<[f32; 2]> },
    Move { last: [f32; 2] },
    Panning,
}

struct View {
    zoom: f32,
    pan: Vec2,
    needs_fit: bool,
}

struct Sim {
    idx: usize,
    playing: bool,
    /// Preview multiplier on top of the machine speed: 1×, 10× or 80×.
    mult: f32,
    acc: f32,
}

struct Shot {
    path: PathBuf,
    frames: u32,
    requested: bool,
}

struct App {
    project: Project,
    sel: Option<u32>,
    tool: Tool,
    view: View,
    sim: Sim,
    seq: Vec<Step>,
    stats: Stats,
    undo: Vec<Vec<Obj>>,
    redo: Vec<Vec<Obj>>,
    dirty: bool,
    drag: Drag,
    show_stitches: bool,
    show_grid: bool,
    show_rulers: bool,
    snap: bool,
    prop_drag: bool,
    hoop_drag: bool,
    tab: usize,
    help_open: bool,
    about_open: bool,
    toast: Option<(String, f32)>,
    toast_secs: f32,
    dialog: io::FileDialog,
    shot: Option<Shot>,
    quit: bool,
}

fn main() -> eframe::Result<()> {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!("Stitch Studio — an embroidery workspace that needs no USB key");
        println!();
        println!("Usage: stitch_studio [--lang <code>] [--screenshot <file.png>]");
        println!("  --lang <code>            en, es or eu (or any language file in lang/)");
        println!("  --screenshot <file.png>  draw a few frames, save the window as a PNG, exit");
        println!();
        println!("Text comes from the JSON files in the lang folder.");
        return Ok(());
    }
    let (shot, lang_code) = parse_args();
    lang::init(lang_code.as_deref());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_position([16.0, 16.0])
            .with_min_inner_size([1024.0, 660.0])
            .with_title(format!("{APP_TITLE} — {}", t("app.window"))),
        ..Default::default()
    };
    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(move |cc| {
            install_fonts(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(App::new(shot)))
        }),
    )
}

fn parse_args() -> (Option<PathBuf>, Option<String>) {
    let args: Vec<String> = std::env::args().collect();
    let mut shot = None;
    let mut language = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--screenshot" || args[i] == "-s" {
            if let Some(p) = args.get(i + 1) {
                shot = Some(PathBuf::from(p));
                i += 1;
            }
        } else if let Some(rest) = args[i].strip_prefix("--screenshot=") {
            shot = Some(PathBuf::from(rest));
        } else if args[i] == "--lang" || args[i] == "-l" {
            if let Some(code) = args.get(i + 1) {
                language = Some(code.to_lowercase());
                i += 1;
            }
        } else if let Some(rest) = args[i].strip_prefix("--lang=") {
            language = Some(rest.to_lowercase());
        }
        i += 1;
    }
    (shot, language)
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for path in [
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert("ui-text".to_string(), egui::FontData::from_owned(bytes));
            if let Some(list) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                list.insert(0, "ui-text".to_string());
            }
            break;
        }
    }
    ctx.set_fonts(fonts);
}

impl App {
    fn new(shot: Option<PathBuf>) -> Self {
        let project = model::demo_project();
        let seq = model::timeline(&project);
        let stats = model::stats(&project);
        let mut app = Self {
            project,
            sel: None,
            tool: Tool::Select,
            view: View {
                zoom: 3.2,
                pan: Vec2::ZERO,
                needs_fit: true,
            },
            sim: Sim {
                idx: 0,
                playing: false,
                mult: 1.0,
                acc: 0.0,
            },
            seq,
            stats,
            undo: Vec::new(),
            redo: Vec::new(),
            dirty: false,
            drag: Drag::None,
            show_stitches: true,
            show_grid: true,
            show_rulers: true,
            snap: true,
            prop_drag: false,
            hoop_drag: false,
            tab: 0,
            help_open: false,
            about_open: false,
            toast: None,
            toast_secs: 2.6,
            dialog: io::FileDialog::new(),
            shot: shot.map(|path| Shot {
                path,
                frames: 0,
                requested: false,
            }),
            quit: false,
        };
        app.project.name = t("name.demo");
        app
    }

    fn say(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), self.toast_secs));
    }

    fn push_undo(&mut self) {
        self.undo.push(self.project.objects.clone());
        if self.undo.len() > 60 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
    }

    fn rebuild(&mut self) {
        self.seq = model::timeline(&self.project);
        self.stats = model::stats(&self.project);
        self.sim.idx = self.sim.idx.min(self.seq.len());
        if let Some(id) = self.sel {
            if self.project.get(id).is_none() {
                self.sel = None;
            }
        }
    }

    fn selected_obj(&self) -> Option<&Obj> {
        self.sel.and_then(|id| self.project.get(id))
    }

    // ------------------------------------------------------------- transforms

    fn to_screen(&self, rect: Rect, mm: [f32; 2]) -> Pos2 {
        rect.center() + self.view.pan + Vec2::new(mm[0] * self.view.zoom, -mm[1] * self.view.zoom)
    }

    fn to_mm(&self, rect: Rect, s: Pos2) -> [f32; 2] {
        let d = s - (rect.center() + self.view.pan);
        [d.x / self.view.zoom, -d.y / self.view.zoom]
    }

    fn fit(&mut self, rect: Rect) {
        let (hw, hh) = self.project.hoop_size();
        let z = ((rect.width() - 120.0) / hw).min((rect.height() - 120.0) / hh);
        self.view.zoom = z.clamp(0.8, 30.0);
        self.view.pan = Vec2::ZERO;
        self.view.needs_fit = false;
    }

    // ------------------------------------------------------------------ shell

    fn chrome(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("chrome")
            .frame(
                egui::Frame::none()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(12.0, 0.0))
                    .stroke(Stroke::new(0.0, LINE_SOFT)),
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
                        Stroke::new(1.6, Color32::from_rgb(0xe9, 0xfb, 0xf3)),
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
                    let picked = ui.selectable_label(current == *code, name.as_str()).clicked();
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
                    self.push_undo();
                    self.project = Project::default();
                    self.project.name = t("name.untitled");
                    self.sel = None;
                    self.sim.idx = 0;
                    self.rebuild();
                    self.view.needs_fit = true;
                    ui.close_menu();
                }
                if ui.button(t("menu.file.example")).clicked() {
                    self.load_example();
                    ui.close_menu();
                }
                if ui.button(t("menu.file.open")).clicked() {
                    self.dialog = std::mem::replace(&mut self.dialog, io::FileDialog::new())
                        .ask_open(&t("title.open_design"), "json", &t("filter.designs"));
                    ui.close_menu();
                }
                if ui.button(t("menu.file.import")).clicked() {
                    self.dialog = std::mem::replace(&mut self.dialog, io::FileDialog::new())
                        .ask_open(&t("title.import"), "dst", &t("filter.dst"));
                    ui.close_menu();
                }
                if ui.button(t("menu.file.save")).clicked() {
                    self.save_quick();
                    ui.close_menu();
                }
                if ui.button(t("menu.file.save_as")).clicked() {
                    self.dialog = std::mem::replace(&mut self.dialog, io::FileDialog::new()).ask_save(
                        &t("title.save_design"),
                        "json",
                        &t("filter.designs"),
                        &format!("{}.json", slug(&self.project.name)),
                    );
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(t("menu.file.export_dst")).clicked() {
                    self.dialog = std::mem::replace(&mut self.dialog, io::FileDialog::new()).ask_save(
                        &t("title.export_dst"),
                        "dst",
                        &t("filter.dst"),
                        &format!("{}.dst", slug(&self.project.name)),
                    );
                    ui.close_menu();
                }
                if ui.button(t("menu.file.export_svg")).clicked() {
                    self.dialog = std::mem::replace(&mut self.dialog, io::FileDialog::new()).ask_save(
                        &t("title.export_svg"),
                        "svg",
                        &t("filter.svg"),
                        &format!("{}.svg", slug(&self.project.name)),
                    );
                    ui.close_menu();
                }
                if ui.button(t("menu.file.export_csv")).clicked() {
                    self.dialog = std::mem::replace(&mut self.dialog, io::FileDialog::new()).ask_save(
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
            if icon_button(ui, Icon::Ruler, &t("toolbar.rulers"), self.show_rulers, true).clicked() {
                self.show_rulers = !self.show_rulers;
            }
            if icon_button(ui, Icon::Needle, &t("toolbar.stitches"), self.show_stitches, true).clicked() {
                self.show_stitches = !self.show_stitches;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let z = self.view.zoom;
                ui.label(
                    RichText::new(format!("{:.0}%", z / 3.78 * 100.0))
                        .size(11.5)
                        .color(MUTED),
                );
                if ui.small_button("+").on_hover_text(t("toolbar.zoom_in")).clicked() {
                    self.view.zoom = (self.view.zoom * 1.2).min(40.0);
                }
                if ui.small_button("−").on_hover_text(t("toolbar.zoom_out")).clicked() {
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

    fn fits_hoop(&self) -> bool {
        let (hw, hh) = self.project.hoop_size();
        match self.project.bounds() {
            Some(b) => b.0 >= -hw / 2.0 && b.2 <= hw / 2.0 && b.1 >= -hh / 2.0 && b.3 <= hh / 2.0,
            None => true,
        }
    }

    // ------------------------------------------------------------- side panels

    fn left_panel(&mut self, ctx: &egui::Context) {
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
                    for (label, val) in [
                        (t("hoop.width"), &mut hw),
                        (t("hoop.height"), &mut hh),
                    ] {
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
                    kv(ui, &t("summary.size"), &format!("{:.1} × {:.1} mm", st.w, st.h));
                    kv(ui, &t("summary.size_cm"), &format!("{:.1} × {:.1} cm", st.w / 10.0, st.h / 10.0));
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
                            .add(egui::Button::new(RichText::new(t("objects.delete")).size(10.5).color(DANGER)).frame(false))
                            .clicked()
                        {
                            self.delete_selected();
                        }
                        if ui
                            .add(egui::Button::new(RichText::new(t("objects.duplicate")).size(10.5).color(MUTED)).frame(false))
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
                                        ui.label(RichText::new(t("objects.empty.title")).size(12.0).color(DIM));
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
                                                .stroke(Stroke::new(1.0, LINE))
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
                                let count = rows.len();
                                for (idx, (id, title, sub, color, visible, selected)) in rows.into_iter().enumerate() {
                                    let (rect, resp) = ui.allocate_exact_size(
                                        Vec2::new(ui.available_width(), 34.0),
                                        Sense::click(),
                                    );
                                    let p = ui.painter();
                                    if selected {
                                        p.rect_filled(rect, Rounding::same(6.0), Color32::from_rgb(0x2b, 0x34, 0x3f));
                                    } else if resp.hovered() {
                                        p.rect_filled(rect, Rounding::same(6.0), Color32::from_rgb(0x22, 0x28, 0x31));
                                    }
                                    let sw = Rect::from_min_size(
                                        rect.left_top() + Vec2::new(8.0, 10.0),
                                        Vec2::new(13.0, 13.0),
                                    );
                                    p.rect_filled(sw, Rounding::same(3.0), model::thread_color(color));
                                    p.rect_stroke(sw, Rounding::same(3.0), Stroke::new(1.0, Color32::from_black_alpha(60)));
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
                                    let eye_resp = ui.interact(eye, ui.id().with(("eye", id)), Sense::click());
                                    let ec = if visible { MUTED } else { DIM };
                                    p.circle_stroke(eye.center(), 4.0, Stroke::new(1.2, ec));
                                    if visible {
                                        p.circle_filled(eye.center(), 1.8, ec);
                                    }
                                    if eye_resp.clicked() {
                                        action = Some(RowAction::Toggle(id));
                                    } else if resp.clicked() {
                                        action = Some(RowAction::Select(id));
                                    }
                                    if selected && idx > 0 && resp.hovered() {
                                        let _ = count;
                                    }
                                }
                            });
                    });
                // move buttons under the list
                ui.horizontal(|ui| {
                    let sel = self.sel;
                    if ui
                        .add_enabled(sel.is_some(), egui::Button::new(RichText::new(t("objects.move_up")).size(10.5)).min_size(Vec2::new(78.0, 22.0)))
                        .clicked()
                    {
                        if let Some(id) = sel {
                            action = Some(RowAction::Up(id));
                        }
                    }
                    if ui
                        .add_enabled(sel.is_some(), egui::Button::new(RichText::new(t("objects.move_down")).size(10.5)).min_size(Vec2::new(78.0, 22.0)))
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

    fn right_panel(&mut self, ctx: &egui::Context) {
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
                    .show(ui, |ui| {
                        match self.tab {
                            0 => self.threads_tab(ui),
                            1 => self.settings_tab(ui),
                            _ => self.plan_tab(ui),
                        }
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
                    ui.painter().rect_filled(r, Rounding::same(3.0), model::thread_color(c));
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
                        let (r, resp) = ui.allocate_exact_size(Vec2::new(28.0, 28.0), Sense::click());
                        let p = ui.painter();
                        p.rect_filled(r.shrink(1.0), Rounding::same(5.0), model::thread_color(i));
                        if sel_color == Some(i) {
                            p.rect_stroke(r, Rounding::same(5.0), Stroke::new(2.0, Color32::WHITE));
                        } else if resp.hovered() {
                            p.rect_stroke(r, Rounding::same(5.0), Stroke::new(1.0, Color32::from_white_alpha(120)));
                        }
                        if resp.on_hover_text(&name).clicked() {
                            picked = Some(i);
                        }
                    }
                });
                let _ = ui.available_width();
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
                ui.painter().rect_filled(r, Rounding::same(3.0), model::thread_color(*c));
                ui.label(RichText::new(format!("{}. {}", i + 1, lang::thread_name(*c))).size(11.5));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(tf("plan.st", &[("n", &fmt_num(*n))])).size(10.5).color(DIM));
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
            (o.kind, o.density, o.stitch_len, o.satin_w, o.rot, o.cx, o.cy, o.w, o.h)
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
                                egui::Button::new(RichText::new(label).size(10.5).color(if on { Color32::from_rgb(0x0d, 0x27, 0x21) } else { MUTED }))
                                    .fill(if on { ACCENT } else { FIELD })
                                    .stroke(Stroke::new(1.0, if on { ACCENT } else { LINE }))
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
        let r = ui.add(egui::Slider::new(&mut rot, -3.14..=3.14).text(t("prop.rotation")).fixed_decimals(2));
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
            let r = ui.add(egui::Slider::new(&mut w, 3.0..=200.0).text(t("prop.width")).suffix(" mm").fixed_decimals(1));
            began |= r.drag_started();
            ended |= r.drag_stopped();
            changed |= r.changed();
            let r = ui.add(egui::Slider::new(&mut h, 3.0..=200.0).text(t("prop.height")).suffix(" mm").fixed_decimals(1));
            began |= r.drag_started();
            ended |= r.drag_stopped();
            changed |= r.changed();
        }

        ui.add_space(8.0);
        section(ui, &t("panel.stitches"));
        let mut d = density;
        let rd = ui.add(egui::Slider::new(&mut d, 0.3..=1.6).text(t("prop.spacing")).suffix(" mm").fixed_decimals(2));
        began |= rd.drag_started();
        ended |= rd.drag_stopped();
        if rd.changed() {
            density = d;
            changed = true;
        }
        let mut sl = slen;
        let rs = ui.add(egui::Slider::new(&mut sl, 1.0..=5.0).text(t("prop.stitch_len")).suffix(" mm").fixed_decimals(1));
        began |= rs.drag_started();
        ended |= rs.drag_stopped();
        if rs.changed() {
            slen = sl;
            changed = true;
        }
        if kind == StitchKind::Satin {
            let mut sw = satin_w;
            let rw = ui.add(egui::Slider::new(&mut sw, 1.5..=15.0).text(t("prop.column_width")).suffix(" mm").fixed_decimals(1));
            began |= rw.drag_started();
            ended |= rw.drag_stopped();
            if rw.changed() {
                satin_w = sw;
                changed = true;
            }
        }
        } else {
            ui.add_space(8.0);
            hint(
                ui,
                &t("settings.imported"),
            );
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
                &format!("{:.1} {}", self.project.stitches_per_second(), t("unit.sps")),
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
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), Sense::click());
            let p = ui.painter();
            if resp.hovered() {
                p.rect_filled(rect, Rounding::same(4.0), Color32::from_rgb(0x24, 0x2a, 0x33));
            }
            let sw = Rect::from_min_size(rect.left_top() + Vec2::new(4.0, 7.0), Vec2::new(12.0, 12.0));
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

    // ---------------------------------------------------------------- actions

    fn load_example(&mut self) {
        self.push_undo();
        self.project = model::flower_example();
        self.project.name = t("name.demo");
        self.sel = None;
        self.sim.idx = 0;
        self.sim.playing = false;
        self.rebuild();
        self.view.needs_fit = true;
        self.dirty = false;
        self.say(t("msg.example"));
    }

    fn save_quick(&mut self) {
        self.dialog = std::mem::replace(&mut self.dialog, io::FileDialog::new()).ask_save(
            &t("title.save_design"),
            "json",
            &t("filter.designs"),
            &format!("{}.json", slug(&self.project.name)),
        );
        self.say(t("msg.choose_where"));
    }

    fn handle_file_pick(&mut self) {
        let Some(path) = self.dialog.picked.clone() else {
            return;
        };
        self.dialog.picked = None;
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        match self.dialog.mode {
            io::DialogMode::Open => {
                let res = if ext == "dst" {
                    io::import_dst(&path).map(|p| (p, t("msg.imported")))
                } else {
                    io::load_project(&path).map(|p| (p, t("msg.opened")))
                };
                match res {
                    Ok((p, msg)) => {
                        self.project = p;
                        self.sel = None;
                        self.sim.idx = 0;
                        self.rebuild();
                        self.view.needs_fit = true;
                        self.dirty = false;
                        self.say(msg);
                    }
                    Err(e) => self.say(e),
                }
            }
            io::DialogMode::Save => {
                let res = match ext.as_str() {
                    "dst" => io::export_dst(&self.project, &path),
                    "svg" => io::export_svg(&self.project, &path),
                    "csv" => io::export_csv(&self.project, &path),
                    _ => io::save_project(&self.project, &path),
                };
                match res {
                    Ok(()) => {
                        self.dirty = false;
                        self.say(tf(
                            "msg.saved",
                            &[(
                                "name",
                                &path.file_name().unwrap_or_default().to_string_lossy(),
                            )],
                        ));
                    }
                    Err(e) => self.say(e),
                }
            }
        }
    }

    fn do_undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            self.redo.push(self.project.objects.clone());
            self.project.objects = prev;
            self.rebuild();
        }
    }

    fn do_redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(self.project.objects.clone());
            self.project.objects = next;
            self.rebuild();
        }
    }

    fn delete_selected(&mut self) {
        if let Some(id) = self.sel {
            self.push_undo();
            self.project.objects.retain(|o| o.id != id);
            self.sel = None;
            self.rebuild();
        }
    }

    fn duplicate(&mut self) {
        if let Some(id) = self.sel {
            if let Some(o) = self.project.get(id).cloned() {
                self.push_undo();
                let mut copy = o;
                copy.cx += 8.0;
                copy.cy -= 8.0;
                if !copy.path.is_empty() {
                    for p in copy.path.iter_mut() {
                        p[0] += 8.0;
                        p[1] -= 8.0;
                    }
                }
                let nid = self.project.push(copy);
                self.sel = Some(nid);
                self.rebuild();
            }
        }
    }

    fn centre_selected(&mut self) {
        if let Some(id) = self.sel {
            self.push_undo();
            if let Some(o) = self.project.get_mut(id) {
                let (cx, cy) = (o.cx, o.cy);
                o.cx = 0.0;
                o.cy = 0.0;
                if !o.path.is_empty() {
                    for p in o.path.iter_mut() {
                        p[0] -= cx;
                        p[1] -= cy;
                    }
                }
            }
            self.rebuild();
        }
    }

    fn move_obj(&mut self, id: u32, dir: i32) {
        if let Some(i) = self.project.objects.iter().position(|o| o.id == id) {
            let j = (i as i32 + dir).clamp(0, self.project.objects.len() as i32 - 1) as usize;
            if i != j {
                self.push_undo();
                let o = self.project.objects.remove(i);
                self.project.objects.insert(j, o);
                self.rebuild();
            }
        }
    }

    fn optimise_order(&mut self) {
        self.push_undo();
        self.project.objects.sort_by_key(|o| o.color);
        self.rebuild();
        self.say(t("msg.grouped"));
    }

    // ----------------------------------------------------------------- canvas

    fn canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG).inner_margin(egui::Margin::same(8.0)))
            .show(ctx, |ui| {
                let rect = ui.available_rect_before_wrap();
                let resp = ui.allocate_rect(rect, Sense::click_and_drag());
                if self.view.needs_fit {
                    self.fit(rect);
                }
                self.canvas_input(ui, rect, &resp);
                self.paint_canvas(ui, rect);
                if let Some((msg, _)) = self.toast.clone() {
                    {
                        let painter = ui.painter_at(rect);
                        let galley = painter.layout_no_wrap(msg, FontId::proportional(12.0), TEXT);
                        let pos = Pos2::new(rect.center().x, rect.bottom() - 34.0);
                        let r = Rect::from_center_size(
                            pos,
                            galley.size() + Vec2::new(26.0, 14.0),
                        );
                        painter.rect_filled(r, Rounding::same(8.0), Color32::from_rgb(0x24, 0x2c, 0x34));
                        painter.rect_stroke(r, Rounding::same(8.0), Stroke::new(1.0, LINE));
                        painter.galley(Pos2::new(r.left() + 13.0, r.top() + 7.0), galley, TEXT);
                    }
                }
                let _ = ctx;
            });
    }

    fn canvas_input(&mut self, ui: &mut egui::Ui, rect: Rect, resp: &egui::Response) {
        let pointer = ui.input(|i| i.pointer.hover_pos());
        // zoom
        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.5 {
                if let Some(pos) = pointer {
                    let anchor = self.to_mm(rect, pos);
                    let z = (self.view.zoom * (1.0 + scroll * 0.0018)).clamp(0.5, 40.0);
                    self.view.zoom = z;
                    self.view.pan = pos - rect.center() - Vec2::new(anchor[0] * z, -anchor[1] * z);
                }
            }
        }
        // pan with middle / secondary button
        if resp.hovered()
            && (ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle))
                || ui.input(|i| i.pointer.button_down(egui::PointerButton::Secondary)))
        {
            let d = ui.input(|i| i.pointer.delta());
            self.view.pan += d;
        }

        // A plain click picks the object under the pointer, or clears the
        // selection when the fabric is empty.
        if resp.clicked() && self.tool == Tool::Select {
            if let Some(pos) = resp.interact_pointer_pos() {
                let mm = self.to_mm(rect, pos);
                self.sel = self
                    .project
                    .objects
                    .iter()
                    .rev()
                    .find(|o| o.visible && o.contains(mm))
                    .map(|o| o.id);
            }
        }

        if resp.drag_started_by(egui::PointerButton::Primary) {
            if let Some(pos) = resp.interact_pointer_pos() {
                let mm = self.to_mm(rect, pos);
                if self.tool == Tool::Pan {
                    self.drag = Drag::Panning;
                } else if self.tool == Tool::Select {
                    let hit = self
                        .project
                        .objects
                        .iter()
                        .rev()
                        .find(|o| o.visible && o.contains(mm))
                        .map(|o| o.id);
                    self.sel = hit;
                    self.drag = match hit {
                        Some(_) => {
                            self.push_undo();
                            Drag::Move { last: mm }
                        }
                        None => Drag::None,
                    };
                } else if self.tool.creating() {
                    self.drag = Drag::Create {
                        start: mm,
                        cur: mm,
                        path: vec![mm],
                    };
                }
            }
        }

        if resp.dragged_by(egui::PointerButton::Primary) {
            if let Some(pos) = resp.interact_pointer_pos() {
                let mm = mm_snap(self.to_mm(rect, pos), self.snap);
                match &mut self.drag {
                    Drag::Move { last } => {
                        let dx = mm[0] - last[0];
                        let dy = mm[1] - last[1];
                        if dx.abs() > 0.001 || dy.abs() > 0.001 {
                            if let Some(id) = self.sel {
                                if let Some(o) = self.project.get_mut(id) {
                                    o.cx += dx;
                                    o.cy += dy;
                                    if !o.path.is_empty() {
                                        for p in o.path.iter_mut() {
                                            p[0] += dx;
                                            p[1] += dy;
                                        }
                                    }
                                }
                            }
                            *last = mm;
                        }
                    }
                    Drag::Create { start, cur, path } => {
                        if self.tool == Tool::RunPath || self.tool == Tool::SatinPath {
                            let last = *path.last().unwrap_or(start);
                            let d = ((mm[0] - last[0]).powi(2) + (mm[1] - last[1]).powi(2)).sqrt();
                            if d > 1.2 {
                                path.push(mm);
                            }
                        } else {
                            let s = start;
                            *cur = [mm[0], mm[1]];
                            let _ = s;
                        }
                    }
                    Drag::Panning => {
                        let d = ui.input(|i| i.pointer.delta());
                        self.view.pan += d;
                    }
                    Drag::None => {}
                }
            }
        }

        if resp.drag_stopped_by(egui::PointerButton::Primary) {
            let drag = std::mem::replace(&mut self.drag, Drag::None);
            if matches!(drag, Drag::Move { .. }) {
                self.rebuild();
            }
            if let Drag::Create { start, cur, path } = drag {
                match self.tool {
                    Tool::FillRect | Tool::FillEllipse => {
                        let (mut x0, mut y0) = (start[0].min(cur[0]), start[1].min(cur[1]));
                        let (mut x1, mut y1) = (start[0].max(cur[0]), start[1].max(cur[1]));
                        if x1 - x0 < 3.0 || y1 - y0 < 3.0 {
                            x0 = start[0] - 12.0;
                            x1 = start[0] + 12.0;
                            y0 = start[1] - 12.0;
                            y1 = start[1] + 12.0;
                        }
                        self.push_undo();
                        let kind = StitchKind::Fill;
                        let o = if self.tool == Tool::FillRect {
                            Obj::new_rect(0, kind, (x0 + x1) / 2.0, (y0 + y1) / 2.0, x1 - x0, y1 - y0)
                        } else {
                            Obj::new_ellipse(0, kind, (x0 + x1) / 2.0, (y0 + y1) / 2.0, x1 - x0, y1 - y0)
                        };
                        let id = self.project.push(o);
                        self.sel = Some(id);
                        self.rebuild();
                        self.tool = Tool::Select;
                    }
                    Tool::RunPath | Tool::SatinPath => {
                        let mut pts = path;
                        if pts.len() < 2 {
                            pts = vec![[start[0] - 16.0, start[1]], [start[0] + 16.0, start[1]]];
                        }
                        let kind = if self.tool == Tool::SatinPath { StitchKind::Satin } else { StitchKind::Run };
                        self.push_undo();
                        let mut o = Obj::new_path(0, kind, pts);
                        if kind == StitchKind::Satin {
                            o.color = 8;
                        }
                        let id = self.project.push(o);
                        self.sel = Some(id);
                        self.rebuild();
                        self.tool = Tool::Select;
                    }
                    _ => {}
                }
            }
        }

        // click creates a default shape as well
        if resp.clicked() && self.tool.creating() {
            if let Some(pos) = resp.interact_pointer_pos() {
                let mm = self.to_mm(rect, pos);
                self.push_undo();
                let o = match self.tool {
                    Tool::FillRect => Obj::new_rect(0, StitchKind::Fill, mm[0], mm[1], 24.0, 24.0),
                    Tool::FillEllipse => Obj::new_ellipse(0, StitchKind::Fill, mm[0], mm[1], 26.0, 22.0),
                    Tool::RunPath => Obj::new_path(
                        0,
                        StitchKind::Run,
                        vec![[mm[0] - 16.0, mm[1]], [mm[0] + 16.0, mm[1]]],
                    ),
                    Tool::SatinPath => Obj::new_path(
                        0,
                        StitchKind::Satin,
                        vec![[mm[0] - 16.0, mm[1]], [mm[0] + 16.0, mm[1]]],
                    ),
                    _ => return,
                };
                let id = self.project.push(o);
                self.sel = Some(id);
                self.rebuild();
                self.tool = Tool::Select;
            }
        }
    }

    /// Millimetre scales against the four edges of the hoop: one small tick
    /// per millimetre, a longer tick every five and a numbered tick every
    /// ten. The numbers are the same coordinates the rest of the window
    /// uses, so the centre of the hoop reads 0.
    fn paint_rulers(&self, p: &egui::Painter, rect: Rect, fabric: Rect, hw: f32, hh: f32) {
        const BAND: f32 = 14.0;
        let outer = fabric.expand(BAND);
        if !rect.intersects(outer) {
            return;
        }
        let z = self.view.zoom;
        let (tick_mm, label_mm) = ruler_steps(z);
        let x0 = -hw / 2.0;
        let y_top = hh / 2.0;
        let label_col = Color32::from_rgb(0xa6, 0xb1, 0xbe);
        let font = FontId::proportional(10.0);

        // The four strips that carry the marks, just outside the fabric.
        for strip in [
            Rect::from_min_max(
                Pos2::new(outer.left(), outer.top()),
                Pos2::new(outer.right(), fabric.top()),
            ),
            Rect::from_min_max(
                Pos2::new(outer.left(), fabric.bottom()),
                Pos2::new(outer.right(), outer.bottom()),
            ),
            Rect::from_min_max(
                Pos2::new(outer.left(), fabric.top()),
                Pos2::new(fabric.left(), fabric.bottom()),
            ),
            Rect::from_min_max(
                Pos2::new(fabric.right(), fabric.top()),
                Pos2::new(outer.right(), fabric.bottom()),
            ),
        ] {
            p.rect_filled(strip, Rounding::ZERO, Color32::from_white_alpha(7));
        }
        p.rect_stroke(
            outer,
            Rounding::same(3.0),
            Stroke::new(1.0, Color32::from_white_alpha(26)),
        );

        // Horizontal scale, along the top and bottom edges.
        let mut v = (x0.ceil() as i32 / tick_mm) * tick_mm;
        let last = (hw / 2.0).floor() as i32;
        while v <= last {
            let sx = fabric.left() + (v as f32 - x0) * z;
            if sx >= rect.left() - 1.0 && sx <= rect.right() + 1.0 {
                let (len, stroke) = tick_style(v);
                p.line_segment(
                    [Pos2::new(sx, fabric.top()), Pos2::new(sx, fabric.top() - len)],
                    stroke,
                );
                p.line_segment(
                    [Pos2::new(sx, fabric.bottom()), Pos2::new(sx, fabric.bottom() + len)],
                    stroke,
                );
                if v % label_mm == 0 {
                    let col = if v == 0 { ACCENT } else { label_col };
                    let txt = v.to_string();
                    p.text(
                        Pos2::new(sx, fabric.top() - BAND - 2.0),
                        Align2::CENTER_BOTTOM,
                        &txt,
                        font.clone(),
                        col,
                    );
                    p.text(
                        Pos2::new(sx, fabric.bottom() + BAND + 2.0),
                        Align2::CENTER_TOP,
                        &txt,
                        font.clone(),
                        col,
                    );
                }
            }
            v += tick_mm;
        }

        // Vertical scale, along the left and right edges.
        let mut v = (y_top.floor() as i32 / tick_mm) * tick_mm;
        let lowest = (-hh / 2.0).ceil() as i32;
        while v >= lowest {
            let sy = fabric.top() + (y_top - v as f32) * z;
            if sy >= rect.top() - 1.0 && sy <= rect.bottom() + 1.0 {
                let (len, stroke) = tick_style(v);
                p.line_segment(
                    [Pos2::new(fabric.left(), sy), Pos2::new(fabric.left() - len, sy)],
                    stroke,
                );
                p.line_segment(
                    [Pos2::new(fabric.right(), sy), Pos2::new(fabric.right() + len, sy)],
                    stroke,
                );
                if v % label_mm == 0 {
                    let col = if v == 0 { ACCENT } else { label_col };
                    let txt = v.to_string();
                    p.text(
                        Pos2::new(fabric.left() - BAND - 2.0, sy),
                        Align2::RIGHT_CENTER,
                        &txt,
                        font.clone(),
                        col,
                    );
                    p.text(
                        Pos2::new(fabric.right() + BAND + 2.0, sy),
                        Align2::LEFT_CENTER,
                        &txt,
                        font.clone(),
                        col,
                    );
                }
            }
            v -= tick_mm;
        }

        // Unit badge, tucked above the top-left corner.
        p.text(
            Pos2::new(outer.left() - 2.0, outer.top() - 2.0),
            Align2::RIGHT_BOTTOM,
            "mm",
            FontId::proportional(9.0),
            DIM,
        );
    }

    fn paint_canvas(&self, ui: &mut egui::Ui, rect: Rect) {
        let p = ui.painter_at(rect);
        p.rect_filled(rect, Rounding::ZERO, BG);
        // subtle vignette lines
        let (hw, hh) = self.project.hoop_size();
        let z = self.view.zoom;

        // fabric
        let top_left = self.to_screen(rect, [-hw / 2.0, hh / 2.0]);
        let fabric = Rect::from_min_max(top_left, self.to_screen(rect, [hw / 2.0, -hh / 2.0]));
        // hoop shadow
        p.rect_filled(fabric.expand(3.0), Rounding::same(4.0), Color32::from_black_alpha(70));
        p.rect_filled(fabric, Rounding::same(2.0), FABRIC);
        // fabric texture: faint weave
        p.rect_stroke(fabric, Rounding::same(2.0), Stroke::new(1.0, FABRIC_EDGE));

        if self.show_grid && z > 1.0 {
            let step = if z > 6.0 { 5.0 } else { 10.0 };
            let mut x = -hw / 2.0;
            while x <= hw / 2.0 + 0.01 {
                let sx = self.to_screen(rect, [x, 0.0]).x;
                let strong = (x.round() as i32) % 50 == 0;
                p.line_segment(
                    [Pos2::new(sx, fabric.top()), Pos2::new(sx, fabric.bottom())],
                    Stroke::new(1.0, Color32::from_black_alpha(if strong { 26 } else { 12 })),
                );
                x += step;
            }
            let mut y = -hh / 2.0;
            while y <= hh / 2.0 + 0.01 {
                let sy = self.to_screen(rect, [0.0, y]).y;
                let strong = (y.round() as i32) % 50 == 0;
                p.line_segment(
                    [Pos2::new(fabric.left(), sy), Pos2::new(fabric.right(), sy)],
                    Stroke::new(1.0, Color32::from_black_alpha(if strong { 26 } else { 12 })),
                );
                y += step;
            }
            // centre axes
            let c = self.to_screen(rect, [0.0, 0.0]);
            dashed(&p, Pos2::new(c.x, fabric.top()), Pos2::new(c.x, fabric.bottom()), Stroke::new(1.0, Color32::from_black_alpha(34)), 5.0);
            dashed(&p, Pos2::new(fabric.left(), c.y), Pos2::new(fabric.right(), c.y), Stroke::new(1.0, Color32::from_black_alpha(34)), 5.0);
        }

        // hoop band
        p.rect_stroke(fabric, Rounding::same(2.0), Stroke::new(2.0, Color32::from_rgb(0x7e, 0x8a, 0x97)));
        let band = Rect::from_min_max(Pos2::new(fabric.left() - 6.0, fabric.top() - 6.0), Pos2::new(fabric.right() + 6.0, fabric.bottom() + 6.0));
        p.rect_stroke(band, Rounding::same(4.0), Stroke::new(1.0, Color32::from_white_alpha(18)));

        if self.show_rulers {
            self.paint_rulers(&p, rect, fabric, hw, hh);
        }

        // clip stitches to the fabric plus a margin
        let clip = fabric.expand(4.0);
        let pain = p.with_clip_rect(clip.intersect(rect));

        // objects
        let stitch_w = (z * 0.5).clamp(0.6, 2.6);
        for o in &self.project.objects {
            if !o.visible {
                continue;
            }
            let col = model::thread_color(o.color);
            let selected = self.sel == Some(o.id);
            if self.show_stitches {
                let mut chunks: Vec<Vec<Pos2>> = Vec::new();
                let mut cur: Vec<Pos2> = Vec::new();
                for el in model::object_stitches(o) {
                    match el {
                        El::Move(pt) => {
                            if cur.len() > 1 {
                                chunks.push(std::mem::take(&mut cur));
                            } else {
                                cur.clear();
                            }
                            cur.push(self.to_screen(rect, pt));
                        }
                        El::Stitch(pt) => cur.push(self.to_screen(rect, pt)),
                    }
                }
                if cur.len() > 1 {
                    chunks.push(cur);
                }
                if selected {
                    for ch in &chunks {
                        pain.add(egui::Shape::line(ch.clone(), Stroke::new(stitch_w + 3.0, Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 46))));
                    }
                }
                for ch in &chunks {
                    pain.add(egui::Shape::line(ch.clone(), Stroke::new(stitch_w, col)));
                }
            } else {
                let pts: Vec<Pos2> = o
                    .local_outline()
                    .iter()
                    .map(|q| self.to_screen(rect, o.to_world(*q)))
                    .collect();
                if pts.len() > 2 {
                    pain.add(egui::Shape::convex_polygon(
                        pts,
                        Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 210),
                        Stroke::new(0.6, Color32::from_black_alpha(60)),
                    ));
                }
            }
        }

        // simulation overlay
        if self.sim.idx > 0 && self.sim.idx <= self.seq.len() {
            let mut chunks: Vec<Vec<Pos2>> = Vec::new();
            let mut cur: Vec<Pos2> = Vec::new();
            let mut needle = None;
            for st in self.seq.iter().take(self.sim.idx) {
                match st.el {
                    El::Move(pt) => {
                        if cur.len() > 1 {
                            chunks.push(std::mem::take(&mut cur));
                        } else {
                            cur.clear();
                        }
                        cur.push(self.to_screen(rect, pt));
                        needle = Some(pt);
                    }
                    El::Stitch(pt) => {
                        cur.push(self.to_screen(rect, pt));
                        needle = Some(pt);
                    }
                }
            }
            if cur.len() > 1 {
                chunks.push(cur);
            }
            for ch in &chunks {
                pain.add(egui::Shape::line(ch.clone(), Stroke::new(stitch_w + 2.0, Color32::from_rgb(0xff, 0xd9, 0x66))));
            }
            if let Some(n) = needle {
                let s = self.to_screen(rect, n);
                pain.circle_stroke(s, 7.0, Stroke::new(1.4, Color32::from_rgb(0xff, 0xd9, 0x66)));
                pain.circle_filled(s, 2.2, Color32::from_rgb(0xff, 0xf3, 0xc4));
                let l = 11.0;
                pain.line_segment([s + Vec2::new(-l, 0.0), s + Vec2::new(-3.0, 0.0)], Stroke::new(1.0, Color32::from_white_alpha(120)));
                pain.line_segment([s + Vec2::new(3.0, 0.0), s + Vec2::new(l, 0.0)], Stroke::new(1.0, Color32::from_white_alpha(120)));
                pain.line_segment([s + Vec2::new(0.0, -l), s + Vec2::new(0.0, -3.0)], Stroke::new(1.0, Color32::from_white_alpha(120)));
                pain.line_segment([s + Vec2::new(0.0, 3.0), s + Vec2::new(0.0, l)], Stroke::new(1.0, Color32::from_white_alpha(120)));
            }
        }

        // selection frame
        if let Some(o) = self.selected_obj() {
            let (x0, y0, x1, y1) = o.bounds();
            let a = self.to_screen(rect, [x0, y1]);
            let b = self.to_screen(rect, [x1, y0]);
            let r = Rect::from_min_max(a, b);
            let col = Color32::from_rgb(0x8a, 0xe4, 0xcb);
            for (p0, p1) in [
                (r.left_top(), r.right_top()),
                (r.right_top(), r.right_bottom()),
                (r.right_bottom(), r.left_bottom()),
                (r.left_bottom(), r.left_top()),
            ] {
                dashed(&pain, p0, p1, Stroke::new(1.2, col), 4.0);
            }
            for c in [r.left_top(), r.right_top(), r.left_bottom(), r.right_bottom()] {
                pain.rect_filled(Rect::from_center_size(c, Vec2::splat(5.0)), Rounding::same(1.0), col);
            }
        }

        // hoop labels (only when the millimetre rulers are off, so the two
        // do not sit on top of each other)
        if !self.show_rulers {
            p.text(
                Pos2::new(fabric.center().x, fabric.top() - 18.0),
                Align2::CENTER_CENTER,
                format!("{:.0} mm", hw),
                FontId::proportional(10.5),
                MUTED,
            );
            p.text(
                Pos2::new(fabric.left() - 20.0, fabric.center().y),
                Align2::CENTER_CENTER,
                format!("{:.0} mm", hh),
                FontId::proportional(10.5),
                MUTED,
            );
        }

        if !self.fits_hoop() {
            let warn_fg = Color32::from_rgb(0xf0, 0xd6, 0x9a);
            let galley = p.layout_no_wrap(
                t("canvas.outside"),
                FontId::proportional(10.5),
                warn_fg,
            );
            let size = galley.size() + Vec2::new(18.0, 8.0);
            let warn = Rect::from_min_size(Pos2::new(fabric.left(), fabric.top() + 8.0), size);
            p.rect_filled(warn, Rounding::same(5.0), Color32::from_rgb(0x4a, 0x3a, 0x1e));
            p.galley(warn.min + Vec2::new(9.0, 4.0), galley, warn_fg);
        }

        // empty state
        if self.project.objects.is_empty() {
            p.text(
                rect.center() - Vec2::new(0.0, 8.0),
                Align2::CENTER_CENTER,
                t("canvas.empty.title"),
                FontId::proportional(17.0),
                Color32::from_gray(120),
            );
            p.text(
                rect.center() + Vec2::new(0.0, 14.0),
                Align2::CENTER_CENTER,
                t("canvas.empty.hint"),
                FontId::proportional(11.5),
                Color32::from_gray(95),
            );
        }
    }

    // --------------------------------------------------------------- bottom bar

    fn bottom(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("transport")
            .exact_height(54.0)
            .frame(
                egui::Frame::none()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0))
                    .stroke(Stroke::new(1.0, LINE_SOFT)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let playing = self.sim.playing;
                    let (r, resp) = ui.allocate_exact_size(Vec2::new(30.0, 30.0), Sense::click());
                    let p = ui.painter();
                    p.circle_filled(r.center(), 14.0, if playing { ACCENT } else { Color32::from_rgb(0x2f, 0x37, 0x42) });
                    if playing {
                        p.rect_filled(Rect::from_center_size(r.center() + Vec2::new(-4.0, 0.0), Vec2::new(3.0, 12.0)), Rounding::ZERO, Color32::from_rgb(0x0d, 0x27, 0x21));
                        p.rect_filled(Rect::from_center_size(r.center() + Vec2::new(3.0, 0.0), Vec2::new(3.0, 12.0)), Rounding::ZERO, Color32::from_rgb(0x0d, 0x27, 0x21));
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
                        .add(egui::Button::new(RichText::new(t("sim.reset")).size(10.5).color(MUTED)).frame(false))
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
                        RichText::new(format!("{} / {}", fmt_num(self.sim.idx), fmt_num(self.seq.len())))
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
                            let btn = egui::Button::new(
                                RichText::new(label).size(10.0).color(if active {
                                    Color32::from_rgb(0x0d, 0x27, 0x21)
                                } else {
                                    MUTED
                                }),
                            )
                            .min_size(Vec2::new(30.0, 20.0))
                            .fill(if active { ACCENT } else { PANEL2 })
                            .stroke(Stroke::new(1.0, if active { ACCENT } else { LINE }));
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
                    .stroke(Stroke::new(1.0, LINE_SOFT)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.tool.hint())
                            .size(10.5)
                            .color(DIM),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let hoop = self.project.hoop_name();
                        ui.label(
                            RichText::new(hoop)
                                .size(10.0)
                                .color(DIM),
                        );
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
                        ui.label(
                            RichText::new(t("status.offline"))
                                .size(10.0)
                                .color(MUTED),
                        )
                        .on_hover_text(t("status.offline_tip"));
                    });
                });
            });
    }

    // ------------------------------------------------------------- help window

    fn help_window(&mut self, ctx: &egui::Context) {
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
                                ui.label(RichText::new(k).size(11.0).color(Color32::from_rgb(0x9d, 0xd8, 0xc7)));
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

    fn about_window(&mut self, ctx: &egui::Context) {
        let mut open = self.about_open;
        egui::Window::new(t("about.title"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(470.0)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(RichText::new(APP_TITLE).size(16.0).color(TEXT));
                ui.label(
                    RichText::new(t("about.subtitle"))
                        .size(11.0)
                        .color(DIM),
                );
                ui.add_space(10.0);
                section(ui, &t("about.tech"));
                kv(ui, &t("about.lang"), "Rust");
                kv(ui, &t("about.ui"), &t("about.value.ui"));
                kv(ui, &t("about.drawing"), &t("about.value.drawing"));
                kv(ui, &t("about.files"), &t("about.value.designs"));
                kv(ui, &t("about.wire"), &t("about.wire.value"));
                ui.add_space(10.0);
                section(ui, &t("about.meaning"));
                for line in [
                    t("about.p1"),
                    t("about.p2"),
                    t("about.p3"),
                    t("about.p4"),
                ] {
                    ui.label(RichText::new(line).size(11.5).color(MUTED));
                    ui.add_space(4.0);
                }
            });
        self.about_open = open;
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }
        let (undo, redo, del, dup, fit, save, new, example, open_dlg) = ctx.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::CTRL, egui::Key::Z),
                i.consume_key(egui::Modifiers::CTRL, egui::Key::Y),
                i.consume_key(egui::Modifiers::NONE, egui::Key::Delete),
                i.consume_key(egui::Modifiers::CTRL, egui::Key::D),
                i.consume_key(egui::Modifiers::NONE, egui::Key::F),
                i.consume_key(egui::Modifiers::CTRL, egui::Key::S),
                i.consume_key(egui::Modifiers::CTRL, egui::Key::N),
                i.consume_key(egui::Modifiers::CTRL, egui::Key::E),
                i.consume_key(egui::Modifiers::CTRL, egui::Key::O),
            )
        });
        if undo {
            self.do_undo();
        }
        if redo {
            self.do_redo();
        }
        if del {
            self.delete_selected();
        }
        if dup {
            self.duplicate();
        }
        if fit {
            self.view.needs_fit = true;
        }
        if save {
            self.save_quick();
        }
        if new {
            self.push_undo();
            self.project = Project::default();
            self.project.name = t("name.untitled");
            self.sel = None;
            self.sim.idx = 0;
            self.rebuild();
            self.view.needs_fit = true;
            self.say(t("msg.new"));
        }
        if example {
            self.load_example();
        }
        if open_dlg {
            self.dialog = std::mem::replace(&mut self.dialog, io::FileDialog::new())
                .ask_open(&t("title.open_design"), "json", &t("filter.designs"));
        }
        let tool = ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::NONE, egui::Key::V) {
                Some(Tool::Select)
            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::R) {
                Some(Tool::FillRect)
            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::E) {
                Some(Tool::FillEllipse)
            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::L) {
                Some(Tool::RunPath)
            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::S) {
                Some(Tool::SatinPath)
            } else if i.consume_key(egui::Modifiers::NONE, egui::Key::H) {
                Some(Tool::Pan)
            } else {
                None
            }
        });
        if let Some(t) = tool {
            self.tool = t;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) && !ctx.wants_keyboard_input() {
            self.sim.playing = !self.sim.playing;
        }
    }

    fn tick_toast(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        if let Some((_, left)) = &mut self.toast {
            *left -= dt;
            if *left <= 0.0 {
                self.toast = None;
            }
        }
    }

    fn tick_sim(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        if self.sim.playing && !self.seq.is_empty() {
            let per_frame = self.project.stitches_per_second() * self.sim.mult * dt;
            self.sim.acc += per_frame;
            let whole = self.sim.acc.floor() as usize;
            if whole > 0 {
                self.sim.acc -= whole as f32;
                self.sim.idx = (self.sim.idx + whole).min(self.seq.len());
                if self.sim.idx >= self.seq.len() {
                    self.sim.playing = false;
                }
            }
        }
    }

    fn screenshot_tick(&mut self, ctx: &egui::Context) {
        let Some(shot) = &mut self.shot else {
            return;
        };
        shot.frames += 1;
        if shot.frames >= 120 && !shot.requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
            shot.requested = true;
        }
        if shot.requested {
            let image = ctx.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            if let Some(img) = image {
                let (w, h) = (img.width() as u32, img.height() as u32);
                let mut buf = Vec::with_capacity((w * h * 4) as usize);
                for px in &img.pixels {
                    buf.extend_from_slice(&px.to_array());
                }
                if let Some(dir) = shot.path.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                match image::save_buffer(&shot.path, &buf, w, h, image::ColorType::Rgba8) {
                    Ok(()) => eprintln!("screenshot written to {}", shot.path.display()),
                    Err(e) => eprintln!("screenshot failed: {e}"),
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if shot.frames > 600 {
                eprintln!("screenshot timed out");
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);
        if self.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.shot.is_some() || self.sim.playing || self.toast.is_some() {
            ctx.request_repaint();
        }
        self.screenshot_tick(ctx);
    }
}

impl App {
    /// Draw one frame of the whole window. Kept apart from `update` so a test
    /// can render the interface without a screen.
    fn draw(&mut self, ctx: &egui::Context) {
        self.tick_toast(ctx);
        self.tick_sim(ctx);
        self.shortcuts(ctx);
        self.chrome(ctx);
        self.bottom(ctx);
        self.left_panel(ctx);
        self.right_panel(ctx);
        self.canvas(ctx);
        self.handle_file_pick();
        self.dialog.show(ctx);
        self.help_window(ctx);
        self.about_window(ctx);
    }
}

// ------------------------------------------------------------------- helpers

/// A number the person can drag or type, drawn as a visible field so it
/// reads as an input and not as static text.
fn num_field(
    ui: &mut egui::Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    speed: f64,
    width: f32,
    suffix: &str,
    tip: &str,
) -> egui::Response {
    let out = egui::Frame::none()
        .fill(FIELD)
        .stroke(Stroke::new(1.0, LINE))
        .rounding(Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.add_sized(
                Vec2::new(width.max(28.0), 17.0),
                egui::DragValue::new(value)
                    .speed(speed)
                    .range(range)
                    .fixed_decimals(1)
                    .suffix(suffix),
            )
        })
        .inner;
    out.on_hover_text(tip)
}

fn mm_snap(p: [f32; 2], on: bool) -> [f32; 2] {
    if on {
        [p[0].round(), p[1].round()]
    } else {
        p
    }
}

/// How dense the hoop rulers are: (tick spacing in mm, number spacing in mm).
/// One-millimetre ticks are used as soon as they are at least 3 px apart;
/// below that they would read as a solid line, so the scale coarsens.
fn ruler_steps(zoom: f32) -> (i32, i32) {
    let tick = if zoom >= 3.0 {
        1
    } else if zoom >= 1.2 {
        5
    } else {
        10
    };
    let label = if zoom * 10.0 >= 30.0 {
        10
    } else if zoom * 20.0 >= 30.0 {
        20
    } else {
        50
    };
    (tick, label.max(tick))
}

/// Length and colour of one ruler tick: tall and bright every ten
/// millimetres, medium every five, short for a single millimetre.
fn tick_style(v: i32) -> (f32, Stroke) {
    let len = if v % 10 == 0 {
        9.5
    } else if v % 5 == 0 {
        6.5
    } else {
        3.5
    };
    let alpha = if v % 10 == 0 {
        170
    } else if v % 5 == 0 {
        110
    } else {
        62
    };
    let col = if v == 0 {
        ACCENT
    } else {
        Color32::from_white_alpha(alpha)
    };
    (len, Stroke::new(1.0, col))
}

fn shape_word(o: &Obj) -> String {
    match o.shape {
        ShapeKind::Rect => t("shape.rect"),
        ShapeKind::Ellipse => t("shape.ellipse"),
        ShapeKind::Path => t("shape.path"),
        ShapeKind::Stitches => t("shape.stitches"),
    }
}

fn dashed(p: &egui::Painter, a: Pos2, b: Pos2, stroke: Stroke, dash: f32) {
    let d = b - a;
    let len = d.length();
    if len < 0.5 {
        return;
    }
    let dir = d / len;
    let mut t = 0.0;
    while t < len {
        let t1 = (t + dash).min(len);
        p.line_segment([a + dir * t, a + dir * t1], stroke);
        t += dash * 2.0;
    }
}

fn fmt_num(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(*c);
    }
    out
}

fn fmt_time(min: f32) -> String {
    let secs = (min * 60.0).round().max(0.0) as u32;
    if secs >= 3600 {
        format!(
            "{}:{:02}:{:02}",
            secs / 3600,
            (secs % 3600) / 60,
            secs % 60
        )
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let t = out.trim_matches('-').to_string();
    if t.is_empty() {
        "design".into()
    } else {
        t
    }
}

fn dir_hint(ui: &mut egui::Ui) {
    ui.label(
        RichText::new(t("tip.satin"))
            .size(11.0)
            .color(DIM),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_menu_loads_the_eight_centimetre_flower() {
        let mut app = App::new(None);
        app.project = Project::default();
        app.rebuild();
        assert_eq!(app.stats.stitches, 0);

        app.load_example();

        assert!((app.stats.w - 80.0).abs() < 0.05, "size {}", app.stats.w);
        assert!((app.stats.h - 80.0).abs() < 0.05, "size {}", app.stats.h);
        assert!(app.stats.stitches > 2000);
        assert!(app.project.name.contains("8 × 8"), "{}", app.project.name);
        assert!(app.toast.is_some(), "loading the example should say so");
        assert!(app.sim.idx == 0 && !app.sim.playing);
    }

    #[test]
    fn hoop_rulers_mark_every_millimetre_when_there_is_room() {
        assert_eq!(ruler_steps(10.0), (1, 10));
        assert_eq!(ruler_steps(3.0), (1, 10));
        assert_eq!(ruler_steps(2.0), (5, 20));
        assert_eq!(ruler_steps(0.5), (10, 50));

        let (ten, _) = tick_style(10);
        let (five, _) = tick_style(5);
        let (one, _) = tick_style(1);
        assert!(ten > five && five > one, "tick lengths {ten} {five} {one}");
        assert!(one > 0.0);
    }

    #[test]
    fn time_is_shown_in_minutes_and_seconds() {
        assert_eq!(fmt_time(0.0), "0:00");
        assert_eq!(fmt_time(1.5), "1:30");
        assert_eq!(fmt_time(12.5), "12:30");
        assert_eq!(fmt_time(75.0), "1:15:00");
    }

    #[test]
    fn changing_the_speed_updates_the_estimated_time() {
        let mut app = App::new(None);
        let slow = app.stats.minutes;
        assert!(slow > 0.0);
        app.project.set_stitches_per_second(25.0);
        app.rebuild();
        assert!(
            (app.stats.minutes - slow / 2.0).abs() < 0.01,
            "{} vs {}",
            app.stats.minutes,
            slow / 2.0
        );
    }

    fn collect_text(shape: &egui::Shape, out: &mut Vec<(String, Rect)>) {
        match shape {
            egui::Shape::Text(t) => {
                let r = Rect::from_min_size(t.pos, t.galley.size());
                out.push((t.galley.text().to_string(), r));
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    collect_text(s, out);
                }
            }
            _ => {}
        }
    }

    const SCREEN: Vec2 = Vec2::new(1280.0, 820.0);

    /// Draw the window off-screen and return every piece of text it put on it
    /// with the rectangle it occupies.
    fn rendered(app: &mut App) -> Vec<(String, Rect)> {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
            ..Default::default()
        };
        let mut out = Vec::new();
        for _ in 0..3 {
            let frame = ctx.run(input.clone(), |c| app.draw(c));
            out.clear();
            for cs in &frame.shapes {
                collect_text(&cs.shape, &mut out);
            }
        }
        out
    }

    /// Every drawn word, for tests that only care about wording.
    fn rendered_text(app: &mut App) -> Vec<String> {
        rendered(app).into_iter().map(|(s, _)| s).collect()
    }

    #[test]
    fn the_window_shows_editable_hoop_sides_and_speed() {
        let mut app = App::new(None);
        let texts = rendered_text(&mut app);
        for want in [
            t("hoop.preset"),
            t("hoop.width"),
            t("hoop.height"),
            t("summary.rate"),
        ] {
            assert!(
                texts.iter().any(|x| x.trim() == want.trim()),
                "the window never draws {want:?}"
            );
        }
        // The speed field shows its value with the unit, e.g. "12.5 st/s".
        let unit = t("unit.sps");
        assert!(
            texts.iter().any(|x| x.contains(&unit)),
            "the speed field never draws its unit {unit:?}"
        );
        // The estimated time is on the summary card and follows the speed.
        let time = fmt_time(app.stats.minutes);
        assert!(
            texts.iter().any(|x| x.trim() == time),
            "the window never draws the sew time {time:?}"
        );
    }

    #[test]
    fn editing_the_speed_redraws_the_time() {
        let mut app = App::new(None);
        app.project.set_stitches_per_second(4.0);
        app.rebuild();
        let slow = fmt_time(app.stats.minutes);
        let texts = rendered_text(&mut app);
        assert!(
            texts.iter().any(|x| x.trim() == slow),
            "slow time {slow:?} is not on screen"
        );

        app.project.set_stitches_per_second(40.0);
        app.rebuild();
        let fast = fmt_time(app.stats.minutes);
        assert_ne!(slow, fast, "the time should change with the speed");
        let texts = rendered_text(&mut app);
        assert!(
            texts.iter().any(|x| x.trim() == fast),
            "new time {fast:?} is not on screen"
        );
    }

    #[test]
    fn an_odd_hoop_size_is_drawn_and_reported() {
        let mut app = App::new(None);
        app.project.set_hoop_size(173.5, 261.0);
        app.rebuild();
        let texts = rendered_text(&mut app);
        let size = format!("{} × {} mm", model::fmt_mm(173.5), model::fmt_mm(261.0));
        assert!(
            texts.iter().any(|x| x.contains(&size)),
            "{size:?} is not shown in the plan or status bar"
        );
        assert_eq!(app.project.hoop_name(), size);
    }

    #[test]
    fn every_language_names_the_flower_example() {
        for code in ["en", "es", "eu"] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("lang")
                .join(format!("{code}.json"));
            let text = std::fs::read_to_string(&path).expect("language file");
            let map: std::collections::BTreeMap<String, String> =
                serde_json::from_str(&text).expect("valid language file");
            let label = map.get("menu.file.example").expect("menu.file.example");
            assert!(label.contains("8 × 8 cm"), "{code}: {label}");
            assert!(map.contains_key("summary.size_cm"), "{code}: summary.size_cm");
            assert!(!map["help.example"].is_empty(), "{code}: help.example");
            assert!(!map["msg.example"].is_empty(), "{code}: msg.example");
        }
    }
}
