//! Translations.
//!
//! Every label in the window comes from a flat JSON file in the `lang`
//! folder: one key per line, the text after it. Adding or correcting a
//! language means editing (or copying) one of those files — no code.
//!
//! The three shipped files are also compiled into the program, so it still
//! runs when the folder is missing. A file found on disk at run time wins
//! over the compiled-in copy, key by key.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

const EMBEDDED: &[(&str, &str)] = &[
    ("en", include_str!("../lang/en.json")),
    ("es", include_str!("../lang/es.json")),
    ("eu", include_str!("../lang/eu.json")),
];

/// The order the language picker lists them in.
const ORDER: [&str; 3] = ["en", "es", "eu"];

struct Lang {
    code: String,
    name: String,
    map: BTreeMap<String, String>,
    english: BTreeMap<String, String>,
    /// Set when a language file on disk exists but could not be read.
    error: Option<String>,
}

static CURRENT: OnceLock<RwLock<Lang>> = OnceLock::new();
static AVAILABLE: OnceLock<Vec<(String, String)>> = OnceLock::new();

fn parse(txt: &str) -> BTreeMap<String, String> {
    serde_json::from_str::<BTreeMap<String, String>>(txt).unwrap_or_default()
}

fn embedded(code: &str) -> &'static str {
    EMBEDDED
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, t)| *t)
        .unwrap_or(EMBEDDED[0].1)
}

fn manifest_lang() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lang")
}

/// The first `lang` folder that exists, so the files can sit next to the
/// program, next to the source, or in the current folder.
fn lang_dir() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(d) = std::env::var_os("STITCH_LANG_DIR") {
        candidates.push(PathBuf::from(d));
    }
    candidates.push(PathBuf::from("lang"));
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..4 {
            if let Some(d) = dir.clone() {
                candidates.push(d.join("lang"));
                dir = d.parent().map(|p| p.to_path_buf());
            }
        }
    }
    candidates.push(manifest_lang());
    candidates.into_iter().find(|p| p.is_dir())
}

fn load(code: &str) -> Lang {
    let mut map = parse(embedded(code));
    let mut error = None;
    if let Some(dir) = lang_dir() {
        let path = dir.join(format!("{code}.json"));
        if path.is_file() {
            match std::fs::read_to_string(&path) {
                Ok(txt) => match serde_json::from_str::<BTreeMap<String, String>>(&txt) {
                    Ok(overlay) => {
                        for (k, v) in overlay {
                            map.insert(k, v);
                        }
                    }
                    Err(e) => error = Some(format!("{}: {e}", path.display())),
                },
                Err(e) => error = Some(format!("{}: {e}", path.display())),
            }
        }
    }
    let name = map
        .get("_name")
        .cloned()
        .unwrap_or_else(|| code.to_string());
    Lang {
        code: code.to_string(),
        name,
        map,
        english: parse(embedded("en")),
        error,
    }
}

fn store() -> &'static RwLock<Lang> {
    CURRENT.get_or_init(|| RwLock::new(load("en")))
}

fn exists(code: &str) -> bool {
    EMBEDDED.iter().any(|(c, _)| *c == code)
        || lang_dir()
            .map(|d| d.join(format!("{code}.json")).is_file())
            .unwrap_or(false)
}

/// Language files present: the three shipped ones first, then any extra
/// `.json` the person dropped into the same folder.
fn discover() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for code in ORDER {
        let l = load(code);
        out.push((l.code, l.name));
    }
    if let Some(dir) = lang_dir() {
        let mut extra: Vec<(String, String)> = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().map(|x| x != "json").unwrap_or(true) {
                    continue;
                }
                let Some(code) = p.file_stem().map(|s| s.to_string_lossy().to_string()) else {
                    continue;
                };
                if out.iter().any(|(c, _)| *c == code) {
                    continue;
                }
                if let Ok(txt) = std::fs::read_to_string(&p) {
                    let m = parse(&txt);
                    if !m.is_empty() {
                        let name = m.get("_name").cloned().unwrap_or_else(|| code.clone());
                        extra.push((code, name));
                    }
                }
            }
        }
        extra.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
        out.extend(extra);
    }
    out
}

/// Choose the language to start in: the argument wins, then the saved
/// choice, then English.
pub fn init(requested: Option<&str>) {
    let code = requested
        .map(|s| s.to_string())
        .filter(|c| exists(c))
        .or_else(saved_language)
        .filter(|c| exists(c))
        .unwrap_or_else(|| "en".to_string());
    set(&code);
    let _ = AVAILABLE.set(discover());
}

pub fn set(code: &str) {
    let l = load(code);
    if let Ok(mut w) = store().write() {
        *w = l;
    }
}

pub fn code() -> String {
    store()
        .read()
        .map(|l| l.code.clone())
        .unwrap_or_else(|_| "en".into())
}

pub fn name() -> String {
    store()
        .read()
        .map(|l| l.name.clone())
        .unwrap_or_else(|_| "English".into())
}

/// A message when the language file for the current language is unreadable,
/// so a half-finished edit does not fail silently.
pub fn load_error() -> Option<String> {
    store().read().ok().and_then(|l| l.error.clone())
}

pub fn available() -> &'static [(String, String)] {
    AVAILABLE.get_or_init(discover)
}

/// One label. Unknown keys fall back to English, then to the key itself.
pub fn t(key: &str) -> String {
    let lock = store().read();
    match lock {
        Ok(l) => l
            .map
            .get(key)
            .or_else(|| l.english.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string()),
        Err(_) => key.to_string(),
    }
}

/// One label with `{placeholders}` filled in.
pub fn tf(key: &str, args: &[(&str, &str)]) -> String {
    let mut s = t(key);
    for (k, v) in args {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

pub fn thread_name(i: usize) -> String {
    t(&format!("thread.{i}"))
}

// ------------------------------------------------------------ saved choice

fn config_file() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("stitch-studio").join("settings.json"))
}

pub fn saved_language() -> Option<String> {
    let path = config_file()?;
    let txt = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&txt).ok()?;
    v.get("lang")?.as_str().map(|s| s.to_string())
}

pub fn save_language(code: &str) {
    let Some(path) = config_file() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let txt = format!("{{\n  \"lang\": \"{code}\"\n}}\n");
    let _ = std::fs::write(path, txt);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_the_english_keys() {
        let en = parse(embedded("en"));
        assert!(en.len() > 100);
        for (code, _) in EMBEDDED {
            let other = parse(embedded(code));
            for k in en.keys() {
                assert!(other.contains_key(k), "{code} is missing {k}");
            }
            for (k, v) in &other {
                let a = v.matches('{').count();
                let b = en[k].matches('{').count();
                assert_eq!(a, b, "placeholder count differs in {code}: {k}");
            }
        }
    }

    #[test]
    fn languages_load_and_switch() {
        set("en");
        assert_eq!(t("menu.file"), "File");
        assert_eq!(tf("objects.header", &[("n", "3")]), "Objects · 3");
        assert_eq!(t("no.such.key"), "no.such.key");
        set("es");
        assert_eq!(t("menu.file"), "Archivo");
        assert_eq!(thread_name(5), "Coral");
        set("eu");
        assert_eq!(t("menu.file"), "Fitxategia");
        assert_eq!(t("tab.settings"), "Ezarpenak");
        set("en");
        assert_eq!(t("menu.file"), "File");
    }
}
