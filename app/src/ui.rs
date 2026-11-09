//! Shared UI helpers and small formatting/geometry utilities.

use crate::lang::t;
use crate::model::{Obj, ShapeKind};
use crate::theme::{ACCENT, DIM, FIELD, LINE};
use eframe::egui;
use egui::{Color32, Pos2, RichText, Rounding, Stroke, Vec2};

pub(crate) fn num_field(
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
        .stroke(Stroke::new(1.0_f32, LINE))
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

pub(crate) fn mm_snap(p: [f32; 2], on: bool) -> [f32; 2] {
    if on {
        [p[0].round(), p[1].round()]
    } else {
        p
    }
}

/// How dense the hoop rulers are: (tick spacing in mm, number spacing in mm).
/// One-millimetre ticks are used as soon as they are at least 3 px apart;
/// below that they would read as a solid line, so the scale coarsens.
pub(crate) fn ruler_steps(zoom: f32) -> (i32, i32) {
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
pub(crate) fn tick_style(v: i32) -> (f32, Stroke) {
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
    (len, Stroke::new(1.0_f32, col))
}

pub(crate) fn shape_word(o: &Obj) -> String {
    match o.shape {
        ShapeKind::Rect => t("shape.rect"),
        ShapeKind::Ellipse => t("shape.ellipse"),
        ShapeKind::Path => t("shape.path"),
        ShapeKind::Stitches => t("shape.stitches"),
    }
}

pub(crate) fn dashed(p: &egui::Painter, a: Pos2, b: Pos2, stroke: Stroke, dash: f32) {
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

pub(crate) fn fmt_num(n: usize) -> String {
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

pub(crate) fn fmt_time(min: f32) -> String {
    let secs = (min * 60.0).round().max(0.0) as u32;
    if secs >= 3600 {
        format!("{}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

pub(crate) fn slug(s: &str) -> String {
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

pub(crate) fn dir_hint(ui: &mut egui::Ui) {
    ui.label(RichText::new(t("tip.satin")).size(11.0).color(DIM));
}
