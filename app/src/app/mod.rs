//! Application state, actions and the eframe application loop.

mod canvas;
mod chrome;
mod panels;
mod windows;

use crate::io;
use crate::lang::{t, tf};
use crate::model::{self, Obj, Project, Stats, Step};
use crate::ui::slug;
use eframe::egui;
use egui::{Pos2, Rect, Vec2};
use std::path::PathBuf;

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
        matches!(
            self,
            Tool::FillRect | Tool::FillEllipse | Tool::RunPath | Tool::SatinPath
        )
    }
}

enum Drag {
    None,
    Create {
        start: [f32; 2],
        cur: [f32; 2],
        path: Vec<[f32; 2]>,
    },
    Move {
        last: [f32; 2],
    },
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

pub(crate) struct App {
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

impl App {
    pub(crate) fn new(shot: Option<PathBuf>) -> Self {
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

    /// Reset the document without asking for a file name.
    fn new_project(&mut self) {
        self.push_undo();
        self.project = Project::default();
        self.project.name = t("name.untitled");
        self.sel = None;
        self.sim.idx = 0;
        self.rebuild();
        self.view.needs_fit = true;
    }

    fn ask_open_dialog(&mut self, title: &str, ext: &str, desc: &str) {
        self.dialog =
            std::mem::replace(&mut self.dialog, io::FileDialog::new()).ask_open(title, ext, desc);
    }

    fn ask_save_dialog(&mut self, title: &str, ext: &str, desc: &str, name: &str) {
        self.dialog = std::mem::replace(&mut self.dialog, io::FileDialog::new())
            .ask_save(title, ext, desc, name);
    }

    fn save_as(&mut self) {
        self.ask_save_dialog(
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
            self.save_as();
        }
        if new {
            self.new_project();
            self.say(t("msg.new"));
        }
        if example {
            self.load_example();
        }
        if open_dlg {
            self.ask_open_dialog(&t("title.open_design"), "json", &t("filter.designs"));
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

#[cfg(test)]
mod tests;
