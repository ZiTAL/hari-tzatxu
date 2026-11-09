//! Canvas painting, pointer interaction and hoop rulers.

use super::{App, Drag, Tool};
use crate::lang::t;
use crate::model::{self, El, Obj, StitchKind};
use crate::theme::{ACCENT, BG, DIM, FABRIC, FABRIC_EDGE, LINE, MUTED, TEXT};
use crate::ui::{dashed, mm_snap, ruler_steps, tick_style};
use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Rounding, Sense, Stroke, Vec2};

impl App {
    // ----------------------------------------------------------------- canvas

    pub(crate) fn canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(BG)
                    .inner_margin(egui::Margin::same(8.0)),
            )
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
                        let r = Rect::from_center_size(pos, galley.size() + Vec2::new(26.0, 14.0));
                        painter.rect_filled(
                            r,
                            Rounding::same(8.0),
                            Color32::from_rgb(0x24, 0x2c, 0x34),
                        );
                        painter.rect_stroke(r, Rounding::same(8.0), Stroke::new(1.0_f32, LINE));
                        painter.galley(Pos2::new(r.left() + 13.0, r.top() + 7.0), galley, TEXT);
                    }
                }
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
                            *cur = [mm[0], mm[1]];
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
                            Obj::new_rect(
                                0,
                                kind,
                                (x0 + x1) / 2.0,
                                (y0 + y1) / 2.0,
                                x1 - x0,
                                y1 - y0,
                            )
                        } else {
                            Obj::new_ellipse(
                                0,
                                kind,
                                (x0 + x1) / 2.0,
                                (y0 + y1) / 2.0,
                                x1 - x0,
                                y1 - y0,
                            )
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
                        let kind = if self.tool == Tool::SatinPath {
                            StitchKind::Satin
                        } else {
                            StitchKind::Run
                        };
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
                    Tool::FillEllipse => {
                        Obj::new_ellipse(0, StitchKind::Fill, mm[0], mm[1], 26.0, 22.0)
                    }
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
            Stroke::new(1.0_f32, Color32::from_white_alpha(26)),
        );

        // Horizontal scale, along the top and bottom edges.
        let mut v = (x0.ceil() as i32 / tick_mm) * tick_mm;
        let last = (hw / 2.0).floor() as i32;
        while v <= last {
            let sx = fabric.left() + (v as f32 - x0) * z;
            if sx >= rect.left() - 1.0 && sx <= rect.right() + 1.0 {
                let (len, stroke) = tick_style(v);
                p.line_segment(
                    [
                        Pos2::new(sx, fabric.top()),
                        Pos2::new(sx, fabric.top() - len),
                    ],
                    stroke,
                );
                p.line_segment(
                    [
                        Pos2::new(sx, fabric.bottom()),
                        Pos2::new(sx, fabric.bottom() + len),
                    ],
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
                    [
                        Pos2::new(fabric.left(), sy),
                        Pos2::new(fabric.left() - len, sy),
                    ],
                    stroke,
                );
                p.line_segment(
                    [
                        Pos2::new(fabric.right(), sy),
                        Pos2::new(fabric.right() + len, sy),
                    ],
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
        p.rect_filled(
            fabric.expand(3.0),
            Rounding::same(4.0),
            Color32::from_black_alpha(70),
        );
        p.rect_filled(fabric, Rounding::same(2.0), FABRIC);
        // fabric texture: faint weave
        p.rect_stroke(
            fabric,
            Rounding::same(2.0),
            Stroke::new(1.0_f32, FABRIC_EDGE),
        );

        if self.show_grid && z > 1.0 {
            let step = if z > 6.0 { 5.0 } else { 10.0 };
            let mut x = -hw / 2.0;
            while x <= hw / 2.0 + 0.01 {
                let sx = self.to_screen(rect, [x, 0.0]).x;
                let strong = (x.round() as i32) % 50 == 0;
                p.line_segment(
                    [Pos2::new(sx, fabric.top()), Pos2::new(sx, fabric.bottom())],
                    Stroke::new(
                        1.0_f32,
                        Color32::from_black_alpha(if strong { 26 } else { 12 }),
                    ),
                );
                x += step;
            }
            let mut y = -hh / 2.0;
            while y <= hh / 2.0 + 0.01 {
                let sy = self.to_screen(rect, [0.0, y]).y;
                let strong = (y.round() as i32) % 50 == 0;
                p.line_segment(
                    [Pos2::new(fabric.left(), sy), Pos2::new(fabric.right(), sy)],
                    Stroke::new(
                        1.0_f32,
                        Color32::from_black_alpha(if strong { 26 } else { 12 }),
                    ),
                );
                y += step;
            }
            // centre axes
            let c = self.to_screen(rect, [0.0, 0.0]);
            dashed(
                &p,
                Pos2::new(c.x, fabric.top()),
                Pos2::new(c.x, fabric.bottom()),
                Stroke::new(1.0_f32, Color32::from_black_alpha(34)),
                5.0,
            );
            dashed(
                &p,
                Pos2::new(fabric.left(), c.y),
                Pos2::new(fabric.right(), c.y),
                Stroke::new(1.0_f32, Color32::from_black_alpha(34)),
                5.0,
            );
        }

        // hoop band
        p.rect_stroke(
            fabric,
            Rounding::same(2.0),
            Stroke::new(2.0_f32, Color32::from_rgb(0x7e, 0x8a, 0x97)),
        );
        let band = Rect::from_min_max(
            Pos2::new(fabric.left() - 6.0, fabric.top() - 6.0),
            Pos2::new(fabric.right() + 6.0, fabric.bottom() + 6.0),
        );
        p.rect_stroke(
            band,
            Rounding::same(4.0),
            Stroke::new(1.0_f32, Color32::from_white_alpha(18)),
        );

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
                        pain.add(egui::Shape::line(
                            ch.clone(),
                            Stroke::new(
                                stitch_w + 3.0,
                                Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 46),
                            ),
                        ));
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
                        Stroke::new(0.6_f32, Color32::from_black_alpha(60)),
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
                pain.add(egui::Shape::line(
                    ch.clone(),
                    Stroke::new(stitch_w + 2.0, Color32::from_rgb(0xff, 0xd9, 0x66)),
                ));
            }
            if let Some(n) = needle {
                let s = self.to_screen(rect, n);
                pain.circle_stroke(
                    s,
                    7.0,
                    Stroke::new(1.4_f32, Color32::from_rgb(0xff, 0xd9, 0x66)),
                );
                pain.circle_filled(s, 2.2, Color32::from_rgb(0xff, 0xf3, 0xc4));
                let l = 11.0;
                pain.line_segment(
                    [s + Vec2::new(-l, 0.0), s + Vec2::new(-3.0, 0.0)],
                    Stroke::new(1.0_f32, Color32::from_white_alpha(120)),
                );
                pain.line_segment(
                    [s + Vec2::new(3.0, 0.0), s + Vec2::new(l, 0.0)],
                    Stroke::new(1.0_f32, Color32::from_white_alpha(120)),
                );
                pain.line_segment(
                    [s + Vec2::new(0.0, -l), s + Vec2::new(0.0, -3.0)],
                    Stroke::new(1.0_f32, Color32::from_white_alpha(120)),
                );
                pain.line_segment(
                    [s + Vec2::new(0.0, 3.0), s + Vec2::new(0.0, l)],
                    Stroke::new(1.0_f32, Color32::from_white_alpha(120)),
                );
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
                dashed(&pain, p0, p1, Stroke::new(1.2_f32, col), 4.0);
            }
            for c in [
                r.left_top(),
                r.right_top(),
                r.left_bottom(),
                r.right_bottom(),
            ] {
                pain.rect_filled(
                    Rect::from_center_size(c, Vec2::splat(5.0)),
                    Rounding::same(1.0),
                    col,
                );
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
            let galley = p.layout_no_wrap(t("canvas.outside"), FontId::proportional(10.5), warn_fg);
            let size = galley.size() + Vec2::new(18.0, 8.0);
            let warn = Rect::from_min_size(Pos2::new(fabric.left(), fabric.top() + 8.0), size);
            p.rect_filled(
                warn,
                Rounding::same(5.0),
                Color32::from_rgb(0x4a, 0x3a, 0x1e),
            );
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
}
