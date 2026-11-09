//! Palette, global styling and small hand-drawn UI helpers.

use egui::{Color32, FontId, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2};

pub const BG: Color32 = Color32::from_rgb(0x13, 0x16, 0x1b);
pub const PANEL: Color32 = Color32::from_rgb(0x1a, 0x1e, 0x25);
pub const PANEL2: Color32 = Color32::from_rgb(0x21, 0x26, 0x2e);
pub const FIELD: Color32 = Color32::from_rgb(0x16, 0x19, 0x1f);
pub const LINE: Color32 = Color32::from_rgb(0x2e, 0x34, 0x3e);
pub const LINE_SOFT: Color32 = Color32::from_rgb(0x26, 0x2b, 0x34);
pub const TEXT: Color32 = Color32::from_rgb(0xdf, 0xe4, 0xea);
pub const MUTED: Color32 = Color32::from_rgb(0x8a, 0x94, 0xa2);
pub const DIM: Color32 = Color32::from_rgb(0x5f, 0x69, 0x77);
pub const ACCENT: Color32 = Color32::from_rgb(0x4d, 0xc3, 0xa6);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(0x2c, 0x6d, 0x5f);
pub const BLUE: Color32 = Color32::from_rgb(0x63, 0xa8, 0xe0);
pub const OK: Color32 = Color32::from_rgb(0x6f, 0xcf, 0x97);
pub const WARN: Color32 = Color32::from_rgb(0xe2, 0xb0, 0x5c);
pub const DANGER: Color32 = Color32::from_rgb(0xe0, 0x76, 0x6d);
pub const FABRIC: Color32 = Color32::from_rgb(0xe9, 0xe4, 0xd8);
pub const FABRIC_EDGE: Color32 = Color32::from_rgb(0xc9, 0xc2, 0xb0);

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::dark();
    v.panel_fill = PANEL;
    v.window_fill = PANEL;
    v.extreme_bg_color = FIELD;
    v.faint_bg_color = PANEL2;
    v.override_text_color = Some(TEXT);
    v.selection.bg_fill = ACCENT_DIM;
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.hyperlink_color = BLUE;
    v.window_stroke = Stroke::new(1.0, LINE);
    v.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 8.0),
        blur: 26.0,
        spread: 0.0,
        color: Color32::from_black_alpha(140),
    };
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.rounding = Rounding::same(6.0);
        w.bg_stroke = Stroke::new(1.0, LINE);
    }
    v.widgets.inactive.bg_fill = PANEL2;
    v.widgets.inactive.weak_bg_fill = PANEL2;
    v.widgets.hovered.bg_fill = Color32::from_rgb(0x2b, 0x32, 0x3c);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x2b, 0x32, 0x3c);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x45, 0x4e, 0x5b));
    v.widgets.active.bg_fill = Color32::from_rgb(0x33, 0x3b, 0x47);
    v.widgets.active.weak_bg_fill = Color32::from_rgb(0x33, 0x3b, 0x47);
    v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_DIM);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE_SOFT);
    style.visuals = v;

    style.spacing.item_spacing = Vec2::new(7.0, 6.0);
    style.spacing.button_padding = Vec2::new(9.0, 5.0);
    style.spacing.slider_width = 130.0;
    style.spacing.interact_size.y = 22.0;
    style.text_styles = [
        (egui::TextStyle::Heading, FontId::proportional(18.0)),
        (egui::TextStyle::Body, FontId::proportional(13.5)),
        (egui::TextStyle::Button, FontId::proportional(13.0)),
        (egui::TextStyle::Small, FontId::proportional(11.5)),
        (egui::TextStyle::Monospace, FontId::monospace(12.0)),
    ]
    .into();
    ctx.set_style(style);
}

pub fn card(ui: &mut Ui, fill: Color32, add: impl FnOnce(&mut Ui)) {
    egui::Frame::none()
        .fill(fill)
        .rounding(Rounding::same(8.0))
        .stroke(Stroke::new(1.0, LINE_SOFT))
        .inner_margin(egui::Margin::symmetric(10.0, 9.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

pub fn section(ui: &mut Ui, title: &str) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(title.to_uppercase()).size(10.5).color(MUTED));
    });
    ui.add_space(1.0);
}

#[allow(dead_code)]
pub fn chip(ui: &mut Ui, text: &str, dot: Color32, fg: Color32) {
    let font = FontId::proportional(11.0);
    let galley = ui.painter().layout_no_wrap(text.to_string(), font.clone(), fg);
    let pad = Vec2::new(9.0, 4.0);
    let size = Vec2::new(galley.size().x + pad.x * 2.0 + 12.0, galley.size().y + pad.y * 2.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, Rounding::same(9.0), Color32::from_rgb(0x20, 0x26, 0x2d));
    p.rect_stroke(rect, Rounding::same(9.0), Stroke::new(1.0, LINE_SOFT));
    let cy = rect.center().y;
    p.circle_filled(egui::pos2(rect.left() + pad.x + 2.0, cy), 2.6, dot);
    p.text(
        egui::pos2(rect.left() + pad.x + 10.0, cy),
        egui::Align2::LEFT_CENTER,
        text,
        font,
        fg,
    );
}

/// Small label/value row used in the summary cards.
pub fn kv(ui: &mut Ui, k: &str, v: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(k).size(11.5).color(MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(v).size(12.0).color(TEXT));
        });
    });
}

pub fn hint(ui: &mut Ui, text: &str) {
    ui.label(egui::RichText::new(text).size(11.0).color(DIM));
}

// ------------------------------------------------------------------ icon art

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Select,
    Rect,
    Ellipse,
    Path,
    Satin,
    Pan,
    Fit,
    Grid,
    Ruler,
    Needle,
    Key,
    Shield,
    Save,
    Open,
    Plus,
    Undo,
    Redo,
    Trash,
}

pub fn icon_button(ui: &mut Ui, icon: Icon, tip: &str, selected: bool, enabled: bool) -> Response {
    let size = Vec2::new(34.0, 30.0);
    let (rect, resp) = ui.allocate_exact_size(size, if enabled { Sense::click() } else { Sense::hover() });
    let p = ui.painter();
    let hovered = resp.hovered() && enabled;
    let bg = if selected {
        ACCENT_DIM
    } else if hovered {
        Color32::from_rgb(0x2a, 0x31, 0x3a)
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        p.rect_filled(rect.shrink(1.0), Rounding::same(6.0), bg);
    }
    let fg = if !enabled {
        DIM
    } else if selected {
        Color32::from_rgb(0xea, 0xfb, 0xf5)
    } else {
        Color32::from_rgb(0xc2, 0xca, 0xd4)
    };
    draw_icon(p, rect.shrink(8.0), icon, fg);
    resp.on_hover_text(tip)
}

fn draw_icon(p: &egui::Painter, r: Rect, icon: Icon, c: Color32) {
    let s = Stroke::new(1.5, c);
    let ctr = r.center();
    let w = r.width();
    let h = r.height();
    match icon {
        Icon::Select => {
            let pts = vec![
                egui::pos2(r.left() + w * 0.1, r.top()),
                egui::pos2(r.left() + w * 0.1, r.bottom()),
                egui::pos2(r.left() + w * 0.45, r.top() + h * 0.72),
                egui::pos2(r.left() + w * 0.65, r.bottom()),
                egui::pos2(r.left() + w * 0.78, r.top() + h * 0.72),
                egui::pos2(r.left() + w * 0.45, r.top() + h * 0.58),
            ];
            p.add(egui::Shape::closed_line(pts, s));
        }
        Icon::Rect => {
            p.rect_stroke(
                Rect::from_center_size(ctr, egui::vec2(w * 0.82, h * 0.66)),
                Rounding::same(1.0),
                s,
            );
        }
        Icon::Ellipse => {
            p.circle_stroke(ctr, w * 0.38, s);
        }
        Icon::Path => {
            let pts = vec![
                egui::pos2(r.left(), r.bottom() - h * 0.1),
                egui::pos2(r.left() + w * 0.32, r.top() + h * 0.15),
                egui::pos2(r.left() + w * 0.6, r.top() + h * 0.72),
                egui::pos2(r.right(), r.top() + h * 0.2),
            ];
            p.add(egui::Shape::line(pts, s));
        }
        Icon::Satin => {
            let mut x = r.left();
            let mut up = true;
            while x < r.right() - 1.0 {
                let y0 = if up { r.top() + h * 0.18 } else { r.bottom() - h * 0.18 };
                let y1 = if up { r.bottom() - h * 0.18 } else { r.top() + h * 0.18 };
                p.line_segment([egui::pos2(x, y0), egui::pos2(x + 3.0, y1)], s);
                x += 4.0;
                up = !up;
            }
            p.line_segment(
                [egui::pos2(r.left(), r.top() + h * 0.18), egui::pos2(r.right(), r.top() + h * 0.18)],
                Stroke::new(1.0, c.gamma_multiply(0.45)),
            );
            p.line_segment(
                [egui::pos2(r.left(), r.bottom() - h * 0.18), egui::pos2(r.right(), r.bottom() - h * 0.18)],
                Stroke::new(1.0, c.gamma_multiply(0.45)),
            );
        }
        Icon::Pan => {
            p.line_segment([egui::pos2(ctr.x, r.top()), egui::pos2(ctr.x, r.bottom())], s);
            p.line_segment([egui::pos2(r.left(), ctr.y), egui::pos2(r.right(), ctr.y)], s);
            for (a, b) in [
                (egui::pos2(ctr.x, r.top()), egui::pos2(ctr.x - 4.0, r.top() + 5.0)),
                (egui::pos2(ctr.x, r.top()), egui::pos2(ctr.x + 4.0, r.top() + 5.0)),
                (egui::pos2(ctr.x, r.bottom()), egui::pos2(ctr.x - 4.0, r.bottom() - 5.0)),
                (egui::pos2(ctr.x, r.bottom()), egui::pos2(ctr.x + 4.0, r.bottom() - 5.0)),
                (egui::pos2(r.left(), ctr.y), egui::pos2(r.left() + 5.0, ctr.y - 4.0)),
                (egui::pos2(r.left(), ctr.y), egui::pos2(r.left() + 5.0, ctr.y + 4.0)),
                (egui::pos2(r.right(), ctr.y), egui::pos2(r.right() - 5.0, ctr.y - 4.0)),
                (egui::pos2(r.right(), ctr.y), egui::pos2(r.right() - 5.0, ctr.y + 4.0)),
            ] {
                p.line_segment([a, b], s);
            }
        }
        Icon::Fit => {
            let rr = Rect::from_center_size(ctr, egui::vec2(w * 0.8, h * 0.72));
            let k = 5.0;
            for (corner, dx, dy) in [
                (rr.left_top(), 1.0, 1.0),
                (rr.right_top(), -1.0, 1.0),
                (rr.left_bottom(), 1.0, -1.0),
                (rr.right_bottom(), -1.0, -1.0),
            ] {
                p.line_segment([corner, egui::pos2(corner.x + k * dx, corner.y)], s);
                p.line_segment([corner, egui::pos2(corner.x, corner.y + k * dy)], s);
            }
            p.rect_stroke(Rect::from_center_size(ctr, Vec2::new(w * 0.26, h * 0.2)), Rounding::same(1.0), Stroke::new(1.0, c.gamma_multiply(0.6)));
        }
        Icon::Grid => {
            for i in 1..4 {
                let t = i as f32 / 4.0;
                p.line_segment(
                    [egui::pos2(r.left() + w * t, r.top()), egui::pos2(r.left() + w * t, r.bottom())],
                    Stroke::new(1.0, c.gamma_multiply(0.8)),
                );
                p.line_segment(
                    [egui::pos2(r.left(), r.top() + h * t), egui::pos2(r.right(), r.top() + h * t)],
                    Stroke::new(1.0, c.gamma_multiply(0.8)),
                );
            }
        }
        Icon::Ruler => {
            let body = Rect::from_min_max(
                egui::pos2(r.left(), r.top() + h * 0.28),
                egui::pos2(r.right(), r.top() + h * 0.7),
            );
            p.rect_stroke(body, Rounding::same(1.0), s);
            for i in 0..=6 {
                let x = body.left() + body.width() * i as f32 / 6.0;
                let len = if i % 3 == 0 { body.height() * 0.62 } else { body.height() * 0.36 };
                p.line_segment(
                    [egui::pos2(x, body.top()), egui::pos2(x, body.top() + len)],
                    Stroke::new(1.0, c),
                );
            }
        }
        Icon::Needle => {
            p.line_segment([egui::pos2(r.left() + w * 0.1, r.bottom()), egui::pos2(r.right() - 2.0, r.top() + 2.0)], s);
            p.circle_filled(egui::pos2(r.right() - 2.0, r.top() + 2.0), 2.0, c);
            p.line_segment([egui::pos2(r.left() + w * 0.6, r.bottom() - 3.0), egui::pos2(r.left() + w * 0.9, r.bottom() - h * 0.55)], Stroke::new(1.0, c.gamma_multiply(0.6)));
        }
        Icon::Key => {
            p.circle_stroke(egui::pos2(r.left() + w * 0.22, ctr.y), w * 0.18, s);
            p.line_segment([egui::pos2(r.left() + w * 0.4, ctr.y), egui::pos2(r.right(), ctr.y)], s);
            p.line_segment([egui::pos2(r.right() - 4.0, ctr.y), egui::pos2(r.right() - 4.0, ctr.y + 5.0)], s);
            p.line_segment([egui::pos2(r.right() - 9.0, ctr.y), egui::pos2(r.right() - 9.0, ctr.y + 4.0)], s);
        }
        Icon::Shield => {
            let pts = vec![
                egui::pos2(ctr.x, r.top()),
                egui::pos2(r.right(), r.top() + h * 0.22),
                egui::pos2(r.right() - 2.0, r.bottom() - h * 0.22),
                egui::pos2(ctr.x, r.bottom()),
                egui::pos2(r.left() + 2.0, r.bottom() - h * 0.22),
                egui::pos2(r.left(), r.top() + h * 0.22),
            ];
            p.add(egui::Shape::closed_line(pts, s));
        }
        Icon::Save => {
            p.rect_stroke(Rect::from_center_size(ctr, Vec2::new(w * 0.78, h * 0.78)), Rounding::same(2.0), s);
            p.rect_filled(
                Rect::from_min_size(
                    egui::pos2(ctr.x - w * 0.22, r.top() + h * 0.11),
                    Vec2::new(w * 0.44, h * 0.26),
                ),
                Rounding::same(1.0),
                c.gamma_multiply(0.85),
            );
            p.rect_stroke(
                Rect::from_min_size(
                    egui::pos2(ctr.x - w * 0.26, ctr.y + h * 0.08),
                    Vec2::new(w * 0.52, h * 0.3),
                ),
                Rounding::same(1.0),
                Stroke::new(1.2, c),
            );
        }
        Icon::Open => {
            p.rect_stroke(
                Rect::from_min_size(
                    egui::pos2(r.left(), r.top() + h * 0.25),
                    Vec2::new(w * 0.72, h * 0.58),
                ),
                Rounding::same(1.5),
                s,
            );
            p.add(egui::Shape::line(
                vec![
                    egui::pos2(r.left() + w * 0.1, r.top() + h * 0.25),
                    egui::pos2(r.left() + w * 0.1, r.top() + h * 0.12),
                    egui::pos2(r.left() + w * 0.42, r.top() + h * 0.12),
                    egui::pos2(r.left() + w * 0.5, r.top() + h * 0.25),
                ],
                s,
            ));
        }
        Icon::Plus => {
            p.line_segment([egui::pos2(ctr.x, r.top()), egui::pos2(ctr.x, r.bottom())], s);
            p.line_segment([egui::pos2(r.left(), ctr.y), egui::pos2(r.right(), ctr.y)], s);
        }
        Icon::Undo => {
            let pts = vec![
                egui::pos2(r.left(), ctr.y),
                egui::pos2(ctr.x, ctr.y - h * 0.32),
                egui::pos2(r.right() - 1.0, ctr.y),
                egui::pos2(r.right() - 1.0, r.bottom() - h * 0.1),
            ];
            p.add(egui::Shape::line(pts, s));
            p.line_segment([egui::pos2(r.left(), ctr.y), egui::pos2(r.left() + 5.0, ctr.y - 4.0)], s);
            p.line_segment([egui::pos2(r.left(), ctr.y), egui::pos2(r.left() + 5.0, ctr.y + 4.0)], s);
        }
        Icon::Redo => {
            let pts = vec![
                egui::pos2(r.right(), ctr.y),
                egui::pos2(ctr.x, ctr.y - h * 0.32),
                egui::pos2(r.left() + 1.0, ctr.y),
                egui::pos2(r.left() + 1.0, r.bottom() - h * 0.1),
            ];
            p.add(egui::Shape::line(pts, s));
            p.line_segment([egui::pos2(r.right(), ctr.y), egui::pos2(r.right() - 5.0, ctr.y - 4.0)], s);
            p.line_segment([egui::pos2(r.right(), ctr.y), egui::pos2(r.right() - 5.0, ctr.y + 4.0)], s);
        }
        Icon::Trash => {
            p.rect_stroke(
                Rect::from_min_size(
                    egui::pos2(r.left() + w * 0.15, r.top() + h * 0.22),
                    Vec2::new(w * 0.7, h * 0.72),
                ),
                Rounding::same(1.5),
                s,
            );
            p.line_segment([egui::pos2(r.left(), r.top() + h * 0.14), egui::pos2(r.right(), r.top() + h * 0.14)], s);
            p.line_segment([egui::pos2(ctr.x - w * 0.16, r.top() + h * 0.14), egui::pos2(ctr.x - w * 0.16, r.top())], s);
        }
    }
}

/// Segmented control; returns true when the value changed.
pub fn segmented(ui: &mut Ui, value: &mut usize, labels: &[&str]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        for (i, l) in labels.iter().enumerate() {
            let sel = *value == i;
            let txt = egui::RichText::new(*l).size(11.5).color(if sel { Color32::from_rgb(0x0f, 0x2a, 0x24) } else { MUTED });
            let btn = egui::Button::new(txt)
                .fill(if sel { ACCENT } else { PANEL2 })
                .stroke(Stroke::new(1.0, if sel { ACCENT } else { LINE }))
                .rounding(Rounding::same(5.0))
                .min_size(Vec2::new(0.0, 22.0));
            if ui.add(btn).clicked() && !sel {
                *value = i;
                changed = true;
            }
        }
    });
    changed
}
