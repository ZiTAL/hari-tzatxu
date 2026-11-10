//! hari-tzatxu - an embroidery workspace that needs no USB key.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod io;
mod lang;
mod model;
mod theme;
mod ui;

use app::App;
use eframe::egui;
use lang::t;
use std::path::PathBuf;

pub(crate) const APP_TITLE: &str = "hari-tzatxu";

fn main() -> eframe::Result<()> {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!("hari-tzatxu — an embroidery workspace that needs no USB key");
        println!();
        println!("Usage: hari-tzatxu [--lang <code>] [--screenshot <file.png>]");
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
