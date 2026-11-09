//! File formats and the in-app file browser.

use crate::lang::{t, tf};
use crate::model::{self, El, Project, Step, THREADS};
use std::fs;
use std::path::{Path, PathBuf};

fn err_write(path: &Path, e: impl std::fmt::Display) -> String {
    tf(
        "err.write",
        &[
            ("path", &path.display().to_string()),
            ("e", &e.to_string()),
        ],
    )
}

fn err_read(path: &Path, e: impl std::fmt::Display) -> String {
    tf(
        "err.read",
        &[
            ("path", &path.display().to_string()),
            ("e", &e.to_string()),
        ],
    )
}

pub fn save_project(p: &Project, path: &Path) -> Result<(), String> {
    let txt = serde_json::to_string_pretty(p).map_err(|e| e.to_string())?;
    fs::write(path, txt).map_err(|e| err_write(path, e))
}

pub fn load_project(path: &Path) -> Result<Project, String> {
    let txt = fs::read_to_string(path).map_err(|e| err_read(path, e))?;
    serde_json::from_str(&txt).map_err(|e| tf("err.not_design", &[("e", &e.to_string())]))
}

// ------------------------------------------------------------------- export

pub fn export_svg(p: &Project, path: &Path) -> Result<(), String> {
    let (hw, hh) = p.hoop_size();
    let mut s = String::new();
    s.push_str(&format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{hw}mm\" height=\"{hh}mm\" viewBox=\"0 0 {hw} {hh}\">\n"
    ));
    s.push_str(&format!(
        "<rect width=\"{hw}\" height=\"{hh}\" fill=\"#f2eee5\"/>\n<rect x=\"0.5\" y=\"0.5\" width=\"{:.1}\" height=\"{:.1}\" fill=\"none\" stroke=\"#b9b2a2\" stroke-width=\"0.3\"/>\n",
        hw - 1.0,
        hh - 1.0
    ));
    let ox = hw / 2.0;
    let oy = hh / 2.0;
    let seq = model::timeline(p);
    let mut cur: Option<(usize, u32)> = None;
    let mut d = String::new();
    let mut last: Option<[f32; 2]> = None;
    for step in &seq {
        if cur != Some((step.color, step.obj)) {
            if !d.is_empty() {
                let c = THREADS[cur.unwrap().0.min(THREADS.len() - 1)].1;
                s.push_str(&format!(
                    "<path d=\"{d}\" fill=\"none\" stroke=\"rgb({},{},{})\" stroke-width=\"0.45\" stroke-linecap=\"round\"/>\n",
                    c[0], c[1], c[2]
                ));
                d.clear();
            }
            cur = Some((step.color, step.obj));
        }
        match step.el {
            El::Move(pt) => {
                d.push_str(&format!("M {:.2} {:.2} ", ox + pt[0], oy - pt[1]));
                last = Some(pt);
            }
            El::Stitch(pt) => {
                let _ = last;
                d.push_str(&format!("L {:.2} {:.2} ", ox + pt[0], oy - pt[1]));
                last = Some(pt);
            }
        }
    }
    if !d.is_empty() {
        let c = THREADS[cur.map(|c| c.0).unwrap_or(0).min(THREADS.len() - 1)].1;
        s.push_str(&format!(
            "<path d=\"{d}\" fill=\"none\" stroke=\"rgb({},{},{})\" stroke-width=\"0.45\" stroke-linecap=\"round\"/>\n",
            c[0], c[1], c[2]
        ));
    }
    s.push_str("</svg>\n");
    fs::write(path, s).map_err(|e| err_write(path, e))
}

pub fn export_csv(p: &Project, path: &Path) -> Result<(), String> {
    let seq = model::timeline(p);
    let mut s = String::from("index,x_mm,y_mm,type,thread\n");
    for (i, st) in seq.iter().enumerate() {
        let (pt, kind) = match st.el {
            El::Move(p) => (p, "jump"),
            El::Stitch(p) => (p, "stitch"),
        };
        let name = crate::lang::thread_name(st.color.min(THREADS.len() - 1));
        s.push_str(&format!("{},{:.2},{:.2},{},{}\n", i + 1, pt[0], pt[1], kind, name));
    }
    fs::write(path, s).map_err(|e| err_write(path, e))
}

/// One DST record: 3 bytes, deltas in 0.1 mm, each axis decomposed into
/// 81/27/9/3/1 steps with a sign bit per step.
#[allow(unused_assignments)]
fn dst_encode(x: i32, y: i32, flags: u32) -> [u8; 3] {
    let mut b0 = 0u8;
    let mut b1 = 0u8;
    let mut b2 = 0u8;
    if flags == 1 || flags == 2 {
        if flags == 2 {
            b2 |= 0x80; // jump
        }
        b2 |= 0x03;
        let mut xs = x;
        let mut ys = y;
        if xs > 40 {
            b2 |= 0x04;
            xs -= 81;
        }
        if xs < -40 {
            b2 |= 0x08;
            xs += 81;
        }
        if xs > 13 {
            b1 |= 0x04;
            xs -= 27;
        }
        if xs < -13 {
            b1 |= 0x08;
            xs += 27;
        }
        if xs > 4 {
            b0 |= 0x04;
            xs -= 9;
        }
        if xs < -4 {
            b0 |= 0x08;
            xs += 9;
        }
        if xs > 1 {
            b1 |= 0x01;
            xs -= 3;
        }
        if xs < -1 {
            b1 |= 0x02;
            xs += 3;
        }
        if xs > 0 {
            b0 |= 0x01;
            xs -= 1;
        }
        if xs < 0 {
            b0 |= 0x02;
            xs += 1;
        }
        if ys > 40 {
            b2 |= 0x20;
            ys -= 81;
        }
        if ys < -40 {
            b2 |= 0x10;
            ys += 81;
        }
        if ys > 13 {
            b1 |= 0x20;
            ys -= 27;
        }
        if ys < -13 {
            b1 |= 0x10;
            ys += 27;
        }
        if ys > 4 {
            b0 |= 0x20;
            ys -= 9;
        }
        if ys < -4 {
            b0 |= 0x10;
            ys += 9;
        }
        if ys > 1 {
            b1 |= 0x80;
            ys -= 3;
        }
        if ys < -1 {
            b1 |= 0x40;
            ys += 3;
        }
        if ys > 0 {
            b0 |= 0x80;
            ys -= 1;
        }
        if ys < 0 {
            b0 |= 0x40;
            ys += 1;
        }
    } else if flags == 3 {
        b2 = 0b1100_0011; // colour change
    } else if flags == 4 {
        b2 = 0b1111_0011; // end
    }
    [b0, b1, b2]
}

/// Universal three-byte stitch format, units of 0.1 mm.
pub fn export_dst(p: &Project, path: &Path) -> Result<(), String> {
    let seq: Vec<Step> = model::timeline(p);
    let mut body: Vec<u8> = Vec::new();
    let mut last = (0i32, 0i32);
    let mut color = usize::MAX;
    let mut stitches = 0usize;
    let mut colors = 0usize;
    let mut min = (i32::MAX, i32::MAX);
    let mut max = (i32::MIN, i32::MIN);

    for step in &seq {
        let (pt, is_jump) = match step.el {
            El::Move(pt) => (pt, true),
            El::Stitch(pt) => (pt, false),
        };
        let x = (pt[0] * 10.0).round() as i32;
        let y = (pt[1] * 10.0).round() as i32;
        min.0 = min.0.min(x);
        min.1 = min.1.min(y);
        max.0 = max.0.max(x);
        max.1 = max.1.max(y);
        if step.color != color {
            color = step.color;
            colors += 1;
            if colors > 1 {
                body.extend_from_slice(&dst_encode(0, 0, 3));
            }
        }
        let mut dx = x - last.0;
        let mut dy = y - last.1;
        while dx != 0 || dy != 0 {
            let sx = dx.clamp(-121, 121);
            let sy = dy.clamp(-121, 121);
            body.extend_from_slice(&dst_encode(sx, sy, if is_jump { 2 } else { 1 }));
            dx -= sx;
            dy -= sy;
            if !is_jump {
                stitches += 1;
            }
        }
        last = (x, y);
    }
    // three end records, as machines expect
    for _ in 0..3 {
        body.extend_from_slice(&dst_encode(0, 0, 4));
    }

    let mut header = String::new();
    header.push_str(&format!("LA:{:<16}\r", trunc(&p.name, 16)));
    header.push_str(&format!("ST:{:>7}\r", stitches));
    header.push_str(&format!("CO:{:>3}\r", colors));
    header.push_str(&format!("+X:{:>5}\r", 0.max(max.0)));
    header.push_str(&format!("-X:{:>5}\r", 0.max(-min.0)));
    header.push_str(&format!("+Y:{:>5}\r", 0.max(max.1)));
    header.push_str(&format!("-Y:{:>5}\r", 0.max(-min.1)));
    header.push_str(&format!("AX:+{:>5}\r", 0.max(last.0)));
    header.push_str(&format!("AY:+{:>5}\r", 0.max(last.1)));
    header.push_str("MX:+    0\r");
    header.push_str("MY:+    0\r");
    header.push_str("PD:******\r");
    let mut bytes = vec![0x20u8; 512];
    let taken: Vec<u8> = header.bytes().take(510).collect();
    bytes[..taken.len()].copy_from_slice(&taken);
    bytes[taken.len().min(511)] = 0x1A;
    let mut out = bytes;
    out.extend_from_slice(&body);
    fs::write(path, out).map_err(|e| err_write(path, e))
}

fn trunc(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Read the three-byte delta of one DST record, as `DSTStitchFormat`'s
/// decode helper does at IMPORT.DLL 0x1002dba0.
///
/// The helper looks each byte up in a 256-entry table of `(dx, dy)` pairs.
/// `0x7F` is a stop marker in that table: when the looked-up pair starts with
/// `0x7F` the helper returns 0 straight away, so that byte and every byte
/// after it contribute nothing. `LoadStitch` (0x1002ddc0) turns that 0 into a
/// non-zero result and produces no stitch, so the whole record is dropped —
/// which is why this returns `None` rather than a partial delta.
/// The bit sums below are the same table for every other value.
pub fn dst_decode(rec: &[u8]) -> Option<(i32, i32)> {
    let (b0, b1, b2) = (rec[0], rec[1], rec[2]);
    if b0 == 0x7F || b1 == 0x7F || b2 == 0x7F {
        return None;
    }
    let mut x = 0i32;
    let mut y = 0i32;
    add_dst_byte(b0, 1, 9, &mut x, &mut y);
    add_dst_byte(b1, 3, 27, &mut x, &mut y);
    add_dst_byte(b2, 0, 81, &mut x, &mut y);
    Some((x, y))
}

/// One byte's `(dx, dy)` pair from the original's table: bits 0/1 are the
/// small step, bits 2/3 the large one, on x and then y.
fn add_dst_byte(b: u8, small: i32, big: i32, x: &mut i32, y: &mut i32) {
    if b & 0x01 != 0 { *x += small; }
    if b & 0x02 != 0 { *x -= small; }
    if b & 0x04 != 0 { *x += big; }
    if b & 0x08 != 0 { *x -= big; }
    if b & 0x80 != 0 { *y += small; }
    if b & 0x40 != 0 { *y -= small; }
    if b & 0x20 != 0 { *y += big; }
    if b & 0x10 != 0 { *y -= big; }
}

/// Thread colours used for imported files, which carry no colour data.
const DST_PALETTE: [usize; 12] = [6, 17, 3, 15, 11, 18, 4, 8, 14, 21, 25, 27];

/// Read a DST stitch file into a design. Every stitch keeps its exact point.
pub fn import_dst(path: &Path) -> Result<Project, String> {
    let bytes = fs::read(path).map_err(|e| err_read(path, e))?;
    if bytes.len() < 515 {
        return Err(t("err.dst_short"));
    }
    let mut chains: Vec<(usize, Vec<[f32; 2]>)> = Vec::new();
    let mut cur: Vec<[f32; 2]> = Vec::new();
    let mut color = 0usize;
    let (mut x, mut y) = (0i32, 0i32);
    let mut i = 512;
    while i + 3 <= bytes.len() {
        let rec = [bytes[i], bytes[i + 1], bytes[i + 2]];
        i += 3;
        // LoadStitch ends only on the exact record 00 00 F3.
        if rec == [0x00, 0x00, 0xF3] {
            break;
        }
        // Any byte whose table entry is the 0x7F stop marker makes the decode
        // helper fail; LoadStitch then returns non-zero without a stitch, which
        // ends the caller's read loop.
        let Some((dx, dy)) = dst_decode(&rec) else {
            break;
        };
        x += dx;
        y += dy;
        let pt = [x as f32 / 10.0, y as f32 / 10.0];
        // The top two bits pick the record kind, exactly as LoadStitch's
        // `byte & 0xC0` switch does: 0x40 colour change, 0x80 jump,
        // 0xC0 both; anything else is an ordinary stitch.
        match rec[2] & 0xC0 {
            0x40 | 0xC0 => {
                if cur.len() >= 2 {
                    chains.push((color, std::mem::take(&mut cur)));
                } else {
                    cur.clear();
                }
                color += 1;
                cur.push(pt);
            }
            0x80 => {
                if cur.len() >= 2 {
                    chains.push((color, std::mem::take(&mut cur)));
                } else {
                    cur.clear();
                }
                cur.push(pt);
            }
            _ => cur.push(pt),
        }
    }
    if cur.len() >= 2 {
        chains.push((color, cur));
    }
    let total: usize = chains.iter().map(|c| c.1.len()).sum();
    if total < 2 {
        return Err(t("err.dst_none"));
    }

    let mut p = Project {
        name: path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| t("name.imported")),
        hoop: 0,
        ..Project::default()
    };
    p.set_hoop_preset(model::largest_preset());
    for (c, pts) in chains {
        p.push(model::Obj::new_stitches(0, DST_PALETTE[c % DST_PALETTE.len()], pts));
    }
    // Pick the smallest hoop that the imported design fits in, by the same
    // rule the original uses (see `model::best_preset_for`).
    if let Some(b) = p.bounds() {
        let (w, h) = (b.2 - b.0, b.3 - b.1);
        if let Some(i) = model::best_preset_for(w, h) {
            p.set_hoop_preset(i);
        }
    }
    Ok(p)
}

// -------------------------------------------------------------- file browser

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DialogMode {
    Open,
    Save,
}

pub struct FileDialog {
    pub open: bool,
    pub mode: DialogMode,
    pub title: String,
    pub ext: String,
    pub filter_desc: String,
    pub dir: PathBuf,
    pub name: String,
    pub error: Option<String>,
    pub picked: Option<PathBuf>,
    entries: Vec<(String, bool)>,
    needs_refresh: bool,
}

impl FileDialog {
    pub fn new() -> Self {
        let dir = default_dir();
        Self {
            open: false,
            mode: DialogMode::Save,
            title: String::new(),
            ext: "json".into(),
            filter_desc: t("filter.design"),
            dir: dir.clone(),
            name: String::new(),
            error: None,
            picked: None,
            entries: Vec::new(),
            needs_refresh: true,
        }
    }

    pub fn ask_open(mut self, title: &str, ext: &str, desc: &str) -> Self {
        self.open = true;
        self.mode = DialogMode::Open;
        self.title = title.into();
        self.ext = ext.into();
        self.filter_desc = desc.into();
        self.name.clear();
        self.picked = None;
        self.error = None;
        self.needs_refresh = true;
        self
    }

    pub fn ask_save(mut self, title: &str, ext: &str, desc: &str, name: &str) -> Self {
        self.open = true;
        self.mode = DialogMode::Save;
        self.title = title.into();
        self.ext = ext.into();
        self.filter_desc = desc.into();
        self.name = name.into();
        self.picked = None;
        self.error = None;
        self.needs_refresh = true;
        self
    }

    fn refresh(&mut self) {
        self.entries.clear();
        let mut dirs: Vec<(String, bool)> = Vec::new();
        let mut files: Vec<(String, bool)> = Vec::new();
        if let Ok(rd) = fs::read_dir(&self.dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                let is_dir = e.path().is_dir();
                if is_dir {
                    dirs.push((name, true));
                } else if name.to_lowercase().ends_with(&format!(".{}", self.ext)) {
                    files.push((name, false));
                }
            }
        }
        dirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        files.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        self.entries = dirs;
        self.entries.extend(files);
        self.needs_refresh = false;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        if self.needs_refresh {
            self.refresh();
        }
        let mut open = self.open;
        let mut action: Option<PathBuf> = None;
        egui::Window::new(self.title.clone())
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(560.0)
            .default_height(430.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(t("dialog.up")).clicked() {
                        if let Some(parent) = self.dir.parent().map(|p| p.to_path_buf()) {
                            self.dir = parent;
                            self.needs_refresh = true;
                        }
                    }
                    ui.label(
                        egui::RichText::new(self.dir.display().to_string())
                            .size(11.0)
                            .color(crate::theme::MUTED),
                    );
                });
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(tf(
                        "dialog.showing",
                        &[("what", &self.filter_desc.to_lowercase())],
                    ))
                    .size(11.0)
                    .color(crate::theme::DIM),
                );
                egui::Frame::none()
                    .fill(crate::theme::FIELD)
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::same(4.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(250.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                let entries = self.entries.clone();
                                if entries.is_empty() {
                                    ui.add_space(10.0);
                                    ui.label(
                                        egui::RichText::new(t("dialog.empty"))
                                            .size(12.0)
                                            .color(crate::theme::DIM),
                                    );
                                }
                                for (name, is_dir) in entries {
                                    let label = if is_dir {
                                        format!("{}/", name)
                                    } else {
                                        name.clone()
                                    };
                                    let selected = !is_dir && self.name == name;
                                    let txt = egui::RichText::new(label).size(12.5).color(if is_dir {
                                        crate::theme::BLUE
                                    } else {
                                        crate::theme::TEXT
                                    });
                                    let resp = ui.selectable_label(selected, txt);
                                    if resp.clicked() {
                                        if is_dir {
                                            self.dir = self.dir.join(&name);
                                            self.needs_refresh = true;
                                        } else {
                                            self.name = name.clone();
                                        }
                                    }
                                    if resp.double_clicked() && !is_dir {
                                        action = Some(self.dir.join(&name));
                                    }
                                }
                            });
                    });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(t("dialog.name")).size(12.0).color(crate::theme::MUTED));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.name)
                            .desired_width(300.0)
                            .hint_text(format!("my-design.{}", self.ext)),
                    );
                    ui.label(
                        egui::RichText::new(format!(".{}", self.ext))
                            .size(12.0)
                            .color(crate::theme::DIM),
                    );
                });
                if let Some(err) = &self.error {
                    ui.label(egui::RichText::new(err).size(11.5).color(crate::theme::DANGER));
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let label = if self.mode == DialogMode::Save {
                        t("dialog.save")
                    } else {
                        t("dialog.open")
                    };
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new(label).size(13.0).color(egui::Color32::from_rgb(0x0d, 0x27, 0x21)),
                            )
                            .fill(crate::theme::ACCENT)
                            .stroke(egui::Stroke::NONE)
                            .min_size(egui::vec2(92.0, 28.0)),
                        )
                        .clicked()
                    {
                        let mut nm = self.name.trim().to_string();
                        if nm.is_empty() {
                            self.error = Some(t("dialog.name_first"));
                        } else {
                            if !nm.to_lowercase().ends_with(&format!(".{}", self.ext)) {
                                nm.push('.');
                                nm.push_str(&self.ext);
                            }
                            action = Some(self.dir.join(nm));
                        }
                    }
                    if ui.button(t("dialog.cancel")).clicked() {
                        self.open = false;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(t("dialog.tip"))
                                .size(10.5)
                                .color(crate::theme::DIM),
                        );
                    });
                });
            });
        self.open = open && self.picked.is_none();
        if let Some(p) = action {
            self.picked = Some(p);
            self.open = false;
        }
    }
}

pub fn default_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        let docs = PathBuf::from(home).join("Documents");
        if docs.is_dir() {
            return docs;
        }
        return PathBuf::from(std::env::var_os("HOME").unwrap());
    }
    PathBuf::from(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::demo_project;

    fn out_dir() -> PathBuf {
        let d = std::env::temp_dir().join("stitch-studio-tests");
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn project_round_trips() {
        let p = demo_project();
        let f = out_dir().join("round-trip.json");
        save_project(&p, &f).unwrap();
        let back = load_project(&f).unwrap();
        assert_eq!(back.objects.len(), p.objects.len());
        assert_eq!(back.name, p.name);
        assert_eq!(back.hoop, p.hoop);
    }

    fn decode(rec: &[u8]) -> (i32, i32) {
        let (b0, b1, b2) = (rec[0], rec[1], rec[2]);
        let mut x = 0i32;
        if b0 & 0x01 != 0 { x += 1; }
        if b0 & 0x02 != 0 { x -= 1; }
        if b1 & 0x01 != 0 { x += 3; }
        if b1 & 0x02 != 0 { x -= 3; }
        if b0 & 0x04 != 0 { x += 9; }
        if b0 & 0x08 != 0 { x -= 9; }
        if b1 & 0x04 != 0 { x += 27; }
        if b1 & 0x08 != 0 { x -= 27; }
        if b2 & 0x04 != 0 { x += 81; }
        if b2 & 0x08 != 0 { x -= 81; }
        let mut y = 0i32;
        if b0 & 0x80 != 0 { y += 1; }
        if b0 & 0x40 != 0 { y -= 1; }
        if b1 & 0x80 != 0 { y += 3; }
        if b1 & 0x40 != 0 { y -= 3; }
        if b0 & 0x20 != 0 { y += 9; }
        if b0 & 0x10 != 0 { y -= 9; }
        if b1 & 0x20 != 0 { y += 27; }
        if b1 & 0x10 != 0 { y -= 27; }
        if b2 & 0x20 != 0 { y += 81; }
        if b2 & 0x10 != 0 { y -= 81; }
        (x, y)
    }

    #[test]
    fn dst_export_matches_the_stitch_plan() {
        let p = demo_project();
        let f = out_dir().join("demo.dst");
        export_dst(&p, &f).unwrap();
        let bytes = std::fs::read(&f).unwrap();
        assert!(bytes.len() > 512 + 100);
        assert!(bytes.starts_with(b"LA:"));
        assert_eq!(bytes[511], 0x20);
        assert_eq!((bytes.len() - 512) % 3, 0, "records are 3 bytes each");
        for i in (512..bytes.len()).step_by(3) {
            assert_eq!(bytes[i + 2] & 0x03, 0x03, "record marker byte");
        }
        // walk the records and confirm they land on the planned points
        let mut here = (0i32, 0i32);
        let mut read: Vec<(i32, i32)> = Vec::new();
        for i in (512..bytes.len()).step_by(3) {
            let b2 = bytes[i + 2];
            if b2 == 0b1111_0011 {
                continue; // end
            }
            if b2 == 0b1100_0011 {
                continue; // colour change
            }
            let (dx, dy) = decode(&bytes[i..i + 3]);
            here.0 += dx;
            here.1 += dy;
            read.push(here);
        }
        let mut want: Vec<(i32, i32)> = Vec::new();
        let mut last = (0i32, 0i32);
        for st in model::timeline(&p) {
            let pt = match st.el {
                El::Move(p) | El::Stitch(p) => p,
            };
            let (x, y) = ((pt[0] * 10.0).round() as i32, (pt[1] * 10.0).round() as i32);
            if (x, y) != last {
                want.push((x, y));
                last = (x, y);
            }
        }
        assert_eq!(read.last(), want.last(), "decoded DST must finish on the last stitch");
        // Splitting a long move into several records inserts intermediate
        // points; every planned point must still be visited in order.
        let mut i = 0;
        for w in &want {
            while i < read.len() && read[i] != *w {
                i += 1;
            }
            assert!(i < read.len(), "planned point {w:?} missing from the DST");
            i += 1;
        }
        assert!(bytes.windows(3).any(|w| w[2] == 0b1111_0011));
    }

    #[test]
    fn dst_import_round_trips_the_design() {
        let p = demo_project();
        let f = out_dir().join("round-trip.dst");
        export_dst(&p, &f).unwrap();
        let q = import_dst(&f).unwrap();
        let a = model::stats(&p);
        let b = model::stats(&q);
        assert!(
            (a.stitches as i64 - b.stitches as i64).abs() <= 2,
            "stitches {} vs {}",
            a.stitches,
            b.stitches
        );
        let blocks = |p: &model::Project| {
            let seq = model::timeline(p);
            let mut n = 0;
            let mut last: Option<usize> = None;
            for s in &seq {
                if last != Some(s.color) {
                    n += 1;
                    last = Some(s.color);
                }
            }
            n
        };
        assert_eq!(
            blocks(&p),
            blocks(&q),
            "colour blocks should survive the round trip"
        );
        let pa = p.bounds().unwrap();
        let pb = q.bounds().unwrap();
        for (x, y) in [(pa.0, pb.0), (pa.1, pb.1), (pa.2, pb.2), (pa.3, pb.3)] {
            assert!((x - y).abs() < 0.5, "bounds differ: {x} vs {y}");
        }
        assert!(q.objects.len() >= 4);
        assert!(q
            .objects
            .iter()
            .all(|o| o.shape == model::ShapeKind::Stitches));
    }

    #[test]
    fn dst_decode_stops_at_a_0x7f_byte() {
        // The helper at IMPORT.DLL 0x1002dba0 returns 0 as soon as one of the
        // three table lookups starts with 0x7F, and LoadStitch then returns a
        // non-zero result instead of a stitch: no partial delta is produced.
        assert_eq!(dst_decode(&[0x7F, 0x00, 0x24]), None);
        assert_eq!(dst_decode(&[0x01, 0x7F, 0x24]), None);
        assert_eq!(dst_decode(&[0x01, 0x00, 0x7F]), None);
        // Ordinary records are unchanged: 0x04 in byte 1 is +27 on x.
        assert_eq!(dst_decode(&[0x00, 0x04, 0x00]), Some((27, 0)));
    }

    #[test]
    fn dst_import_treats_0x40_as_a_colour_change() {
        let mut bytes = vec![0x20u8; 512];
        bytes[..3].copy_from_slice(b"LA:");
        for rec in [
            [0x01, 0x00, 0x03], // stitch
            [0x01, 0x00, 0x03], // stitch
            [0x00, 0x00, 0x43], // 0x40: colour change
            [0x01, 0x00, 0x03], // stitch
            [0x01, 0x00, 0x03], // stitch
            [0x00, 0x00, 0xF3], // end
        ] {
            bytes.extend_from_slice(&rec);
        }
        let f = out_dir().join("colour-change.dst");
        fs::write(&f, &bytes).unwrap();
        let p = import_dst(&f).unwrap();
        assert_eq!(p.objects.len(), 2, "0x40 must start a second colour block");
    }

    #[test]
    fn dst_import_ends_only_on_the_exact_end_record() {
        let mut bytes = vec![0x20u8; 512];
        bytes[..3].copy_from_slice(b"LA:");
        for rec in [
            [0x01, 0x00, 0xF3], // NOT the end: the first two bytes are not 0
            [0x01, 0x00, 0x03], // stitch
            [0x01, 0x00, 0x03], // stitch
            [0x00, 0x00, 0xF3], // end
        ] {
            bytes.extend_from_slice(&rec);
        }
        let f = out_dir().join("not-the-end.dst");
        fs::write(&f, &bytes).unwrap();
        let p = import_dst(&f).unwrap();
        assert_eq!(p.objects.len(), 1);
        assert!(p.objects[0].path.len() >= 3, "the stitches after 0x...F3 must be read");
    }

    #[test]
    fn svg_and_csv_export() {
        let p = demo_project();
        let svg = out_dir().join("demo.svg");
        export_svg(&p, &svg).unwrap();
        let s = std::fs::read_to_string(&svg).unwrap();
        assert!(s.starts_with("<?xml"));
        assert!(s.contains("<path"));
        assert!(s.contains("</svg>"));
        let csv = out_dir().join("demo.csv");
        export_csv(&p, &csv).unwrap();
        let c = std::fs::read_to_string(&csv).unwrap();
        assert!(c.starts_with("index,x_mm,y_mm,type,thread"));
        assert!(c.lines().count() > 500);
    }
}
