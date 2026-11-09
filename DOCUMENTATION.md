# Stitch Studio — Repository Documentation

> Generated on 2026-11-09 from commit `25cd889eacf3c0e89e6653170dd7d387e0a5960e`.
> Repository: <https://github.com/ZiTAL/hari-tzatxu>

This document describes everything in the repository: the application, its
architecture and algorithms, the file formats it reads and writes, the user
interface, the translation system, the build setup, the test and check
artifacts, and the reverse-engineering evidence that informed the rebuild.

---

## Contents

1. [What this repository is](#1-what-this-repository-is)
2. [Repository map](#2-repository-map)
3. [Building and running](#3-building-and-running)
4. [Architecture](#4-architecture)
5. [Data model](#5-data-model)
6. [Stitch generation](#6-stitch-generation)
7. [File formats and I/O](#7-file-formats-and-io)
8. [User interface](#8-user-interface)
9. [Internationalisation](#9-internationalisation)
10. [Theme and rendering](#10-theme-and-rendering)
11. [Tests and build checks](#11-tests-and-build-checks)
12. [Provenance and reverse engineering](#12-provenance-and-reverse-engineering)
13. [Development history](#13-development-history)
14. [Extending the program](#14-extending-the-program)
15. [Known limitations](#15-known-limitations)
16. [Appendices](#16-appendices)

---

## 1. What this repository is

Stitch Studio is a native desktop embroidery workspace written in Rust with
the `egui`/`eframe` GUI toolkit. It is a **new program**, not a patched copy of
the original Windows application.

The request that started the project, recorded in `craft.json`, was:

> “I would like to remove the USB dongle requirement so that it works without
> it, and block JANOME.EXE from making internet connections”

The result is a program that:

- starts without a USB key, licence screen, sign-in, or dongle check;
- contains no networking code and opens no sockets;
- draws embroidery shapes on hoops measured in millimetres;
- generates run, satin, and fill stitches and previews them step by step;
- saves/loads its own JSON design format;
- imports and exports DST stitch files;
- exports SVG drawings and CSV stitch lists;
- ships with English, Spanish, and Basque interface text.

The original `JANOME.EXE` and its DLLs are **not included** in this repository.
The original files were read only; they were never modified or redistributed.

### Quick facts

| Property | Value |
|---|---|
| Application name | Stitch Studio |
| Cargo package / binary | `stitch_studio` |
| Version | 0.1.0 |
| Language / edition | Rust, edition 2021 |
| GUI | `eframe` / `egui` 0.28.1 |
| Image output | `image` 0.25.10, PNG only |
| Serialisation | `serde` 1.0.229, `serde_json` 1.0.154 |
| Default window | 1280 × 820 px; minimum 1024 × 660 px |
| Units | Millimetres; origin at hoop centre; y axis points up in the model |
| Hoop presets | 39, plus any custom size from 20 to 600 mm per side |
| Default hoop | Hoop B, 140 × 200 mm |
| Thread palette | 30 fixed colours |
| Default speed | 12.5 stitches/second (750 stitches/minute) |
| Interface languages | English (`en`), Spanish (`es`), Basque (`eu`) |
| Translation keys | 239 per language file |
| Tests | 27 unit/UI-render tests |
| Network access | None |
| Dongle / licence check | None |
| Licence file | Not present in the repository |

### Status

`craft.json` reports `"status": "ready"` and `"note": "Ready: all 2 checks
pass."`. The two checks are a successful `cargo build` and an
`app-screenshot` run at 1280 × 820. All 27 Rust tests pass in the current
checkout.

---

## 2. Repository map

The project is not a Cargo workspace at the root; the Rust package lives in
`app/`.

```text
hari-tzatxu/
├── .cargo/
│   └── config.toml                 Build settings used when cargo is run at the root
├── .gitignore                      Ignores Rust build output and shim logs
├── app/
│   ├── .cargo/
│   │   ├── config.toml             Build settings used when cargo is run in app/
│   │   └── rustc-shim.sh           Compiler wrapper for stripped-PATH/deleted-cwd builds
│   ├── Cargo.toml                  Package manifest
│   ├── Cargo.lock                  Locked dependency graph
│   ├── README.md                   Short user-facing readme
│   ├── MECHANICS.md                Detailed reverse-engineering dossier
│   ├── TRANSLATING.md              Translator guide
│   ├── lang/
│   │   ├── en.json                 English labels
│   │   ├── es.json                 Spanish labels
│   │   └── eu.json                 Basque labels
│   ├── shots/                      Screenshots from development, 1280 × 820 PNG
│   ├── src/
│   │   ├── main.rs                 Application, UI, canvas, input, tests (2721 lines)
│   │   ├── model.rs                Data model, hoops, stitch generation, tests (1177 lines)
│   │   ├── io.rs                   Project/DST/SVG/CSV I/O and file browser (868 lines)
│   │   ├── theme.rs                Palette, styling, icons, small widgets (416 lines)
│   │   └── lang.rs                 Translation loading and lookup (287 lines)
│   └── target/                     Build output, ignored by git (about 2.7 GB here)
├── capture/
│   └── README.md                   Initial clean-room note
├── checks/                         Historical verification screenshots and a request
├── investigation.cany/             REA/Cany investigation archive
│   ├── project.toml
│   ├── annotations/                7 annotation files (all empty maps)
│   ├── evidence/                   123 JSON evidence records
│   ├── journal/                    612 tool-call journal entries
│   └── unknowns/                   Empty
├── reference/                      Empty
├── pinned/play.log                 Empty file
├── inbox/build-progress.json       Build progress fingerprint
├── craft.json                      Craft Anything project/run metadata
├── feed.json                       Activity feed, version 5, 98 items
├── build-activity.log              Human-readable build/run log
└── usage.json                      Work-time/token/cost counters
```

### Artifact notes

- `app/shots/` contains development screenshots: `en.png`, `es.png`,
  `eu.png`, the corresponding `check-*` images, `round-11*.png`, and
  `after-decompile.png`.
- `checks/round-{0,2,3,4,5,7,8,9}/screenshot.png` are check artifacts from the
  recorded rounds. `checks/round-10.png`, `checks/round-10-es.png`,
  `checks/round-11.png`, and `checks/round-11-es.png` are later images that
  are not listed in `craft.json`'s check array.
- `checks/asks/1794167531586-559954-0.request.json` is a historical question
  request named “Check the build” with no choices and no text input.
- `inbox/build-progress.json` stores a fingerprint and an
  `unchanged_turns` counter.
- `usage.json` records `work_ms: 8460049` (about 2 h 21 min), zero tokens, and
  zero recorded cost.
- `investigation.cany/project.toml` names the investigation “JANOME”, sets
  purpose `own_or_authorized`, format version 1, and no allowed tools.

---

## 3. Building and running

### Requirements

- A Rust toolchain with Cargo (the checkout was verified with Cargo 1.99.1).
- A C compiler and linker. The pinned configuration uses `/usr/bin/cc`.
- Linux desktop/windowing libraries required by `eframe`/`winit` for a real
  window. `cargo test` itself runs the UI headlessly through `egui::Context`.
- No USB device, dongle, licence server, or network connection.

### Commands

```sh
cd app
cargo build              # debug build
cargo build --release    # release build
cargo test               # 27 tests
cargo run                # launch the GUI
cargo run -- --lang es   # launch in Spanish
cargo run -- --screenshot shot.png
```

On this machine the pinned build configuration sends build output to the
absolute `target-dir` in `.cargo/config.toml`, so `app/target/` may not be the
directory Cargo actually writes to. Both paths are ignored by git.

### Command-line options

`main.rs` parses arguments manually.

| Option | Meaning |
|---|---|
| `-h`, `--help` | Print usage and exit before creating a window |
| `-l <code>`, `--lang <code>` | Start in a language code such as `en`, `es`, `eu` |
| `--lang=<code>` | Same, with an equals sign |
| `-s <file>`, `--screenshot <file>` | Draw the window, save a PNG, and exit |
| `--screenshot=<file>` | Same, with an equals sign |

`--help` prints:

```text
Stitch Studio — an embroidery workspace that needs no USB key

Usage: stitch_studio [--lang <code>] [--screenshot <file.png>]
  --lang <code>            en, es or eu (or any language file in lang/)
  --screenshot <file.png>  draw a few frames, save the window as a PNG, exit

Text comes from the JSON files in the lang folder.
```

### Screenshot mode

When `--screenshot` is present, the app records the path in `Shot`. After 120
rendered frames it sends `egui::ViewportCommand::Screenshot`; when the
screenshot event arrives, it reads the RGBA pixels, creates the parent
directory if needed, writes the PNG with `image::save_buffer`, prints
`screenshot written to …`, and closes the window. If no image arrives by frame
600, it prints `screenshot timed out` and closes.

### Environment variables

| Variable | Effect |
|---|---|
| `STITCH_LANG_DIR` | First candidate directory for language files, before `./lang` |
| `XDG_CONFIG_HOME` | Base for the saved language setting |
| `HOME` | Fallback for `~/.config`, default save/open folder, and the font search |

The saved language file is:

```text
$XDG_CONFIG_HOME/stitch-studio/settings.json
```

or, when `XDG_CONFIG_HOME` is unset:

```text
~/.config/stitch-studio/settings.json
```

It contains only the language code:

```json
{
  "lang": "es"
}
```

### Build configuration and portability

Both `.cargo/config.toml` files pin machine-specific absolute paths:

```toml
[build]
rustc = "/home/zital/Documents/Craft Anything/janome/app/.cargo/rustc-shim.sh"
rustdoc = "/home/zital/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc"
target-dir = "/home/zital/Documents/Craft Anything/janome/app/target"

[target.x86_64-unknown-linux-gnu]
linker = "/usr/bin/cc"

[target.aarch64-unknown-linux-gnu]
linker = "/usr/bin/cc"
```

The root copy exists because Cargo ignores `app/.cargo/config.toml` when it is
launched from the repository root; the two files are meant to be kept in sync.

`rustc-shim.sh` exists because the original build checker started Cargo from a
temporary directory that could be deleted before `rustc` ran, with a `PATH`
containing only the Rust tools. The shim:

1. determines its own directory using shell parameter expansion only;
2. changes into the `app/` directory one level above it;
3. restores a normal `PATH` and a fallback `HOME`;
4. optionally writes one line to `.cargo/shim.log` when it had to repair an
   invalid working directory;
5. executes the real `rustc` with all arguments unchanged.

For a fresh clone on a different machine, one of these is required:

- update the three absolute paths in both config files; or
- remove the `rustc`, `rustdoc`, and `target-dir` overrides and build with the
  normal system toolchain.

The `linker = "/usr/bin/cc"` entries can stay if `cc` exists there, otherwise
change them to the local compiler path.

---

## 4. Architecture

### Modules

| File | Lines | Responsibility |
|---|---:|---|
| `app/src/main.rs` | 2721 | `App` state, startup, CLI, all panels, canvas painting/input, menus, dialogs, simulation, screenshots, UI tests |
| `app/src/model.rs` | 1177 | Hoop catalogue, thread palette, `Obj`/`Project`, geometry, stitch generators, timeline, statistics, flower example, model tests |
| `app/src/io.rs` | 868 | Project JSON, DST import/export, SVG export, CSV export, in-app file browser, I/O tests |
| `app/src/theme.rs` | 416 | Colour palette, global `egui` style, cards/sections, hand-drawn icons, segmented control |
| `app/src/lang.rs` | 287 | Embedded/on-disk translations, language discovery, fallback, placeholders, saved choice, language tests |

### Startup flow

```text
main()
  ├── handle --help and exit
  ├── parse --lang / --screenshot
  ├── lang::init(requested)
  │     ├── requested language if it exists
  │     ├── else saved language if it exists
  │     └── else English
  ├── build eframe window options (1280×820, min 1024×660)
  └── run_native
        ├── install_fonts(ctx)
        ├── theme::apply(ctx)
        └── App::new(shot)
              ├── project = model::demo_project()   (8×8 cm flower)
              ├── seq = model::timeline(project)
              ├── stats = model::stats(project)
              └── default view/simulation/tool state
```

### Per-frame flow

`eframe::App::update` calls `App::draw` and then requests another repaint when
a screenshot is pending, the simulation is playing, or a toast is visible.
`draw` runs, in order:

```text
tick_toast()       decrement the toast timer
tick_sim()         advance the stitch simulation
shortcuts()        handle keyboard commands
chrome()           top bar: logo, menus, language picker, toolbar
bottom()           transport bar and status bar
left_panel()       design card, summary, object list
right_panel()      Threads / Settings / Plan tabs
canvas()           fabric, grid, rulers, stitches, selection, simulation
handle_file_pick() act on a file chosen in the dialog
dialog.show()      in-app file browser
help_window()      Help window
about_window()     About window
```

The state is centralised in `App`:

| Group | Fields |
|---|---|
| Document | `project: Project`, `sel: Option<u32>`, `dirty: bool` |
| Tools and view | `tool: Tool`, `view: View { zoom, pan, needs_fit }`, `drag: Drag` |
| Display toggles | `show_stitches`, `show_grid`, `show_rulers`, `snap` |
| Simulation | `sim: Sim { idx, playing, mult, acc }` |
| Derived data | `seq: Vec<Step>`, `stats: Stats` |
| History | `undo: Vec<Vec<Obj>>`, `redo: Vec<Vec<Obj>>`, `prop_drag`, `hoop_drag` |
| Transient UI | `tab`, `help_open`, `about_open`, `toast`, `dialog`, `shot`, `quit` |
| Build identity | `APP_TITLE = "Stitch Studio"` |

Every model change follows the same pattern:

```text
push_undo()  (when appropriate)
mutate Project
rebuild()    → recompute seq and stats, clamp simulation index, clear stale selection
```

### Data flow

```text
             ┌──────────────┐
 user input  │  App / UI    │  save/open/import/export
 ──────────► │  main.rs     │ ──────────────────────► io.rs
             └──────┬───────┘
                    │ mutates
                    ▼
             ┌──────────────┐
             │ Project/Obj  │  model.rs
             │  (mm, y-up)  │
             └──────┬───────┘
                    │ timeline()
                    ▼
             ┌──────────────┐
             │ Vec<Step>    │  Move / Stitch + colour + object id
             └──┬────┬───┬──┘
                │    │   │
       stats()  │    │   └────────► canvas painter
                ▼    └────────────► DST / SVG / CSV export
             Stats
```

---

## 5. Data model

### Coordinates and units

- All geometry is stored in **millimetres**.
- The origin is the **centre of the hoop**.
- The model’s y axis points **up**; screen painting negates y.
- Rotation is in **radians**, counter-clockwise.
- DST data is quantised to 0.1 mm (one DST unit).
- Screen conversion is:

```rust
screen = rect.center() + pan + (mm.x * zoom, -mm.y * zoom)
mm     = ((screen - rect.center() - pan).x / zoom,
          -(screen - rect.center() - pan).y / zoom)
```

### Core types

| Type | Definition / purpose |
|---|---|
| `P` | `[f32; 2]`, a millimetre point |
| `StitchKind` | `Run`, `Satin`, `Fill` |
| `ShapeKind` | `Rect`, `Ellipse`, `Path`, `Stitches` |
| `Obj` | One design object, stored in hoop coordinates |
| `Project` | Design document: name, hoop, rate, objects, next id |
| `El` | `Move(P)` travel or `Stitch(P)` sewn point |
| `Step` | `{ el, color, obj }`, one entry in the machine timeline |
| `Stats` | Stitch count, colour count, estimated minutes, bounds |

### `Obj`

| Field | Type | Meaning |
|---|---|---|
| `id` | `u32` | Unique object id within the project |
| `kind` | `StitchKind` | Run, satin, or fill |
| `shape` | `ShapeKind` | Rectangle, ellipse, freehand path, imported stitch chain |
| `cx`, `cy` | `f32` | Object centre in hoop millimetres |
| `w`, `h` | `f32` | Local width and height in millimetres |
| `rot` | `f32` | Rotation around the centre, radians |
| `path` | `Vec<P>` | World points for `Path`/`Stitches`; empty otherwise |
| `color` | `usize` | Index into `THREADS` |
| `visible` | `bool` | Hidden objects are skipped by timeline, stats, and exports |
| `stitch_len` | `f32` | Run/fill step spacing, default 2.5 mm |
| `density` | `f32` | Fill/satin row spacing, default 0.45 / 0.4 mm |
| `satin_w` | `f32` | Satin column width, default 5.0 or 6.0 mm |

Constructors:

- `Obj::new_rect(id, kind, cx, cy, w, h)` — minimum size 1 mm, colour 6
  (Scarlet), visible, run length 2.5 mm, density 0.4 for satin / 0.45
  otherwise, satin width 5 mm.
- `Obj::new_ellipse(...)` — same as rectangle, `shape = Ellipse`.
- `Obj::new_path(id, kind, path)` — computes centre and bounds from the points;
  satin width 6 mm.
- `Obj::new_stitches(id, color, path)` — an imported chain; run kind,
  `shape = Stitches`, minimum size 0.1 mm; all points are kept exactly.

### `Project`

| Field | Type | Meaning |
|---|---|---|
| `version` | `u32` | Current documents use 2; version 1 uses legacy hoop indices |
| `name` | `String` | Design name; used for window labels and suggested filenames |
| `hoop` | `usize` | Last selected preset index, or legacy index for version 1 |
| `hoop_w` | `f32` | Actual hoop width, mm; 0 means “use the preset” |
| `hoop_h` | `f32` | Actual hoop height, mm; 0 means “use the preset” |
| `rate` | `f32` | Sewing speed in stitches/second; 0 means “use the default” |
| `objects` | `Vec<Obj>` | Draw and sewing order |
| `next_id` | `u32` | Next object id; starts at 1 |

`Project::default()` creates version 2, name `"Untitled design"`, the default
hoop preset (Hoop B, 140 × 200 mm), rate `12.5`, no objects, and `next_id = 1`.
The running app replaces the name with the translated demo name at startup.

`hoop_w`, `hoop_h`, and `rate` use `#[serde(default)]`, so older JSON files
without them still load.

### Constants

| Constant | Value | Meaning |
|---|---:|---|
| `DEFAULT_HOOP` | 140 × 200 mm | Hoop B |
| `HOOP_ALLOWANCE_MM` | 0.0 | Extra clearance used by the fit rule |
| `HOOP_MIN_MM` | 20.0 | Smallest accepted hoop side |
| `HOOP_MAX_MM` | 600.0 | Largest accepted hoop side |
| `DEFAULT_RATE_SPS` | 12.5 | Stitches per second |
| `RATE_MIN_SPS` | 0.5 | Minimum editable speed |
| `RATE_MAX_SPS` | 60.0 | Maximum editable speed |
| `EXAMPLE_SIDE_MM` | 80.0 | The flower example is exactly 8 × 8 cm |
| `THREADS` | 30 entries | Fixed thread palette |

### Hoop selection

`Project::hoop_size()` returns the stored size when valid, otherwise the
preset size, clamped to `HOOP_MIN_MM..=HOOP_MAX_MM`. A non-finite value, or a
value of 0.5 mm or less, falls back to the preset.

`Project::set_hoop_size(w, h)` clamps both sides. `set_hoop_preset(i)` selects a
preset, upgrades a version-1 document to version 2, and copies the preset’s
size into `hoop_w`/`hoop_h`.

`matching_preset()` returns a preset index when both sides are within 0.05 mm
of a catalogue entry. Otherwise the UI shows the translated “Custom size” label
and `hoop_name()` formats the size as, for example, `187.5 × 243 mm`.

`best_preset_for(w, h)` implements the rule taken from the original:

```text
for each hoop:
    if design_w + allowance < hoop_w and design_h + allowance < hoop_h:
        candidate = hoop
choose the candidate with the smallest area
on equal area, keep the one earlier in the original library order
return None if nothing fits
```

The comparison is strict, so a 50 × 50 mm design does **not** fit a 50 × 50 mm
hoop, and an 80 × 80 mm design chooses the 84 × 84 mm T3 hoop. Four presets are
50 × 50; `LIBRARY_RANK` breaks the tie using the original display-table order,
so a 48 × 48 mm design chooses **Hoop FA**.

`largest_preset()` returns Hoop MA (200 × 280 mm), used only as a temporary
starting hoop when importing a design larger than every preset.

Version-1 documents refer to five older hoop entries by index:

| Index | Legacy label | Size |
|---:|---|---|
| 0 | `140 x 140 mm` | 140 × 140 |
| 1 | `100 x 100 mm` | 100 × 100 |
| 2 | `200 x 200 mm` | 200 × 200 |
| 3 | `230 x 300 mm` | 230 × 300 |
| 4 | `260 x 400 mm` | 260 × 400 |

Selecting a current preset upgrades the document to version 2. The full
39-hoop catalogue is in [Appendix A](#appendix-a-hoop-catalogue).

### Geometry helpers

- `Obj::local_outline()` — rectangle corners, a 96-segment ellipse, or the path
  points transformed into local coordinates.
- `to_local(p)` / `to_world(p)` — rotate and translate between hoop coordinates
  and object-local coordinates.
- `bounds()` — axis-aligned bounds. For paths it uses the stored centre and
  size; for rectangles and ellipses it uses the rotated extent of the local box.
- `contains(p)` — path/stitch objects use the distance to the nearest segment
  and accept within 3.5 mm. Rectangle and ellipse objects use the normalised
  ellipse test `(x/(w/2))² + (y/(h/2))² <= 1`.
- `path_bounds(path)` — centre, width, and height of a point list.
- `dist_to_seg(p, a, b)` — point-to-segment distance.
- `resample(poly, step, closed)` — walk the polyline, subdivide each segment
  into `round(length/step)` pieces, and emit points at roughly `step` spacing.
  The minimum step is 0.2 mm.

> Hit-testing note: rectangles use the same elliptical test as ellipses, so the
> clickable area of a rectangle is an inscribed ellipse, not its full box.

---

## 6. Stitch generation

### Timeline

`model::timeline(&Project)` walks visible objects in list order and flattens
each object’s stitch elements into `Vec<Step>`. Each step carries:

- `El::Move(point)` — a travel/jump with no stitch;
- `El::Stitch(point)` — a sewn segment from the previous point;
- `color` — thread palette index;
- `obj` — owning object id.

`object_stitches(&Obj)` selects the generator by `StitchKind`, except for
`ShapeKind::Stitches`, which returns the stored points exactly: the first point
is a `Move`, every later point is a `Stitch`.

Hidden objects contribute nothing. Object order is the sewing order; the user
can reorder objects or group them by colour.

### Run stitches

`run_stitches`:

1. take `local_outline()`;
2. resample it with `stitch_len` (minimum 0.2 mm);
3. treat the shape as closed for rectangles and ellipses, open for freehand
   paths;
4. emit the first point as `Move` and all remaining points as `Stitch`.

This walks the outline; it does not fill the interior.

### Satin stitches

`satin_stitches`:

1. take the outline;
2. resample at `density` clamped to 0.2–1.5 mm;
3. at each point, estimate the tangent from the previous and next points;
4. take the perpendicular normal;
5. alternate the point to one side and the other by half of `satin_w`;
6. emit a zigzag along the centreline.

This is a simplified satin representation: the zigzag straddles the outline
rather than generating a full column between two independently offset rails.
The result is visually useful but is not a production-grade satin-column
algorithm.

### Fill stitches

`fill_stitches` is an even-odd scanline fill:

1. take the outline polygon;
2. find its vertical extent;
3. set row spacing to `density` clamped to 0.3–2.0 mm;
4. row count is `ceil(height / density)`;
5. for each row, intersect the horizontal line with every polygon edge;
6. sort the intersections and pair them;
7. walk each pair at `stitch_len` spacing (minimum 0.8 mm);
8. reverse direction on alternate rows for a serpentine path;
9. each row starts with a `Move`.

Only rectangle and ellipse fills are created by the tools; there is no freehand
filled-path tool, although the model can represent paths.

### Transformations

`transform(o, pts)` applies the object’s rotation and translation to local
points. Rectangles and ellipses are generated in local space and rotated; paths
are stored in world space, so the generators map them into local space and back
before transforming. Moving an object updates both its centre and every stored
path point.

### Statistics and estimated time

`model::stats(&Project)`:

- counts only `El::Stitch` entries as stitches;
- collects the distinct thread indices used → `colors`;
- computes visible-object bounds → width and height;
- estimates time as:

```text
minutes = stitch_count / (stitches_per_second × 60)
```

Moves, colour changes, machine acceleration, trims, and thread-change dwell
time are not included. The estimate changes immediately when the rate changes.

### The flower example

`flower_example()` constructs, in order:

1. 8 outer fill petals, alternating Rose and Orchid, each rotated around the
   centre and edged with a deeper Wine run stitch;
2. 8 shorter inner fill petals in Blush;
3. one satin ring in Amber around the centre;
4. one gold fill centre;
5. `fit_square(&mut p, 80.0)`, which recentres the design and scales the whole
   composition so its outer bounds are exactly 80 × 80 mm.

That is **26 objects**. The development screenshot shows 4,782 stitches and 6
distinct thread colours. Tests verify that both the object bounds and the
actual stitch points reach 40 mm from the centre, not merely the outlines.

`demo_project()` returns the same flower; the app names it with the translated
`name.demo` label (for example, “Flower · 8 × 8 cm”).

### History

Undo and redo store clones of `project.objects` only, with a maximum of 60 undo
entries. They do not restore hoop size, speed, project name, language, or other
UI state. A drag or slider gesture pushes one undo entry at the start of the
gesture rather than one per frame.

---

## 7. File formats and I/O

### Project JSON

`save_project` writes pretty-printed JSON. `load_project` reads text and lets
Serde parse it; a parse error becomes a translated “that file is not a design”
message.

Example shape:

```json
{
  "version": 2,
  "name": "Flower",
  "hoop": 22,
  "hoop_w": 140.0,
  "hoop_h": 200.0,
  "rate": 12.5,
  "objects": [
    {
      "id": 1,
      "kind": "Fill",
      "shape": "Ellipse",
      "cx": 0.0,
      "cy": 23.5,
      "w": 16.0,
      "h": 33.0,
      "rot": 1.57,
      "path": [],
      "color": 8,
      "visible": true,
      "stitch_len": 2.5,
      "density": 0.6,
      "satin_w": 5.0
    }
  ],
  "next_id": 2
}
```

Compatibility rules:

- missing `hoop_w`, `hoop_h`, or `rate` → Serde default 0 → preset/default
  fallback;
- version 1 uses `LEGACY_HOOPS` for the `hoop` index;
- current version is 2;
- unknown JSON fields are accepted by Serde’s default behaviour;
- there is no schema validation beyond deserialisation.

### DST files

DST is the universal three-byte stitch format. The implementation follows the
behaviour read from `IMPORT.DLL`; see `MECHANICS.md` for the decompiled
function addresses.

#### Record structure

Each record is exactly 3 bytes:

```text
byte 0: x small (1/9), y small (1/9)
byte 1: x medium (3/27), y medium (3/27)
byte 2: x large (81), y large (81), kind bits
```

The delta is in 0.1 mm units. The maximum per-axis movement in one record is
121 units (12.1 mm) because the encoder decomposes into 81 + 27 + 9 + 3 + 1.

`dst_encode(x, y, flags)`:

| Flag | Meaning | Bytes |
|---:|---|---|
| 1 | ordinary stitch | data bits with low bits `0b11` |
| 2 | jump | data bits plus `0x80`, low bits `0b11` |
| 3 | colour change | `b2 = 0b1100_0011` (`0xC3`) |
| 4 | end | `b2 = 0b1111_0011` (`0xF3`) |

Bit meanings inside a data record:

- x: `0x01` +1, `0x02` −1, `0x04` +9, `0x08` −9;
- x: `0x01`/`0x02` in byte 1 are +3/−3, `0x04`/`0x08` are +27/−27;
- x: byte 2 `0x04`/`0x08` are +81/−81;
- y in byte 0: `0x80` +1, `0x40` −1, `0x20` +9, `0x10` −9;
- y in byte 1: `0x80` +3, `0x40` −3, `0x20` +27, `0x10` −27;
- y in byte 2: `0x20` +81, `0x10` −81.

#### Export

`export_dst`:

1. builds the timeline;
2. converts each point to integer 0.1 mm;
3. tracks min/max coordinates for the header;
4. inserts a colour-change record whenever the colour changes after the first;
5. splits long moves into repeated records clamped to ±121 units;
6. writes a 512-byte header padded with spaces and an `0x1A` terminator;
7. appends three `00 00 F3` end records.

The header contains `LA:`, `ST:`, `CO:`, `+X:`, `-X:`, `+Y:`, `-Y:`, `AX:`,
`AY:`, `MX:`, `MY:`, and `PD:` lines. The design name is truncated to 16
characters.

#### Import

`import_dst`:

- requires at least 515 bytes (512-byte header + one record);
- starts reading at offset 512;
- stops **only** on the exact record `00 00 F3`;
- stops if any of the three bytes is `0x7F`, because that is the decoder
  table’s stop marker and the original produces no partial stitch;
- decodes the delta before inspecting the record kind;
- treats `rec[2] & 0xC0` as:
  - `0x40` or `0xC0` → colour change: end the current chain and start a new one;
  - `0x80` → jump: end the current chain and start a new one;
  - anything else → ordinary stitch point;
- keeps chains with at least two points as objects;
- assigns colours cyclically from a 12-entry DST palette because DST carries no
  colour values;
- names the project from the file stem;
- starts at the largest preset, then, if the imported bounds fit a smaller
  preset, applies `best_preset_for`;
- returns an error if fewer than two points were found.

DST coordinates are taken as absolute millimetres relative to the DST origin;
the importer does not rescale or centre the design beyond choosing a hoop.

### SVG export

`export_svg` writes an SVG sized in millimetres:

- `width`/`height` are the hoop size plus `mm` units;
- `viewBox` is `0 0 w h`;
- a fabric-coloured background and a border are drawn;
- model y is flipped (`oy - y`);
- timeline steps are grouped into paths by `(colour, object)`;
- a new path starts at a `Move` (`M`) and uses `L` for stitches and moves;
- stroke is 0.45 mm, round caps, no fill;
- hidden objects are excluded through `timeline`.

### CSV export

`export_csv` writes:

```text
index,x_mm,y_mm,type,thread
1,-12.34,5.67,jump,Rose
2,-11.90,5.67,stitch,Rose
```

`type` is `jump` for `El::Move` and `stitch` for `El::Stitch`. The thread name
comes from the current language file.

### In-app file browser

`io::FileDialog` replaces the system dialog with an `egui` window:

- starts in `$HOME/Documents` when it exists, otherwise `$HOME`, otherwise `.`;
- Up button, current path, and a description of the active filter;
- shows directories and files matching the requested extension;
- hides dotfiles;
- single-click selects; double-click opens a file;
- name field; the correct extension is appended for saving if missing;
- Save/Open and Cancel buttons;
- an empty-folder message and an error line;
- no overwrite confirmation and no “all files” filter.

The dialog is anchored at the centre of the window and remembers its directory
for the rest of the session.

---

## 8. User interface

### Window layout

```text
┌────────────────────────────────────────────────────────────────────┐
│ logo  Stitch Studio   File Edit View Stitch Help      Language [en]│
│ toolbar: tools | undo redo | fit grid rulers stitches | zoom | fit │
├──────────────┬──────────────────────────────────┬──────────────────┤
│ DESIGN       │                                  │ Threads Settings │
│ name         │                                  │        Plan      │
│ hoop preset  │            canvas                │                  │
│ width/height │       fabric + hoop rulers       │  tab content     │
├──────────────┤                                  │                  │
│ SUMMARY      │                                  │                  │
│ size, cm     │                                  │                  │
│ stitches     │                                  │                  │
│ colours      │                                  │                  │
│ sew time     │                                  │                  │
│ rate         │                                  │                  │
├──────────────┤                                  │                  │
│ OBJECTS      │                                  │                  │
│ object list  │                                  │                  │
│ up/down      │                                  │                  │
├──────────────┴──────────────────────────────────┴──────────────────┤
│ ▶ Reset ─────────slider──────── 0 / N   speed 1× 10× Max 12.5 st/s │
├────────────────────────────────────────────────────────────────────┤
│ tool hint                     objects · hoop name · offline badge   │
└────────────────────────────────────────────────────────────────────┘
```

Fixed sizes: left panel 262 px, right panel 292 px, transport bar 54 px,
status bar 24 px, canvas margin 8 px.

### Top bar and menus

The top bar contains the hand-drawn logo, the product name and tagline, the
menu bar, and the language picker on the right.

| Menu | Items |
|---|---|
| **File** | New design (Ctrl+N), Flower example (Ctrl+E), Open design (Ctrl+O), Import stitch file (DST), Save (Ctrl+S), Save as, Export DST, Export SVG, Export CSV, Quit |
| **Edit** | Undo (Ctrl+Z), Redo (Ctrl+Y), Duplicate (Ctrl+D), Delete (Del), Centre in hoop |
| **View** | Show stitches, Show grid, Hoop rulers, Snap to 1 mm, Fit to hoop (F) |
| **Stitch** | Play/Pause simulation, Reset simulation, Reduce colour changes |
| **Help** | How this copy works, How it is built |

The View checkboxes are session state, not saved to disk. “Reduce colour
changes” sorts objects by thread colour (`sort_by_key(o.color)`), which groups
same-colour objects together and reduces the number of machine colour changes.

The language picker lists the three built-in languages plus any extra JSON
files found in the first language directory. It shows `name !` in warning
colour when the current file could not be parsed, with the error on hover.
Switching language updates the window title and writes the choice to the
settings file.

### Toolbar

| Control | Action |
|---|---|
| Select / move (V) | Pick and drag objects |
| Filled rectangle (R) | Drag or click to create a fill rectangle |
| Filled ellipse (E) | Drag or click to create a fill ellipse |
| Run stitch line (L) | Drag or click to create a run path |
| Satin column (S) | Drag or click to create a satin path |
| Pan (H) | Drag the workspace |
| Undo / Redo | Same as Edit menu; disabled when the stack is empty |
| Fit | Fit the hoop to the canvas |
| Grid | Toggle 1 mm / 5 mm / 10 mm grid overlay |
| Rulers | Toggle the four-edge millimetre rulers |
| Stitch preview | Toggle between stitch lines and filled shape previews |
| − / + | Zoom out/in |
| Fit status | “Design fits inside the hoop” or “Design extends past the hoop” |

The zoom percentage is `zoom / 3.78 × 100`, a display convention rather than a
physical 1:1 mapping.

### Left panel

**Design card**

- Editable project name.
- Hoop preset combo box: all 39 presets plus “Custom size”.
- Width and height drag-value fields, in millimetres, range 20–600, step 0.5.
- Typing a size updates the fabric, rulers, fit warning, and saved design.
- A preset selection fills both fields and fits the view when it changes the
  size.

**Summary card**

| Row | Value |
|---|---|
| Size | `width × height mm` |
| Size in cm | `width × height cm` |
| Stitches | Stitch count with space thousands separators |
| Colours | Number of distinct thread indices used |
| Sew time | `m:ss`, or `h:mm:ss` above one hour |
| Stitches per second | Editable rate, range 0.5–60, step 0.1 |

Changing the rate recalculates the time and rewrites the project’s rate.

**Object list**

- Header with object count and Duplicate/Delete actions.
- Rows show a colour swatch, stitch kind + shape, size, and thread name.
- An eye control toggles visibility.
- Clicking a row selects the object.
- Move up/down buttons reorder the selected object.
- Empty state offers a “Flower example” button.
- The list is scrollable and height-limited.

### Right panel tabs

**Threads**

- Shows the selected object’s colour name and swatch.
- A 30-swatch grid; hovering shows the translated colour name.
- Clicking a swatch changes the selected object’s colour; with no selection it
  shows “pick an object first”.
- “Stitch plan” lists contiguous colour runs in timeline order with
  timeline-step counts. Unlike the Plan tab, this counter includes `Move`
  entries as well as stitches.

**Settings**

- Only enabled for a selection.
- Stitch type segmented buttons: Run, Satin, Fill (hidden for imported
  `Stitches` objects).
- Placement sliders:
  - rotation −3.14…3.14 rad;
  - across −130…130 mm;
  - up −190…190 mm;
  - width and height 3…200 mm (only for non-path objects).
- Stitch sliders:
  - spacing 0.3…1.6 mm;
  - stitch length 1.0…5.0 mm;
  - satin column width 1.5…15.0 mm when the kind is Satin.
- Centre and Delete buttons.
- Imported stitch objects show a message instead of the sliders: every stitch
  stays where the file put it; the object can still be dragged, recoloured,
  reordered, hidden, or deleted.

**Plan**

- Machine sequence summary: stitches, colour blocks, estimated time, hoop
  size, rate.
- “Colour order” list of contiguous colour runs with stitch counts.
- “Reduce colour changes” button.

> The Plan summary’s “Colour blocks” value comes from `stats.colors`, which
> counts **distinct thread indices**, while the list below counts contiguous
> runs. They can differ when a colour is used in more than one run.

### Canvas

- Fabric is drawn as a cream rectangle with a soft shadow and hoop band.
- Grid is drawn at 5 mm or 10 mm spacing depending on zoom, with stronger
  lines every 50 mm and dashed centre axes.
- Stitches are drawn as polylines per object; the selected object gets a wider
  translucent glow.
- With stitch preview off, objects are drawn as translucent filled polygons.
- The simulation overlay draws completed stitches in yellow with a needle
  marker and crosshair.
- The selection is a dashed rectangle with four corner handles.
- If the visible design extends past the hoop, a warning badge is drawn on the
  fabric and the toolbar says it overflows.
- An empty design shows “Start a design” and a hint.
- Hovering over the canvas zooms around the pointer with the scroll wheel,
  clamped to 0.5–40×.
- Middle-button or secondary-button drag pans. The Pan tool also pans with
  primary drag.
- In Select mode, a click picks the topmost visible object under the pointer,
  or clears the selection. Dragging moves the selected object, snapping to
  whole millimetres when Snap is on.
- In a creation mode, dragging previews the shape; releasing creates it. A
  plain click creates a default-sized shape:
  - rectangle: 24 × 24 mm;
  - ellipse: 26 × 22 mm;
  - run/satin path: a 32 mm horizontal line.
- A rectangle/ellipse drag under 3 mm in either direction falls back to a
  24 × 24 mm shape centred on the start point.
- After creating an object, the tool returns to Select and the new object is
  selected.

### Rulers

The ruler is drawn in a band around all four fabric edges:

- one tick per millimetre at zoom ≥ 3;
- one tick every 5 mm at zoom ≥ 1.2;
- one tick every 10 mm below that;
- labels every 10, 20, or 50 mm depending on available pixel spacing;
- tick lengths: 9.5 px every 10 mm, 6.5 px every 5 mm, 3.5 px otherwise;
- alpha: 170 / 110 / 62 respectively;
- the zero line is accent-coloured;
- an `mm` badge sits outside the top-left corner;
- when rulers are off, the two hoop dimension labels are drawn instead.

### Bottom bars

**Transport**

- Circular play/pause button (restarts from 0 if the simulation has reached
  the end).
- Reset button.
- Timeline slider and `current / total` counter.
- Simulation-speed buttons: `1×`, `10×`, `Max` (80×).
- Stitches-per-second field.

The simulation advances by:

```text
per_frame = stitches_per_second × multiplier × dt
```

and consumes whole stitches, so the preview is driven by the same rate as the
estimated time. `Max` is 80× the machine rate, described by the UI as up to
60,000 stitches/minute.

**Status**

- Current tool hint.
- Object count.
- Hoop name or custom size.
- “No USB key · never online” badge with a tooltip.

### Dialogs and windows

- **File browser** — described in section 7.
- **How this copy works** — scrollable help: no USB key, no network, tools,
  rulers, files, build, languages. Buttons: Close and More about the build.
- **How it is built** — technology summary (Rust, egui/eframe, OpenGL through
  eframe, project/DST/SVG/CSV, no network) and four explanatory paragraphs.
- Both help/about windows are normal egui windows with the usual window X and
  are not application-modal. The Help window also has Close and “More about the
  build” buttons.
- Toasts appear at the bottom centre of the canvas for about 2.6 seconds.

### Keyboard shortcuts

Shortcuts are ignored while a text field wants keyboard input.

| Shortcut | Action |
|---|---|
| `Ctrl+N` | New design |
| `Ctrl+O` | Open design |
| `Ctrl+S` | Save (opens the save dialog) |
| `Ctrl+E` | Flower example |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+D` | Duplicate selected object |
| `Delete` | Delete selected object |
| `F` | Fit hoop to view |
| `Space` | Play/pause simulation |
| `V` | Select tool |
| `R` | Filled rectangle tool |
| `E` | Filled ellipse tool |
| `L` | Run path tool |
| `S` | Satin path tool |
| `H` | Pan tool |

---

## 9. Internationalisation

### Files

All variable interface text is read from a flat JSON object. A few brand,
technical, and unit strings (for example the product name, `mm`, `Rust`, and
the zoom signs) are hard-coded.

```json
{
  "_name": "English",
  "menu.file": "File",
  "objects.header": "Objects · {n}",
  "thread.0": "Ivory"
}
```

All three shipped files contain 239 keys. `_name` is the name shown in the
language picker. Thread names are `thread.0` through `thread.29`.

### Placeholders

The program replaces these tokens at runtime:

| Token | Used for |
|---|---|
| `{n}` | Object/stitch counts |
| `{path}` | File path in an error |
| `{e}` | Error detail |
| `{name}` | Saved file name |
| `{what}` | Lower-cased file-type description |

Translations may move placeholders within a sentence. The test suite checks
that each translated key has the same number of `{` characters as English, not
the exact placeholder names.

### Lookup order for the language directory

`lang_dir()` builds candidates in this order and uses the **first existing
directory**:

1. `$STITCH_LANG_DIR`;
2. `./lang` relative to the current working directory;
3. `lang` next to the executable and up to four parent directories;
4. `app/lang` next to the compiled source (`CARGO_MANIFEST_DIR/lang`).

Within that directory, `<code>.json` is loaded. The three shipped languages
are also compiled into the binary with `include_str!`.

Loading is a key-by-key overlay:

```text
start with the embedded language map
if <code>.json exists on disk, overlay every key it contains
unknown key → English value → key itself
```

A malformed file leaves the built-in text in place and records an error; the
picker shows `name !` and the error is available on hover. A partially
translated file works, because missing keys fall through to the embedded copy.

### Choosing and saving a language

Priority at startup:

1. `--lang <code>` / `--lang=<code>` if that language exists;
2. saved language in the settings file, if it exists;
3. English.

Changing the picker immediately calls `lang::set(code)` and
`lang::save_language(code)`, then updates the window title. Extra `.json` files
in the language directory are discovered once at startup; adding one while the
program is running requires a restart.

To add a language, copy `app/lang/en.json` to `xx.json`, translate the values,
and set `_name`. See `app/TRANSLATING.md` for the user-facing guide.

### Key groups

| Prefix | Keys | Covers |
|---|---:|---|
| `_name` | 1 | Language name |
| `app` | 3 | Product title and tagline |
| `menu` | 32 | Menus and menu items |
| `tool` | 12 | Tool names and hints |
| `toolbar` | 10 | Tooltips and status labels |
| `panel` | 5 | Section headings |
| `hoop` | 5 | Hoop preset/width/height/tip |
| `summary` | 7 | Summary card |
| `plan` | 7 | Plan tab |
| `settings` | 3 | Settings tab messages |
| `prop` | 8 | Property sliders |
| `sim` | 8 | Simulation controls |
| `status` | 3 | Bottom status bar |
| `thread` | 33 | 30 colour names + colour UI |
| `objects` | 8 | Object list |
| `dialog` | 9 | File browser |
| `err` | 5 | Error messages |
| `msg` | 7 | Toast messages |
| `title` | 6 | Dialog titles |
| `filter` | 5 | File filters |
| `shape` | 4 | Shape words |
| `stitch` | 4 | Stitch kind labels |
| `help` | 23 | Help window |
| `about` | 17 | About window |
| `canvas` | 3 | Empty/overflow canvas text |
| `name` | 3 | Design names |
| `action` | 2 | Centre/delete buttons |
| `common` | 1 | No-stitches message |
| `tip` | 1 | Satin tip |
| `unit` | 1 | Stitches-per-second unit |

`sim.spm` exists in the files but is not referenced by the current source.

---

## 10. Theme and rendering

### Palette

All colours are defined in `theme.rs`.

| Constant | RGB | Hex | Use |
|---|---|---|---|
| `BG` | 19, 22, 27 | `#13161b` | Canvas background |
| `PANEL` | 26, 30, 37 | `#1a1e25` | Panels |
| `PANEL2` | 33, 38, 46 | `#21262e` | Cards, inactive controls |
| `FIELD` | 22, 25, 31 | `#16191f` | Input fields, lists |
| `LINE` | 46, 52, 62 | `#2e343e` | Default borders |
| `LINE_SOFT` | 38, 43, 52 | `#262b34` | Soft borders |
| `TEXT` | 223, 228, 234 | `#dfe4ea` | Primary text |
| `MUTED` | 138, 148, 162 | `#8a94a2` | Secondary text |
| `DIM` | 95, 105, 119 | `#5f6977` | Tertiary text |
| `ACCENT` | 77, 195, 166 | `#4dc3a6` | Selection, active controls |
| `ACCENT_DIM` | 44, 109, 95 | `#2c6d5f` | Selected background |
| `BLUE` | 99, 168, 224 | `#63a8e0` | Links, directories |
| `OK` | 111, 207, 151 | `#6fcf97` | Fits status |
| `WARN` | 226, 176, 92 | `#e2b05c` | Overflow/warning |
| `DANGER` | 224, 118, 109 | `#e0766d` | Delete/error |
| `FABRIC` | 233, 228, 216 | `#e9e4d8` | Fabric |
| `FABRIC_EDGE` | 201, 194, 176 | `#c9c2b0` | Fabric border |

`theme::apply` installs a dark `egui` visual style: 6 px widget rounding,
1 px borders, 7 × 6 px item spacing, 9 × 5 px button padding, 130 px sliders,
and text styles from 11.5 to 18 px. Window shadows are 26 px blurred black.

### Icons and widgets

`theme::draw_icon` draws every toolbar icon by hand with lines, circles, and
polygons: select cursor, rectangle, ellipse, path, satin zigzag, pan cross,
fit frame, grid, ruler, needle, key, shield, save, open, plus, undo, redo, and
trash. `icon_button` handles selected, hovered, disabled, and tooltip states.

`card`, `section`, `kv`, `hint`, `chip`, and `segmented` provide the repeated
layout pieces. `chip` is currently marked `#[allow(dead_code)]`.

### Fonts

`install_fonts` tries, in order:

1. `/usr/share/fonts/noto/NotoSans-Regular.ttf`
2. `/usr/share/fonts/TTF/DejaVuSans.ttf`
3. `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`

The first readable file becomes the proportional UI font. If none exists,
egui’s bundled default font is used. The program does not ship a font file.

### Canvas painting order

1. Background fill.
2. Fabric shadow and fabric rectangle.
3. Fabric border.
4. Grid and centre axes when enabled.
5. Hoop band.
6. Millimetre rulers when enabled.
7. Objects, clipped to the fabric plus 4 mm:
   - stitch polylines, or filled polygons when stitch preview is off;
   - selected-object glow before the normal stroke.
8. Simulation overlay and needle.
9. Selection dashed frame and corner handles.
10. Hoop dimension labels when rulers are off.
11. Overflow warning badge.
12. Empty-state text.
13. Toast (drawn by `canvas` after `paint_canvas`).

### Screenshot rendering

The `--screenshot` path uses egui’s own viewport screenshot event, not a
separate headless renderer. It needs a working `eframe` window/backend. The
image is saved as RGBA8 PNG at the actual window size, normally 1280 × 820.

---

## 11. Tests and build checks

### Unit/UI tests

Run with:

```sh
cd app
cargo test
```

The current result is:

```text
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

| Module | Tests | Coverage |
|---|---:|---|
| `model` | 10 | Flower size/stitches, demo fit, fill density, satin zigzag, run outline, rotation bounds, free hoop sizes, best-fit hoop and tie-break, time follows rate, legacy files |
| `io` | 7 | Project JSON round trip, DST export plan, DST import round trip, `0x7F` stop, `0x40` colour change, exact end record, SVG/CSV export |
| `lang` | 2 | All languages have English keys/placeholders; switching languages |
| `main` | 8 | Example menu, ruler density, time formatting, rate changes, rendered hoop/rate fields, rendered time updates, odd hoop size, flower name in all languages |

Notable test techniques:

- `rendered(app)` runs `egui::Context::run` three times at 1280 × 820 and walks
  the shape list to collect every text string and its rectangle. This verifies
  that labels are actually drawn, not merely present in the language files.
- The flower test checks both object bounds and actual stitch coordinates,
  ensuring the sewn design is 80 × 80 mm.
- DST tests decode the exported file independently and compare the visited
  points with the planned timeline.
- The import tests construct minimal DST files with specific markers.

The tests do not cover pointer drags, the file dialog, window-manager
behaviour, or pixel-level appearance.

### Historical checks

`craft.json` records nine rounds. A check round has up to two checks: `build`
and `app-screenshot`.

| Round | Build | Screenshot | Note |
|---:|---|---|---|
| 1 | Failed | — | Cargo not installed or not on PATH in the check environment |
| 2 | Passed | Passed | First successful build and 1280 × 820 screenshot |
| 3 | Passed | Passed | After a follow-up request |
| 4 | Passed | Passed | After removing status messages, adding simulation speeds |
| 5 | Passed | Passed | After removing the offline guard and adding translations |
| 6 | Failed | — | Linker `cc` not found under the stripped PATH |
| 7 | Passed | Passed | After pinning `/usr/bin/cc` in the Cargo config |
| 8 | Passed | Passed | After adding millimetre rulers |
| 9 | Passed | Passed | After editable hoop size and stitches/second |

Final project note: “Ready: all 2 checks pass.”

The screenshot artifacts are 1280 × 820 RGBA PNGs. `build-activity.log` and
`feed.json` preserve the same history in more detail, including repeated
Hermes startup failures caused by a read-only install lock in the build
environment.

---

## 12. Provenance and reverse engineering

`app/MECHANICS.md` is the primary dossier. It documents a read-only analysis
of the installed Janome DigitizerPro 3.0 files, performed with Ghidra 12.1.4
headless and with the REA evidence tools.

### Original artifacts

| File | SHA-256 | Notes |
|---|---|---|
| `JANOME.EXE` | `cab5d0d28c0eafeb9ecbdb15997dc913eda3c9f51784ef283e1aa2c51554cde8` | 7,639,040 bytes, PE32, linker 7.0, image base 0x400000, entry RVA 0x6fe280 (VA 0xafe280), timestamp 2009-11-09 10:37:50 UTC |
| `RESOURCE.DLL` | `8078d02899c433d416153d708f4945a07a3a71741efd0585f5d78c7893f66739` | Hoop names, labels, resource strings |
| `MOD1.DLL` | `25988c9b9b7d18fac7fa11b3bee93f8da47413940d272f6a78003949c65bbd81` | Hoop library, stitch criteria, fit rule |
| `IMPORT.DLL` | `faabe3e689f5f07c59eb283be2e797db4c7b233bb2b4888691641fe9c4320eb3` | DST reader |
| `DEFMGR.DLL` | `7e4898b8aa1a6e862b839867faa92569b09322965fc990953e6c65a38e0eae78` | Defaults accessor, statically links HASP |

The main executable is packed and encrypted:

- `.text` has entropy 8.0000 and is encrypted program code;
- `.rdata`, `.data`, and `.protect` are also near 8.0 entropy;
- the entry point lands in `.protect`, which decrypts code and calls the real
  entry;
- the HASP dongle check lives inside the encrypted stub and cannot be read or
  patched from the file;
- string searches find no readable `Hoop`, `Janome`, `JANOME`, `stitch`, `DST`,
  `Update`, or `http://` text in the executable.

This is why the project is a rebuild rather than a patched binary: there is no
plain “is the key present” function to remove.

### Functions that informed the rebuild

| File | Function / address | Finding used |
|---|---|---|
| `RESOURCE.DLL` | `GetHoopDisplayName` 0x100027c0 | Hoop display-name lookup via property 0x9ca |
| `RESOURCE.DLL` | `FUN_10002640` | Bounds-checked display/internal name tables |
| `RESOURCE.DLL` | `FUN_100024c0` | 45 string slots of 0x104 bytes; fills name tables |
| `RESOURCE.DLL` | `GetHoopInternalName` 0x10002700 | Internal-name lookup |
| `MOD1.DLL` | `GetBestFittingHoop` 0x1009ba60 | Strict fit, allowance on both sides, smallest area, earlier hoop wins ties |
| `MOD1.DLL` | `GetMatchingHoop` 0x1009b810 | Tolerance match, not the fit rule |
| `MOD1.DLL` | `CreateUserDefinedHoopPrototype` 0x1009bd90 | Confirms properties 0x9cc/0x9cd are hoop sides |
| `MOD1.DLL` | `GetDesignStitchCriteria` 0x10018390 | Design criteria from machine property 0x99 |
| `MOD1.DLL` | `GetObjectStitchCriteria` 0x10018400 | Object criteria + tufting/chenille flags |
| `MOD1.DLL` | `FillStyle::GetStitchCriteria` 0x101dd4a0 | Spacing fields at +0x10/+0x14/+0x18 |
| `MOD1.DLL` | `StitchGenerationCriteria` 0x103e1f80 | Min of non-zero slots, max of doubles, OR of flags |
| `MOD1.DLL` | `FixLargeStitchLength` 0x1022a3b0 | Writes `INT_MAX` to +0x90 |
| `IMPORT.DLL` | `DSTStitchFormat::LoadStitch` 0x1002ddc0 | 3-byte record, exact `00 00 F3` end, `0x40/0x80/0xC0` branches |
| `IMPORT.DLL` | `FUN_1002dba0` | Three 256-entry `(dx,dy)` tables; `0x7F` stops decode |
| `DEFMGR.DLL` | `GetGlobalDefaultData` / `PutGlobalDefaultData` 0x10002210 / 0x10002230 | Thin accessors around HASP-protected defaults |

### Findings reflected in this code

- **Hoop catalogue.** 38 hoops come from `RES/*.DSC` lines; M4 exists only in
  the 39-name display table.
- **Hoop shapes.** The seven `D..` hoops are stored as `HOOP RECTANGLE (d,d)`,
  so the rebuild offers them as squares.
- **Multi-position hoops.** Hoop MA and Hoop D are `HOOP 3_POSITION` entries;
  the rebuild uses their declared overall size.
- **Fit rule.** Strict `<` on both sides, allowance added to both, smallest
  area, earlier library entry on ties. Allowance is set to 0 because the caller
  is inside the encrypted executable.
- **DST.** The rebuild reproduces the exact end record, the `0x7F` stop, the
  record-kind switch, and the ±121-unit decomposition.
- **Defaults.** `DEFAULTS.DFL` is HASP-encrypted and cannot be read without the
  dongle. The rebuild uses its own 2.5 mm run, 0.4 mm satin, and 0.45 mm fill
  spacing and does not claim they are Janome’s values.
- **Speed unit.** The original labels speed as stitches per second (`s/s`), not
  per minute. No readable default speed exists; 12.5 s/s is a reconstruction.
- **Network endpoints.** The original registry records
  `http://packagedfilestorage.com/download/janome` and
  `http://www.janome.co.jp/Dealers/DealerUpdate`. The rebuild contains no
  network code and contacts neither.

### Evidence archive

`investigation.cany/` contains the audit trail:

| Item | Count / detail |
|---|---|
| Evidence records | 123 JSON files under two-hex shard directories |
| Subjects | `MOD1.DLL` 43, `RESOURCE.DLL` 33, `DEFMGR.DLL` 16, `JANOME.EXE` 15, `IMPORT.DLL` 10, `REVERT.EXE` 5, `DEFMERGE.EXE` 1 |
| Operations | 53 string searches, 38 function searches, 22 pseudocode reads, 3 address views, 2 library-call lists, 2 xref lists, 1 functions list, 1 section list, 1 callers list |
| Confidence | 59 observed, 64 derived |
| Journal | 612 `tool_call` entries, 5 recorded failures |
| Annotations | 7 files, all empty name/comment/bookmark maps |
| Unknowns | Empty directory |

The five journal failures are:

1. two attempts to ask a build question blocked by the clean-room policy;
2. one library-call listing blocked by the same policy;
3. two pseudocode requests for addresses that had no function
   (`0x100089c0`, `0x10007104`).

`investigation.cany/project.toml` limits the project to no tools and records
the purpose as `own_or_authorized`.

### The two provenance notes

The repository contains two statements that should not be confused:

- `capture/README.md` is the **initial clean-room instruction**. It says
  nothing inside `JANOME.EXE` was read and that the build should use only the
  request and reference images.
- `app/MECHANICS.md` and `investigation.cany/` record **read-only direct
  analysis of the installed DLLs**, while the in-app `about.p4` text still says
  the original “was never opened”.

For engineering purposes, the mechanics and evidence archive are authoritative
for the hoop catalogue, fit rule, DST behaviour, and resource strings. The
About text is a user-facing claim that predates or omits the later read-only
analysis. This discrepancy is documented here rather than silently repeated.

---

## 13. Development history

`feed.json` (version 5, 98 items) and `build-activity.log` record the build
sessions. The user-visible sequence was:

1. **Initial build.** Create a desktop embroidery workspace that works without
   the USB dongle and contains no network code. The first check failed because
   the check environment could not find `cargo`; restoring the Cargo bin
   directory fixed it, and round 2 passed.
2. **Technology question.** The user asked what technology was used; the
   answer described Rust + egui/eframe and the absence of a network client.
   Round 3 passed.
3. **UI request.** Remove the “OFFLINE” and “NO USB KEY” messages, allow faster
   simulation, and remove the Help window’s Close button while keeping the
   window X. Round 4 passed.
4. **Remove the offline guard and add translations.** The guard panel was
   removed and all text moved into `lang/*.json` with English, Spanish, and
   Basque. Round 5 passed.
5. **Flower example.** Build an exactly 8 × 8 cm flower motif. Round 6 failed
   because the stripped `PATH` could not find `cc`; Cargo config files pinned
   `/usr/bin/cc`, and round 7 passed.
6. **Millimetre rulers.** Add a mark every millimetre around the hoop, longer
   marks every five, numbers every ten, and zero at the centre. Round 8 passed.
7. **Free hoop size and sewing speed.** Make hoop width/height freely editable
   and make stitches per second editable so the estimated time is exact.
   Round 9 passed.

Later screenshots (`round-10.png`, `round-10-es.png`, `round-11.png`,
`round-11-es.png`) exist outside the recorded check list, including Spanish
variants.

The repeated early errors in `feed.json` and `build-activity.log` are Hermes
runtime failures (`Read-only file system` on an install lock), not application
build errors.

---

## 14. Extending the program

### Add or change a hoop preset

1. Add the tuple to `HOOPS` in `app/src/model.rs`.
2. Update the array length `39` in the type.
3. Add the matching entry to `LIBRARY_RANK`, using the hoop’s position in the
   original display table. The two arrays must stay aligned.
4. If the default hoop changes, update `DEFAULT_HOOP`.
5. Update the `MECHANICS.md` hoop table if the provenance should remain
   complete.

### Add a language

1. Copy `app/lang/en.json` to `app/lang/xx.json`.
2. Set `_name` and translate the values.
3. Keep placeholders and key names unchanged.
4. Restart the program. The picker discovers the new file at startup.
5. Run `cargo test`; the key/placeholder consistency test runs over the three
   built-in languages only, so consider adding `xx` to `EMBEDDED` and `ORDER`
   in `lang.rs` if it should be compiled in.

### Add a stitch kind

1. Add a variant to `StitchKind`.
2. Add its `key()` and language entries (`stitch.<key>`).
3. Add a generator function and dispatch it in `object_stitches`.
4. Add UI buttons in `settings_tab`.
5. Add export/import handling if the format needs it.
6. Add model tests.

### Add an export format

1. Write a function in `io.rs` using `model::timeline`.
2. Add a menu item and a save-dialog branch in `handle_file_pick`.
3. Add a filter label to all language files.
4. Add a test that writes to a temporary directory and inspects the output.

### Change defaults

Spacing and speed defaults are in `Obj::new_rect`/`new_path`, `Project::default`,
and `DEFAULT_RATE_SPS`. Update `MECHANICS.md` if the provenance claim changes.

---

## 15. Known limitations

1. **Visual reconstruction.** There were no reference screenshots of the
   original application. The layout, dark palette, icons, and sample flower
   are an interpretation, not a faithful visual copy.
2. **Reconstructed sewing defaults.** Run/satin/fill spacing and the 12.5 s/s
   speed are not Janome’s values; the original defaults are HASP-encrypted.
3. **Zero fit allowance.** The original’s allowance argument is inside the
   encrypted executable; the rebuild uses 0 mm.
4. **Multi-position hoops.** Hoop MA and Hoop D are represented by their
   declared overall rectangles, not by individual positions or repositioning
   logic.
5. **DST is not hardware-verified.** Export is validated by decoding it back,
   but it has not been sewn on a real machine. The writer emits three end
   records as a machine convention; the original reader accepts the first
   exact end record.
6. **DST import is raw geometry.** Imported files keep exact stitch points but
   lose thread colours, machine commands, trims, and other metadata. Colours
   are assigned from a fixed 12-entry palette.
7. **Simplified satin.** The generator zigzags across the outline rather than
   producing a true offset-rail satin column.
8. **Rectangle hit-testing is elliptical.** Both rectangles and ellipses use
   the same normalised ellipse test.
9. **Undo is partial.** Only object lists are stored (maximum 60). Hoop size,
   rate, project name, language, and view state are not undoable.
10. **No unsaved-changes prompt.** `dirty` is tracked but never consulted when
    quitting, opening, or creating a new design.
11. **Save always opens a dialog.** There is no current-file path, so Ctrl+S
    behaves like Save As. Overwriting is not confirmed.
12. **Fixed property ranges.** Position sliders are fixed at −130…130 mm and
    −190…190 mm, and size sliders at 3…200 mm, even for larger custom hoops.
13. **Imported objects are read-only in Settings.** They can be moved,
    recoloured, reordered, hidden, and deleted, but the stitch settings sliders
    are hidden.
14. **Plan “blocks” vs list.** The summary counts distinct colours; the list
    counts contiguous colour runs.
15. **Language list is fixed at startup.** Extra language files require a
    restart to appear.
16. **Machine-specific build paths.** Both Cargo config files contain absolute
    paths that must be updated on another machine. `app/target/` may be stale
    because the config redirects output.
17. **No licence file.** The repository does not state a licence.
18. **Limited automated UI coverage.** Pointer gestures, the file dialog, and
    pixel appearance are not covered by `cargo test`; screenshots are generated
    by the external check harness.
19. **No multi-position hoop support, SVG import, or PDF export.**
20. **`about.p4` is historically inaccurate.** It says the original was never
    opened, while `MECHANICS.md` and the evidence archive document read-only
    analysis of its DLLs.

---

## 16. Appendices

### Appendix A: Hoop catalogue

The index is the `HOOPS` array index. “Library rank” is the position in the
original 39-name display table, used only for equal-area tie-breaks.

| # | Hoop | Width (mm) | Height (mm) | Library rank |
|---:|---|---:|---:|---:|
| 0 | Hoop C (50 x 50) | 50.0 | 50.0 | 17 |
| 1 | J5 No.3:MB Hoop (47 x 47) | 47.0 | 47.0 | 26 |
| 2 | J6 No.2:MB Hoop (66 x 66) | 66.0 | 66.0 | 27 |
| 3 | J3 No.13:MB Hoop (64 x 28) | 64.0 | 28.0 | 24 |
| 4 | J4 No.8:MB Hoop (42 x 67) | 42.0 | 67.0 | 25 |
| 5 | J2 No.6:MB Hoop (24 x 54) | 24.0 | 54.0 | 23 |
| 6 | S2:MB Hoop (28 x 50) | 28.0 | 50.0 | 32 |
| 7 | S1:MB Hoop (37 x 59) | 37.0 | 59.0 | 31 |
| 8 | M4:MB Hoop (100 x 60) | 100.0 | 60.0 | 13 |
| 9 | J7 No.1:MB Hoop (110 x 95) | 110.0 | 95.0 | 29 |
| 10 | Hoop No. 1 & 3 (126 x 90) | 126.0 | 90.0 | 19 |
| 11 | Hoop A (110 x 110) Ver.2.21 | 110.0 | 110.0 | 15 |
| 12 | H1:MB Hoop (100 x 90) | 100.0 | 90.0 | 20 |
| 13 | Hoop HH (100 x 90) | 100.0 | 90.0 | 7 |
| 14 | Hoop A(F) (126 x 110) Ver.3.0 | 126.0 | 110.0 | 14 |
| 15 | Hoop ST (126 x 110) | 126.0 | 110.0 | 0 |
| 16 | M2:MB Hoop (126 x 110) | 126.0 | 110.0 | 11 |
| 17 | J8 No.7:MB Hoop (120 x 121) | 120.0 | 121.0 | 30 |
| 18 | 200 Hoop (50 x 50) | 50.0 | 50.0 | 9 |
| 19 | 200 Hoop (140 x 140) | 140.0 | 140.0 | 8 |
| 20 | Hoop FA (50 x 50) | 50.0 | 50.0 | 3 |
| 21 | M3:MB Hoop (50 x 50) | 50.0 | 50.0 | 12 |
| 22 | Hoop B (140 x 200) | 140.0 | 200.0 | 16 |
| 23 | Hoop RE (140 x 200) | 140.0 | 200.0 | 1 |
| 24 | Hoop AQ (120 x 180) | 120.0 | 180.0 | 5 |
| 25 | T6:MB Hoop (78 x 168) | 78.0 | 168.0 | 38 |
| 26 | Hoop ASQ (180 x 180) | 180.0 | 180.0 | 6 |
| 27 | Hoop SQ (200 x 200) | 200.0 | 200.0 | 2 |
| 28 | Hoop D (220 x 190) (Giga Hoop) | 220.0 | 190.0 | 18 |
| 29 | Hoop MA (200 x 280) | 200.0 | 280.0 | 4 |
| 30 | M1:MB Hoop (240 x 200) | 240.0 | 200.0 | 10 |
| 31 | J6 No.9:MB Hoop (66 x 66) | 66.0 | 66.0 | 28 |
| 32 | T1:MB Hoop (D34) | 34.0 | 34.0 | 33 |
| 33 | J1 No.11:MB Hoop (D30) | 30.0 | 30.0 | 21 |
| 34 | J1 No.12:MB Hoop (D30) | 30.0 | 30.0 | 22 |
| 35 | T2:MB Hoop (D54) | 54.0 | 54.0 | 34 |
| 36 | T3:MB Hoop (D84) | 84.0 | 84.0 | 35 |
| 37 | T4:MB Hoop (D112) | 112.0 | 112.0 | 36 |
| 38 | T5:MB Hoop (D142) | 142.0 | 142.0 | 37 |

### Appendix B: Thread palette

| # | Name | RGB | Hex | Language key |
|---:|---|---|---|---|
| 0 | Ivory | 245, 240, 226 | `#f5f0e2` | `thread.0` |
| 1 | Cream | 233, 221, 196 | `#e9ddc4` | `thread.1` |
| 2 | Lemon | 246, 226, 127 | `#f6e27f` | `thread.2` |
| 3 | Gold | 228, 176, 66 | `#e4b042` | `thread.3` |
| 4 | Amber | 214, 138, 49 | `#d68a31` | `thread.4` |
| 5 | Coral | 232, 122, 100 | `#e87a64` | `thread.5` |
| 6 | Scarlet | 198, 54, 55 | `#c63637` | `thread.6` |
| 7 | Wine | 136, 40, 58 | `#88283a` | `thread.7` |
| 8 | Rose | 224, 132, 152 | `#e08498` | `thread.8` |
| 9 | Blush | 238, 186, 190 | `#eebabe` | `thread.9` |
| 10 | Orchid | 178, 114, 176 | `#b272b0` | `thread.10` |
| 11 | Plum | 110, 62, 120 | `#6e3e78` | `thread.11` |
| 12 | Lavender | 178, 168, 214 | `#b2a8d6` | `thread.12` |
| 13 | Periwinkle | 128, 142, 208 | `#808ed0` | `thread.13` |
| 14 | Indigo | 58, 70, 128 | `#3a4680` | `thread.14` |
| 15 | Sky | 124, 182, 222 | `#7cb6de` | `thread.15` |
| 16 | Teal | 58, 152, 156 | `#3a989c` | `thread.16` |
| 17 | Pine | 44, 102, 82 | `#2c6652` | `thread.17` |
| 18 | Moss | 104, 130, 68 | `#688244` | `thread.18` |
| 19 | Olive | 132, 126, 62 | `#847e3e` | `thread.19` |
| 20 | Sand | 214, 190, 152 | `#d6be98` | `thread.20` |
| 21 | Tan | 186, 150, 110 | `#ba966e` | `thread.21` |
| 22 | Cocoa | 110, 78, 58 | `#6e4e3a` | `thread.22` |
| 23 | Chocolate | 74, 52, 40 | `#4a3428` | `thread.23` |
| 24 | Silver | 190, 196, 202 | `#bec4ca` | `thread.24` |
| 25 | Slate | 104, 116, 128 | `#687480` | `thread.25` |
| 26 | Graphite | 62, 68, 78 | `#3e444e` | `thread.26` |
| 27 | Black | 28, 30, 34 | `#1c1e22` | `thread.27` |
| 28 | White | 252, 252, 250 | `#fcfcfa` | `thread.28` |
| 29 | Rosewood | 150, 84, 72 | `#965448` | `thread.29` |

### Appendix C: DST palette used by import

Imported DST blocks cycle through these thread indices:

```text
[6, 17, 3, 15, 11, 18, 4, 8, 14, 21, 25, 27]
```

That is Scarlet, Pine, Gold, Sky, Plum, Moss, Amber, Rose, Indigo, Tan, Slate,
Black.

### Appendix D: File-by-file index

| Path | Contents |
|---|---|
| `.gitignore` | Ignores `/target/`, `/app/target/`, and shim logs |
| `.cargo/config.toml` | Root build configuration |
| `app/Cargo.toml` | Package metadata and five runtime dependencies |
| `app/Cargo.lock` | Locked dependency graph |
| `app/.cargo/config.toml` | App-level build configuration |
| `app/.cargo/rustc-shim.sh` | Compiler wrapper |
| `app/README.md` | User-facing feature summary |
| `app/MECHANICS.md` | Reverse-engineering dossier and provenance tables |
| `app/TRANSLATING.md` | Translator instructions |
| `app/lang/en.json` | English labels |
| `app/lang/es.json` | Spanish labels |
| `app/lang/eu.json` | Basque labels |
| `app/src/main.rs` | Application shell, UI, canvas, input, simulation, tests |
| `app/src/model.rs` | Data model, hoops, threads, stitch generation, tests |
| `app/src/io.rs` | Project/DST/SVG/CSV I/O, file dialog, tests |
| `app/src/theme.rs` | Palette, style, hand-drawn icons, widgets |
| `app/src/lang.rs` | Translation loading, fallback, saved language, tests |
| `app/shots/*.png` | Development screenshots |
| `capture/README.md` | Initial clean-room note |
| `checks/**` | Check screenshots and a historical question request |
| `investigation.cany/**` | Evidence, journal, annotations, project metadata |
| `craft.json` | Craft Anything project metadata and check history |
| `feed.json` | Activity feed (98 items) |
| `build-activity.log` | Human-readable build/run log |
| `inbox/build-progress.json` | Build progress fingerprint |
| `usage.json` | Work time and cost counters |
| `pinned/play.log` | Empty placeholder |
| `reference/` | Empty |
