//! Design model: hoops, stitch objects and the stitch generator.
//! All geometry is stored in millimetres, origin at the hoop centre.

use serde::{Deserialize, Serialize};

pub type P = [f32; 2];

/// The hoops the original software offers, with the names and sizes it
/// stores in its own hoop files (`RES/HoopsCommon(Digitizer).DSC` and
/// `RES/HoopsProMBOnly(Digitizer).DSC`, lines `HOOP RECTANGLE (w,h) Name`).
/// The seven `D..` hoops are round on the machine, but the original's own
/// hoop files describe them as rectangles of that diameter, so they are
/// offered as squares here too (see MECHANICS.md).
pub const HOOPS: [(&str, f32, f32); 39] = [
    ("Hoop C (50 x 50)", 50.0, 50.0),
    ("J5 No.3:MB Hoop (47 x 47)", 47.0, 47.0),
    ("J6 No.2:MB Hoop (66 x 66)", 66.0, 66.0),
    ("J3 No.13:MB Hoop (64 x 28)", 64.0, 28.0),
    ("J4 No.8:MB Hoop (42 x 67)", 42.0, 67.0),
    ("J2 No.6:MB Hoop (24 x 54)", 24.0, 54.0),
    ("S2:MB Hoop (28 x 50)", 28.0, 50.0),
    ("S1:MB Hoop (37 x 59)", 37.0, 59.0),
    ("M4:MB Hoop (100 x 60)", 100.0, 60.0),
    ("J7 No.1:MB Hoop (110 x 95)", 110.0, 95.0),
    ("Hoop No. 1 & 3 (126 x 90)", 126.0, 90.0),
    ("Hoop A (110 x 110) Ver.2.21", 110.0, 110.0),
    ("H1:MB Hoop (100 x 90)", 100.0, 90.0),
    ("Hoop HH (100 x 90)", 100.0, 90.0),
    ("Hoop A(F) (126 x 110) Ver.3.0", 126.0, 110.0),
    ("Hoop ST (126 x 110)", 126.0, 110.0),
    ("M2:MB Hoop (126 x 110)", 126.0, 110.0),
    ("J8 No.7:MB Hoop (120 x 121)", 120.0, 121.0),
    ("200 Hoop (50 x 50)", 50.0, 50.0),
    ("200 Hoop (140 x 140)", 140.0, 140.0),
    ("Hoop FA (50 x 50)", 50.0, 50.0),
    ("M3:MB Hoop (50 x 50)", 50.0, 50.0),
    ("Hoop B (140 x 200)", 140.0, 200.0),
    ("Hoop RE (140 x 200)", 140.0, 200.0),
    ("Hoop AQ (120 x 180)", 120.0, 180.0),
    ("T6:MB Hoop (78 x 168)", 78.0, 168.0),
    ("Hoop ASQ (180 x 180)", 180.0, 180.0),
    ("Hoop SQ (200 x 200)", 200.0, 200.0),
    ("Hoop D (220 x 190) (Giga Hoop)", 220.0, 190.0),
    ("Hoop MA (200 x 280)", 200.0, 280.0),
    ("M1:MB Hoop (240 x 200)", 240.0, 200.0),
    ("J6 No.9:MB Hoop (66 x 66)", 66.0, 66.0),
    // The seven D-sized hoops are stored in HoopsProMBOnly(Digitizer).DSC as
    // `HOOP RECTANGLE (d,d)`, i.e. as squares of their diameter.
    ("T1:MB Hoop (D34)", 34.0, 34.0),
    ("J1 No.11:MB Hoop (D30)", 30.0, 30.0),
    ("J1 No.12:MB Hoop (D30)", 30.0, 30.0),
    ("T2:MB Hoop (D54)", 54.0, 54.0),
    ("T3:MB Hoop (D84)", 84.0, 84.0),
    ("T4:MB Hoop (D112)", 112.0, 112.0),
    ("T5:MB Hoop (D142)", 142.0, 142.0),
];

/// The hoop a new design starts in: the 140 × 200 mm Hoop B, the standard
/// rectangular hoop on the machines this software shipped with.
pub const DEFAULT_HOOP: (f32, f32) = (140.0, 200.0);

/// The original adds one caller-supplied allowance to **both** sides of the
/// design before comparing it with a hoop: `GetBestFittingHoop` is declared
/// `(uint machine, double w, double h, double allowance)` at MOD1.DLL
/// 0x1009ba60 and tests `w + allowance < hoop_w` and `h + allowance < hoop_h`
/// (strictly smaller, both sides). The call site that chooses the allowance
/// sits inside the protected executable and cannot be read, so the rebuild
/// uses no extra allowance.
pub const HOOP_ALLOWANCE_MM: f32 = 0.0;

/// Smallest and largest hoop side the program accepts, in millimetres.
pub const HOOP_MIN_MM: f32 = 20.0;
pub const HOOP_MAX_MM: f32 = 600.0;

/// Machine speed used to turn a stitch count into sewing time, in stitches
/// per second. 12.5 stitches per second is 750 stitches per minute, a common
/// single-needle rate. The person can change it and the time follows.
pub const DEFAULT_RATE_SPS: f32 = 12.5;
pub const RATE_MIN_SPS: f32 = 0.5;
pub const RATE_MAX_SPS: f32 = 60.0;

pub const THREADS: [(&str, [u8; 3]); 30] = [
    ("Ivory", [245, 240, 226]),
    ("Cream", [233, 221, 196]),
    ("Lemon", [246, 226, 127]),
    ("Gold", [228, 176, 66]),
    ("Amber", [214, 138, 49]),
    ("Coral", [232, 122, 100]),
    ("Scarlet", [198, 54, 55]),
    ("Wine", [136, 40, 58]),
    ("Rose", [224, 132, 152]),
    ("Blush", [238, 186, 190]),
    ("Orchid", [178, 114, 176]),
    ("Plum", [110, 62, 120]),
    ("Lavender", [178, 168, 214]),
    ("Periwinkle", [128, 142, 208]),
    ("Indigo", [58, 70, 128]),
    ("Sky", [124, 182, 222]),
    ("Teal", [58, 152, 156]),
    ("Pine", [44, 102, 82]),
    ("Moss", [104, 130, 68]),
    ("Olive", [132, 126, 62]),
    ("Sand", [214, 190, 152]),
    ("Tan", [186, 150, 110]),
    ("Cocoa", [110, 78, 58]),
    ("Chocolate", [74, 52, 40]),
    ("Silver", [190, 196, 202]),
    ("Slate", [104, 116, 128]),
    ("Graphite", [62, 68, 78]),
    ("Black", [28, 30, 34]),
    ("White", [252, 252, 250]),
    ("Rosewood", [150, 84, 72]),
];

pub fn thread_color(idx: usize) -> egui::Color32 {
    let c = THREADS[idx.min(THREADS.len() - 1)].1;
    egui::Color32::from_rgb(c[0], c[1], c[2])
}

/// A millimetre figure as a short label: whole numbers without a decimal.
pub fn fmt_mm(v: f32) -> String {
    if (v - v.round()).abs() < 0.05 {
        format!("{:.0}", v)
    } else {
        format!("{:.1}", v)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum StitchKind {
    Run,
    Satin,
    Fill,
}

impl StitchKind {
    /// Key stem used by the language files (`stitch.run`, …).
    pub fn key(self) -> &'static str {
        match self {
            StitchKind::Run => "run",
            StitchKind::Satin => "satin",
            StitchKind::Fill => "fill",
        }
    }

    /// The stitch type's name in the current language.
    pub fn label(self) -> String {
        crate::lang::t(&format!("stitch.{}", self.key()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ShapeKind {
    Rect,
    Ellipse,
    Path,
    /// A fixed sequence of stitch points, used by imported stitch files.
    Stitches,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Obj {
    pub id: u32,
    pub kind: StitchKind,
    pub shape: ShapeKind,
    /// Centre of the object in hoop millimetres.
    pub cx: f32,
    pub cy: f32,
    /// Local size in millimetres.
    pub w: f32,
    pub h: f32,
    /// Rotation around the centre, radians.
    pub rot: f32,
    /// Freehand points in hoop millimetres (only for ShapeKind::Path).
    pub path: Vec<P>,
    pub color: usize,
    pub visible: bool,
    /// Run stitch spacing, mm.
    pub stitch_len: f32,
    /// Fill / satin spacing, mm.
    pub density: f32,
    /// Satin column width, mm.
    pub satin_w: f32,
}

impl Obj {
    pub fn new_rect(id: u32, kind: StitchKind, cx: f32, cy: f32, w: f32, h: f32) -> Self {
        Self {
            id,
            kind,
            shape: ShapeKind::Rect,
            cx,
            cy,
            w: w.max(1.0),
            h: h.max(1.0),
            rot: 0.0,
            path: Vec::new(),
            color: 6,
            visible: true,
            stitch_len: 2.5,
            density: if kind == StitchKind::Satin { 0.4 } else { 0.45 },
            satin_w: 5.0,
        }
    }

    pub fn new_ellipse(id: u32, kind: StitchKind, cx: f32, cy: f32, w: f32, h: f32) -> Self {
        let mut o = Self::new_rect(id, kind, cx, cy, w, h);
        o.shape = ShapeKind::Ellipse;
        o
    }

    pub fn new_path(id: u32, kind: StitchKind, path: Vec<P>) -> Self {
        let (c, w, h) = path_bounds(&path);
        Self {
            id,
            kind,
            shape: ShapeKind::Path,
            cx: c[0],
            cy: c[1],
            w,
            h,
            rot: 0.0,
            color: 6,
            visible: true,
            stitch_len: 2.5,
            density: if kind == StitchKind::Satin { 0.4 } else { 0.45 },
            satin_w: 6.0,
            path,
        }
    }

    /// An object made of stitch points exactly as they were read from a file.
    /// The first point is the travel-in; every later point is a stitch.
    pub fn new_stitches(id: u32, color: usize, path: Vec<P>) -> Self {
        let (c, w, h) = path_bounds(&path);
        Self {
            id,
            kind: StitchKind::Run,
            shape: ShapeKind::Stitches,
            cx: c[0],
            cy: c[1],
            w: w.max(0.1),
            h: h.max(0.1),
            rot: 0.0,
            color,
            visible: true,
            stitch_len: 2.5,
            density: 0.45,
            satin_w: 5.0,
            path,
        }
    }

    /// Local (unrotated) corner points.
    pub fn local_outline(&self) -> Vec<P> {
        match self.shape {
            ShapeKind::Rect => vec![
                [-self.w / 2.0, -self.h / 2.0],
                [self.w / 2.0, -self.h / 2.0],
                [self.w / 2.0, self.h / 2.0],
                [-self.w / 2.0, self.h / 2.0],
            ],
            ShapeKind::Ellipse => {
                let n = 96;
                (0..n)
                    .map(|i| {
                        let a = i as f32 / n as f32 * std::f32::consts::TAU;
                        [a.cos() * self.w / 2.0, a.sin() * self.h / 2.0]
                    })
                    .collect()
            }
            ShapeKind::Path | ShapeKind::Stitches => {
                let mut v = Vec::new();
                for p in &self.path {
                    let lp = self.to_local(*p);
                    v.push(lp);
                }
                v
            }
        }
    }

    pub fn to_local(&self, p: P) -> P {
        let dx = p[0] - self.cx;
        let dy = p[1] - self.cy;
        let (s, c) = (-self.rot).sin_cos();
        [dx * c - dy * s, dx * s + dy * c]
    }

    pub fn to_world(&self, p: P) -> P {
        let (s, c) = self.rot.sin_cos();
        [self.cx + p[0] * c - p[1] * s, self.cy + p[0] * s + p[1] * c]
    }

    /// Axis aligned bounds in hoop millimetres.
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        if matches!(self.shape, ShapeKind::Path | ShapeKind::Stitches) && !self.path.is_empty() {
            return (
                self.cx - self.w / 2.0,
                self.cy - self.h / 2.0,
                self.cx + self.w / 2.0,
                self.cy + self.h / 2.0,
            );
        }
        // Rotated ellipse / rect: use the rotated extents of the local box.
        let c = self.rot.cos().abs();
        let s = self.rot.sin().abs();
        let hw = (self.w / 2.0) * c + (self.h / 2.0) * s;
        let hh = (self.w / 2.0) * s + (self.h / 2.0) * c;
        (self.cx - hw, self.cy - hh, self.cx + hw, self.cy + hh)
    }

    pub fn contains(&self, p: P) -> bool {
        let l = self.to_local(p);
        if matches!(self.shape, ShapeKind::Path | ShapeKind::Stitches) {
            let mut best = f32::MAX;
            let n = self.path.len();
            if n < 2 {
                return false;
            }
            for i in 0..n - 1 {
                let a = self.to_local(self.path[i]);
                let b = self.to_local(self.path[i + 1]);
                best = best.min(dist_to_seg(l, a, b));
            }
            best < 3.5
        } else {
            let dx = l[0] / (self.w / 2.0).max(0.01);
            let dy = l[1] / (self.h / 2.0).max(0.01);
            dx * dx + dy * dy <= 1.0
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Project {
    pub version: u32,
    pub name: String,
    /// Index of the last preset hoop the design was given. Kept so older
    /// design files still open; the real size lives in `hoop_w` / `hoop_h`.
    pub hoop: usize,
    /// Hoop width in millimetres. Zero means "use the preset".
    #[serde(default)]
    pub hoop_w: f32,
    /// Hoop height in millimetres. Zero means "use the preset".
    #[serde(default)]
    pub hoop_h: f32,
    /// Sewing speed in stitches per second; 0 means the default.
    #[serde(default)]
    pub rate: f32,
    pub objects: Vec<Obj>,
    pub next_id: u32,
}

impl Default for Project {
    fn default() -> Self {
        let hoop = default_hoop_preset();
        let (w, h) = (HOOPS[hoop].1, HOOPS[hoop].2);
        Self {
            version: 2,
            name: "Untitled design".to_string(),
            hoop,
            hoop_w: w,
            hoop_h: h,
            rate: DEFAULT_RATE_SPS,
            objects: Vec::new(),
            next_id: 1,
        }
    }
}

/// Index of the hoop a new design starts in.
pub fn default_hoop_preset() -> usize {
    HOOPS
        .iter()
        .position(|h| h.1 == DEFAULT_HOOP.0 && h.2 == DEFAULT_HOOP.1)
        .unwrap_or(0)
}

/// The smallest preset hoop a design of `w` × `h` millimetres fits in, using
/// the rule read from the original (`HoopLibrary::GetBestFittingHoop`,
/// MOD1.DLL 0x1009ba60): the design plus the allowance must be **strictly
/// smaller** than both sides, and of the hoops that pass, the smallest area
/// wins. The original keeps the earlier hoop when two areas are equal
/// (`best_area <= new_area`), and "earlier" is the library's own order — the
/// 39 display strings at RESOURCE.DLL 0x10007120 — so ties are broken with
/// `LIBRARY_RANK` rather than by this array's order.
pub fn best_preset_for(w: f32, h: f32) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (i, hoop) in HOOPS.iter().enumerate() {
        if w + HOOP_ALLOWANCE_MM < hoop.1 && h + HOOP_ALLOWANCE_MM < hoop.2 {
            let area = hoop.1 * hoop.2;
            let better = match best {
                None => true,
                Some((bi, ba)) => area < ba || (area == ba && LIBRARY_RANK[i] < LIBRARY_RANK[bi]),
            };
            if better {
                best = Some((i, area));
            }
        }
    }
    best.map(|(i, _)| i)
}

/// Each hoop's position in the original's display table at RESOURCE.DLL
/// 0x10007120 (stride 0x104), indexed like `HOOPS`. `GetBestFittingHoop`
/// walks the library in that order and keeps the first of two equal areas.
const LIBRARY_RANK: [u8; 39] = [
    17, // Hoop C (50 x 50)
    26, // J5 No.3:MB Hoop (47 x 47)
    27, // J6 No.2:MB Hoop (66 x 66)
    24, // J3 No.13:MB Hoop (64 x 28)
    25, // J4 No.8:MB Hoop (42 x 67)
    23, // J2 No.6:MB Hoop (24 x 54)
    32, // S2:MB Hoop (28 x 50)
    31, // S1:MB Hoop (37 x 59)
    13, // M4:MB Hoop (100 x 60)
    29, // J7 No.1:MB Hoop (110 x 95)
    19, // Hoop No. 1 & 3 (126 x 90)
    15, // Hoop A (110 x 110) Ver.2.21
    20, // H1:MB Hoop (100 x 90)
    7,  // Hoop HH (100 x 90)
    14, // Hoop A(F) (126 x 110) Ver.3.0
    0,  // Hoop ST (126 x 110)
    11, // M2:MB Hoop (126 x 110)
    30, // J8 No.7:MB Hoop (120 x 121)
    9,  // 200 Hoop (50 x 50)
    8,  // 200 Hoop (140 x 140)
    3,  // Hoop FA (50 x 50)
    12, // M3:MB Hoop (50 x 50)
    16, // Hoop B (140 x 200)
    1,  // Hoop RE (140 x 200)
    5,  // Hoop AQ (120 x 180)
    38, // T6:MB Hoop (78 x 168)
    6,  // Hoop ASQ (180 x 180)
    2,  // Hoop SQ (200 x 200)
    18, // Hoop D (220 x 190) (Giga Hoop)
    4,  // Hoop MA (200 x 280)
    10, // M1:MB Hoop (240 x 200)
    28, // J6 No.9:MB Hoop (66 x 66)
    33, // T1:MB Hoop (D34)
    21, // J1 No.11:MB Hoop (D30)
    22, // J1 No.12:MB Hoop (D30)
    34, // T2:MB Hoop (D54)
    35, // T3:MB Hoop (D84)
    36, // T4:MB Hoop (D112)
    37, // T5:MB Hoop (D142)
];

/// The preset with the largest area, used as the starting hoop when an
/// imported design is bigger than every preset. The original returns no hoop
/// in that case; the rebuild keeps the biggest one so the design stays visible.
pub fn largest_preset() -> usize {
    let mut best = 0;
    for (i, hoop) in HOOPS.iter().enumerate() {
        if hoop.1 * hoop.2 > HOOPS[best].1 * HOOPS[best].2 {
            best = i;
        }
    }
    best
}

/// The five presets design files saved before version 2 refer to by index.
/// Kept only so those files open at the size they were saved with; every new
/// design uses `HOOPS`.
const LEGACY_HOOPS: [(&str, f32, f32); 5] = [
    ("140 x 140 mm", 140.0, 140.0),
    ("100 x 100 mm", 100.0, 100.0),
    ("200 x 200 mm", 200.0, 200.0),
    ("230 x 300 mm", 230.0, 300.0),
    ("260 x 400 mm", 260.0, 400.0),
];

impl Project {
    fn preset(&self) -> (f32, f32) {
        if self.version < 2 {
            let h = LEGACY_HOOPS[self.hoop.min(LEGACY_HOOPS.len() - 1)];
            return (h.1, h.2);
        }
        let h = HOOPS[self.hoop.min(HOOPS.len() - 1)];
        (h.1, h.2)
    }

    /// The hoop the design is drawn in, in millimetres. The two sides are
    /// freely editable, so any hoop size can be used, not just the presets.
    pub fn hoop_size(&self) -> (f32, f32) {
        let (pw, ph) = self.preset();
        let clean = |v: f32, fallback: f32| {
            if v.is_finite() && v > 0.5 {
                v.clamp(HOOP_MIN_MM, HOOP_MAX_MM)
            } else {
                fallback
            }
        };
        (clean(self.hoop_w, pw), clean(self.hoop_h, ph))
    }

    /// Set both sides at once, e.g. after picking a preset or typing a size.
    pub fn set_hoop_size(&mut self, w: f32, h: f32) {
        self.hoop_w = if w.is_finite() {
            w.clamp(HOOP_MIN_MM, HOOP_MAX_MM)
        } else {
            DEFAULT_HOOP.0
        };
        self.hoop_h = if h.is_finite() {
            h.clamp(HOOP_MIN_MM, HOOP_MAX_MM)
        } else {
            DEFAULT_HOOP.1
        };
    }

    /// Give the design one of the preset hoops.
    pub fn set_hoop_preset(&mut self, i: usize) {
        let i = i.min(HOOPS.len() - 1);
        self.hoop = i;
        // Picking from the current catalogue upgrades an old file.
        self.version = self.version.max(2);
        let (w, h) = (HOOPS[i].1, HOOPS[i].2);
        self.set_hoop_size(w, h);
    }

    /// Index of the preset that matches the current size, if any.
    pub fn matching_preset(&self) -> Option<usize> {
        if self.version < 2 {
            return None;
        }
        let (w, h) = self.hoop_size();
        HOOPS
            .iter()
            .position(|p| (p.1 - w).abs() < 0.05 && (p.2 - h).abs() < 0.05)
    }

    /// Hoop name for labels: the preset's own name, or the size when the
    /// person typed a size of their own.
    pub fn hoop_name(&self) -> String {
        if self.version < 2 {
            let h = LEGACY_HOOPS[self.hoop.min(LEGACY_HOOPS.len() - 1)];
            return h.0.to_string();
        }
        match self.matching_preset() {
            Some(i) => HOOPS[i].0.to_string(),
            None => {
                let (w, h) = self.hoop_size();
                format!("{} × {} mm", fmt_mm(w), fmt_mm(h))
            }
        }
    }

    /// Sewing speed in stitches per second, falling back to the default.
    pub fn stitches_per_second(&self) -> f32 {
        if self.rate.is_finite() && self.rate >= RATE_MIN_SPS * 0.5 {
            self.rate.clamp(RATE_MIN_SPS, RATE_MAX_SPS)
        } else {
            DEFAULT_RATE_SPS
        }
    }

    pub fn set_stitches_per_second(&mut self, v: f32) {
        self.rate = if v.is_finite() {
            v.clamp(RATE_MIN_SPS, RATE_MAX_SPS)
        } else {
            DEFAULT_RATE_SPS
        };
    }

    pub fn push(&mut self, mut o: Obj) -> u32 {
        o.id = self.next_id;
        self.next_id += 1;
        let id = o.id;
        self.objects.push(o);
        id
    }

    pub fn get(&self, id: u32) -> Option<&Obj> {
        self.objects.iter().find(|o| o.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Obj> {
        self.objects.iter_mut().find(|o| o.id == id)
    }

    pub fn bounds(&self) -> Option<(f32, f32, f32, f32)> {
        let mut it = self.objects.iter().filter(|o| o.visible);
        let first = it.next()?;
        let mut b = first.bounds();
        for o in it {
            let ob = o.bounds();
            b.0 = b.0.min(ob.0);
            b.1 = b.1.min(ob.1);
            b.2 = b.2.max(ob.2);
            b.3 = b.3.max(ob.3);
        }
        Some(b)
    }
}

// ---------------------------------------------------------------- stitch data

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum El {
    /// Travel without stitching.
    Move(P),
    /// Sew to this point from the previous stitch.
    Stitch(P),
}

#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub el: El,
    pub color: usize,
    pub obj: u32,
}

pub fn path_bounds(path: &[P]) -> (P, f32, f32) {
    if path.is_empty() {
        return ([0.0, 0.0], 0.0, 0.0);
    }
    let mut min = [f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN];
    for p in path {
        min[0] = min[0].min(p[0]);
        min[1] = min[1].min(p[1]);
        max[0] = max[0].max(p[0]);
        max[1] = max[1].max(p[1]);
    }
    (
        [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0],
        (max[0] - min[0]).max(0.0),
        (max[1] - min[1]).max(0.0),
    )
}

fn dist_to_seg(p: P, a: P, b: P) -> f32 {
    let abx = b[0] - a[0];
    let aby = b[1] - a[1];
    let apx = p[0] - a[0];
    let apy = p[1] - a[1];
    let den = abx * abx + aby * aby;
    let t = if den <= 1e-6 {
        0.0
    } else {
        ((apx * abx + apy * aby) / den).clamp(0.0, 1.0)
    };
    let dx = apx - abx * t;
    let dy = apy - aby * t;
    (dx * dx + dy * dy).sqrt()
}

/// Resample a polyline to roughly `step` millimetre spacing.
fn resample(poly: &[P], step: f32, closed: bool) -> Vec<P> {
    let step = step.max(0.2);
    let mut pts: Vec<P> = Vec::new();
    if poly.is_empty() {
        return pts;
    }
    pts.push(poly[0]);
    let n = poly.len();
    let last = if closed { n } else { n - 1 };
    for i in 0..last {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-4 {
            continue;
        }
        let count = ((len / step).round() as usize).max(1);
        for k in 1..=count {
            let t = k as f32 / count as f32;
            pts.push([a[0] + dx * t, a[1] + dy * t]);
        }
    }
    pts
}

fn transform(o: &Obj, pts: &[P]) -> Vec<P> {
    pts.iter().map(|p| o.to_world(*p)).collect()
}

pub fn object_stitches(o: &Obj) -> Vec<El> {
    if !o.visible {
        return Vec::new();
    }
    if o.shape == ShapeKind::Stitches {
        let mut out = Vec::with_capacity(o.path.len());
        for (i, p) in o.path.iter().enumerate() {
            let wp = o.to_world(o.to_local(*p));
            out.push(if i == 0 { El::Move(wp) } else { El::Stitch(wp) });
        }
        return out;
    }
    match o.kind {
        StitchKind::Run => run_stitches(o),
        StitchKind::Satin => satin_stitches(o),
        StitchKind::Fill => fill_stitches(o),
    }
}

fn run_stitches(o: &Obj) -> Vec<El> {
    let local = o.local_outline();
    if local.len() < 2 {
        return Vec::new();
    }
    let closed = o.shape != ShapeKind::Path;
    let pts = transform(o, &resample(&local, o.stitch_len, closed));
    let mut out = Vec::with_capacity(pts.len() + 1);
    for (i, p) in pts.iter().enumerate() {
        if i == 0 {
            out.push(El::Move(*p));
        } else {
            out.push(El::Stitch(*p));
        }
    }
    out
}

fn satin_stitches(o: &Obj) -> Vec<El> {
    let local = o.local_outline();
    if local.len() < 2 {
        return Vec::new();
    }
    let closed = o.shape != ShapeKind::Path;
    let cl = resample(&local, o.density.clamp(0.2, 1.5), closed);
    if cl.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(cl.len() * 2);
    for (i, p) in cl.iter().enumerate() {
        let nxt = cl[(i + 1) % cl.len()];
        let prv = cl[(i + cl.len() - 1) % cl.len()];
        let mut tx = nxt[0] - prv[0];
        let mut ty = nxt[1] - prv[1];
        let len = (tx * tx + ty * ty).sqrt();
        if len < 1e-5 {
            tx = 1.0;
            ty = 0.0;
        } else {
            tx /= len;
            ty /= len;
        }
        let nx = -ty;
        let ny = tx;
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        let half = o.satin_w / 2.0 * side;
        let pt = [p[0] + nx * half, p[1] + ny * half];
        let wp = o.to_world(pt);
        if i == 0 {
            out.push(El::Move(wp));
        } else {
            out.push(El::Stitch(wp));
        }
    }
    out
}

fn fill_stitches(o: &Obj) -> Vec<El> {
    let poly = o.local_outline();
    if poly.len() < 3 {
        return Vec::new();
    }
    let mut miny = f32::MAX;
    let mut maxy = f32::MIN;
    for p in &poly {
        miny = miny.min(p[1]);
        maxy = maxy.max(p[1]);
    }
    let density = o.density.clamp(0.3, 2.0);
    let step = o.stitch_len.max(0.8);
    let rows = (((maxy - miny) / density).ceil() as usize).max(1);
    let mut out: Vec<El> = Vec::new();
    let mut flip = false;
    for r in 0..rows {
        let y = miny + density * (r as f32 + 0.5);
        if y > maxy {
            break;
        }
        let mut xs: Vec<f32> = Vec::new();
        let n = poly.len();
        for i in 0..n {
            let a = poly[i];
            let b = poly[(i + 1) % n];
            if (a[1] <= y && b[1] > y) || (b[1] <= y && a[1] > y) {
                let t = (y - a[1]) / (b[1] - a[1]);
                xs.push(a[0] + (b[0] - a[0]) * t);
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut k = 0;
        while k + 1 < xs.len() {
            let mut x0 = xs[k];
            let mut x1 = xs[k + 1];
            if x1 - x0 < 0.05 {
                k += 2;
                continue;
            }
            if flip {
                std::mem::swap(&mut x0, &mut x1);
            }
            let len = (x1 - x0).abs();
            let segs = ((len / step).round() as usize).max(1);
            let mut row: Vec<P> = Vec::with_capacity(segs + 1);
            for s in 0..=segs {
                let t = s as f32 / segs as f32;
                row.push([x0 + (x1 - x0) * t, y]);
            }
            for (i, p) in row.iter().enumerate() {
                let wp = o.to_world(*p);
                if i == 0 {
                    out.push(El::Move(wp));
                } else {
                    out.push(El::Stitch(wp));
                }
            }
            k += 2;
        }
        flip = !flip;
    }
    out
}

// -------------------------------------------------------------------- project

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub stitches: usize,
    pub colors: usize,
    pub minutes: f32,
    pub w: f32,
    pub h: f32,
}

pub fn timeline(p: &Project) -> Vec<Step> {
    let mut out: Vec<Step> = Vec::new();
    for o in &p.objects {
        if !o.visible {
            continue;
        }
        for el in object_stitches(o) {
            out.push(Step {
                el,
                color: o.color,
                obj: o.id,
            });
        }
    }
    out
}

pub fn stats(p: &Project) -> Stats {
    let seq = timeline(p);
    let stitches = seq.iter().filter(|s| matches!(s.el, El::Stitch(_))).count();
    let mut colors: Vec<usize> = Vec::new();
    for s in &seq {
        if !colors.contains(&s.color) {
            colors.push(s.color);
        }
    }
    let (w, h) = match p.bounds() {
        Some(b) => (b.2 - b.0, b.3 - b.1),
        None => (0.0, 0.0),
    };
    Stats {
        stitches,
        colors: colors.len(),
        // Sewing time follows the speed the person set: stitches divided by
        // stitches per second, in minutes.
        minutes: stitches as f32 / (p.stitches_per_second().max(0.1) * 60.0),
        w,
        h,
    }
}

// ------------------------------------------------------------- example design

/// The built-in flower example is exactly 8 cm by 8 cm.
pub const EXAMPLE_SIDE_MM: f32 = 80.0;

/// A flower motif whose stitches fill an 8 x 8 cm square exactly.
///
/// The design is drawn at a comfortable size, then shifted to the hoop centre
/// and scaled as a whole so its outer edge lands on 80 mm in both directions.
pub fn flower_example() -> Project {
    let mut p = Project {
        name: "Flower".to_string(),
        ..Project::default()
    };
    let tau = std::f32::consts::TAU;

    // Eight long outer petals, alternating between two rose tones, each edged
    // with a fine run stitch in a deeper shade.
    for k in 0..8 {
        let a = k as f32 * tau / 8.0;
        let mut petal = Obj::new_ellipse(
            0,
            StitchKind::Fill,
            a.cos() * 23.5,
            a.sin() * 23.5,
            16.0,
            33.0,
        );
        petal.rot = a + std::f32::consts::FRAC_PI_2;
        petal.color = if k % 2 == 0 { 8 } else { 10 };
        petal.density = 0.6;
        p.push(petal);

        let mut edge = Obj::new_ellipse(
            0,
            StitchKind::Run,
            a.cos() * 23.5,
            a.sin() * 23.5,
            16.0,
            33.0,
        );
        edge.rot = a + std::f32::consts::FRAC_PI_2;
        edge.color = 7;
        edge.stitch_len = 1.8;
        p.push(edge);
    }

    // Eight shorter inner petals, offset by half a step, for depth.
    for k in 0..8 {
        let a = (k as f32 + 0.5) * tau / 8.0;
        let mut petal = Obj::new_ellipse(
            0,
            StitchKind::Fill,
            a.cos() * 12.5,
            a.sin() * 12.5,
            11.0,
            19.0,
        );
        petal.rot = a + std::f32::consts::FRAC_PI_2;
        petal.color = 9;
        petal.density = 0.5;
        p.push(petal);
    }

    // Satin ring around the heart of the flower.
    let mut ring = Obj::new_ellipse(0, StitchKind::Satin, 0.0, 0.0, 38.0, 38.0);
    ring.satin_w = 2.8;
    ring.density = 0.45;
    ring.color = 4;
    p.push(ring);

    // Gold centre.
    let mut c = Obj::new_ellipse(0, StitchKind::Fill, 0.0, 0.0, 28.0, 28.0);
    c.color = 3;
    c.density = 0.5;
    p.push(c);

    fit_square(&mut p, EXAMPLE_SIDE_MM);
    p
}

/// The design the program opens with: the 8 x 8 cm flower.
pub fn demo_project() -> Project {
    flower_example()
}

/// Shift a design to the hoop centre and scale it so its bounds are `side`
/// millimetres square.
fn fit_square(p: &mut Project, side: f32) {
    let Some(b) = p.bounds() else {
        return;
    };
    let w = (b.2 - b.0).max(0.001);
    let h = (b.3 - b.1).max(0.001);
    let s = side / w.max(h);
    let cx = (b.0 + b.2) / 2.0;
    let cy = (b.1 + b.3) / 2.0;
    for o in &mut p.objects {
        o.cx = (o.cx - cx) * s;
        o.cy = (o.cy - cy) * s;
        o.w *= s;
        o.h *= s;
        o.satin_w *= s;
        for q in o.path.iter_mut() {
            q[0] = (q[0] - cx) * s;
            q[1] = (q[1] - cy) * s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flower_example_is_exactly_eight_centimetres_square() {
        let p = flower_example();
        let b = p.bounds().unwrap();
        let w = b.2 - b.0;
        let h = b.3 - b.1;
        assert!((w - 80.0).abs() < 0.05, "flower width is {w} mm, not 80");
        assert!((h - 80.0).abs() < 0.05, "flower height is {h} mm, not 80");
        assert!((b.0 + 40.0).abs() < 0.05, "left edge at {} mm", b.0);
        assert!((b.2 - 40.0).abs() < 0.05, "right edge at {} mm", b.2);
        assert!((b.1 + 40.0).abs() < 0.05, "bottom edge at {} mm", b.1);
        assert!((b.3 - 40.0).abs() < 0.05, "top edge at {} mm", b.3);

        // The stitches themselves — not just the outlines — reach the edges of
        // the square, so the sewn design is 8 cm by 8 cm too.
        let mut maxx: f32 = 0.0;
        let mut maxy: f32 = 0.0;
        for st in timeline(&p) {
            let q = match st.el {
                El::Stitch(q) | El::Move(q) => q,
            };
            assert!(q[0].abs() <= 40.01, "stitch x {} outside the square", q[0]);
            assert!(q[1].abs() <= 40.01, "stitch y {} outside the square", q[1]);
            maxx = maxx.max(q[0].abs());
            maxy = maxy.max(q[1].abs());
        }
        assert!((maxx - 40.0).abs() < 0.05, "stitches stop at {maxx} mm");
        assert!((maxy - 40.0).abs() < 0.05, "stitches stop at {maxy} mm");

        let st = stats(&p);
        assert!(st.stitches > 2000, "flower is too sparse: {}", st.stitches);
        assert!(st.colors >= 4, "flower should use several threads");
        assert!((st.w - 80.0).abs() < 0.05 && (st.h - 80.0).abs() < 0.05);
    }

    #[test]
    fn demo_design_has_stitches_and_fits() {
        let p = demo_project();
        let st = stats(&p);
        assert!(p.objects.len() >= 5, "demo should have several objects");
        assert!(
            st.stitches > 1000,
            "demo should produce a real stitch count, got {}",
            st.stitches
        );
        assert!(st.colors >= 4);
        let (hw, hh) = p.hoop_size();
        let b = p.bounds().unwrap();
        assert!(
            b.0 >= -hw / 2.0 && b.2 <= hw / 2.0,
            "width {} outside hoop",
            st.w
        );
        assert!(
            b.1 >= -hh / 2.0 && b.3 <= hh / 2.0,
            "height {} outside hoop",
            st.h
        );
    }

    #[test]
    fn fill_produces_a_dense_stitch_field() {
        let o = Obj::new_rect(1, StitchKind::Fill, 0.0, 0.0, 20.0, 20.0);
        let el = object_stitches(&o);
        let stitches = el.iter().filter(|e| matches!(e, El::Stitch(_))).count();
        assert!(stitches > 200, "expected a dense fill, got {stitches}");
        let moves = el.iter().filter(|e| matches!(e, El::Move(_))).count();
        assert!(
            moves > 5,
            "fill should start a new row repeatedly, got {moves}"
        );
    }

    #[test]
    fn satin_makes_a_zigzag() {
        let mut o = Obj::new_rect(1, StitchKind::Satin, 0.0, 0.0, 40.0, 40.0);
        o.satin_w = 6.0;
        let pts: Vec<P> = object_stitches(&o)
            .iter()
            .filter_map(|e| match e {
                El::Stitch(p) | El::Move(p) => Some(*p),
            })
            .collect();
        assert!(pts.len() > 50);
        let mut spread = false;
        for w in pts.windows(2) {
            let d = ((w[0][0] - w[1][0]).powi(2) + (w[0][1] - w[1][1]).powi(2)).sqrt();
            if d > 3.0 {
                spread = true;
            }
        }
        assert!(spread, "satin should step across the column");
    }

    #[test]
    fn run_walks_the_outline() {
        let o = Obj::new_ellipse(1, StitchKind::Run, 0.0, 0.0, 30.0, 30.0);
        let pts: Vec<P> = object_stitches(&o)
            .iter()
            .filter_map(|e| match e {
                El::Stitch(p) => Some(*p),
                _ => None,
            })
            .collect();
        assert!(pts.len() > 20);
        for p in &pts {
            let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
            assert!((r - 15.0).abs() < 1.5, "run stitch left the outline: {r}");
        }
    }

    #[test]
    fn rotation_moves_the_bounds() {
        let mut o = Obj::new_rect(1, StitchKind::Fill, 0.0, 0.0, 40.0, 10.0);
        let b0 = o.bounds();
        o.rot = std::f32::consts::FRAC_PI_2;
        let b1 = o.bounds();
        assert!((b0.2 - b0.0 - 40.0).abs() < 0.01);
        assert!(
            (b1.2 - b1.0 - 10.0).abs() < 0.5,
            "rotated box should be narrow"
        );
        assert!(
            (b1.3 - b1.1 - 40.0).abs() < 0.5,
            "rotated box should be tall"
        );
    }

    #[test]
    fn hoop_size_is_free_not_only_the_presets() {
        let mut p = Project::default();
        assert_eq!(p.hoop_size(), (140.0, 200.0));
        assert_eq!(p.hoop_name(), "Hoop B (140 x 200)");

        p.set_hoop_size(187.5, 243.0);
        assert_eq!(p.hoop_size(), (187.5, 243.0));
        assert_eq!(p.matching_preset(), None);
        assert_eq!(p.hoop_name(), "187.5 × 243 mm");

        // Presets still work and set both sides.
        p.set_hoop_preset(0);
        assert_eq!(p.hoop_size(), (50.0, 50.0));
        assert_eq!(p.hoop_name(), "Hoop C (50 x 50)");

        // Typed values stay inside sensible limits.
        p.set_hoop_size(1.0, 5000.0);
        assert_eq!(p.hoop_size(), (HOOP_MIN_MM, HOOP_MAX_MM));

        // The design check follows the size that was typed.
        let mut q = flower_example(); // 80 mm wide
        q.set_hoop_size(60.0, 60.0);
        let b = q.bounds().unwrap();
        let (hw, hh) = q.hoop_size();
        assert!(
            b.2 > hw / 2.0 || b.3 > hh / 2.0,
            "an 8 cm design cannot fit a 6 cm hoop"
        );
    }

    #[test]
    fn the_smallest_hoop_that_fits_is_chosen() {
        // An 80 × 80 mm design: the 84 × 84 mm hoop (T3) has the smallest
        // area of the presets that can hold it, now that the D-sized hoops
        // from the original's hoop file are part of the catalogue.
        let i = best_preset_for(80.0, 80.0).expect("something fits 80 mm");
        assert_eq!(HOOPS[i].1, 84.0);
        assert_eq!(HOOPS[i].2, 84.0);

        // The original's comparison is strictly smaller, so a 50 × 50 mm
        // design does not fit a 50 × 50 mm hoop; the next size up wins.
        let i = best_preset_for(50.0, 50.0).expect("something fits just over 50 mm");
        assert_eq!((HOOPS[i].1, HOOPS[i].2), (54.0, 54.0));

        // Exactly the same size as a hoop's side is also too big.
        assert_ne!(best_preset_for(140.0, 200.0), Some(default_hoop_preset()));

        // Four presets are 50 × 50. The original keeps the first of equal
        // areas in its library order (RESOURCE.DLL 0x10007120), and Hoop FA
        // comes before 200 Hoop (50 x 50), M3 and Hoop C there.
        let i = best_preset_for(48.0, 48.0).expect("something fits 48 mm");
        assert_eq!(HOOPS[i].0, "Hoop FA (50 x 50)");

        // Nothing in the catalogue takes a 300 mm design.
        assert_eq!(best_preset_for(300.0, 300.0), None);
    }

    #[test]
    fn sewing_time_follows_the_stitch_rate() {
        let mut p = flower_example();
        p.set_stitches_per_second(10.0);
        let st = stats(&p);
        let n = st.stitches as f32;
        assert!(
            (st.minutes - n / 600.0).abs() < 0.01,
            "{} vs {}",
            st.minutes,
            n / 600.0
        );

        p.set_stitches_per_second(25.0);
        let faster = stats(&p);
        assert!((faster.minutes - n / 1500.0).abs() < 0.01);
        assert!(faster.minutes < st.minutes);

        // A design that forgot to store a rate falls back to the default.
        let mut q = flower_example();
        q.rate = 0.0;
        assert_eq!(q.stitches_per_second(), DEFAULT_RATE_SPS);
        assert_eq!(
            stats(&q).minutes,
            st.stitches as f32 / (DEFAULT_RATE_SPS * 60.0)
        );
    }

    #[test]
    fn older_design_files_still_open() {
        // A file saved before hoop sizes and speeds were editable.
        let old = r#"{"version":1,"name":"Old","hoop":3,"objects":[],"next_id":1}"#;
        let p: Project = serde_json::from_str(old).expect("old design should load");
        assert_eq!(p.hoop_size(), (230.0, 300.0));
        assert_eq!(p.stitches_per_second(), DEFAULT_RATE_SPS);
        assert_eq!(p.hoop_name(), "230 x 300 mm");
    }
}
