use super::App;
use crate::lang::t;
use crate::model::{self, Project};
use crate::ui::{fmt_time, ruler_steps, tick_style};
use eframe::egui;
use egui::{Pos2, Rect, Vec2};

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
        assert!(
            map.contains_key("summary.size_cm"),
            "{code}: summary.size_cm"
        );
        assert!(!map["help.example"].is_empty(), "{code}: help.example");
        assert!(!map["msg.example"].is_empty(), "{code}: msg.example");
    }
}
