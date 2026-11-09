# Stitch Studio

A desktop embroidery workspace that starts with no USB key and opens no
network connection.

## Run it

    cargo run

## Options

    stitch_studio --lang es              start in Spanish (en, es, eu)
    stitch_studio --screenshot shot.png  draw the window, save a PNG, exit
    stitch_studio --help

## What it does

- Opens on a flower example that measures exactly 8 × 8 cm (80 × 80 mm);
  load it again any time from File → Flower example or with Ctrl+E.
- Hoop size is free: pick one of the presets or type any width and height
  in millimetres. The fabric, the rulers and the "design fits" check all
  follow the size you type, and the size is saved with the design.
- Sewing speed is editable: type the stitches per second of your machine
  and the estimated sewing time is recalculated immediately (stitches
  divided by speed). 12.5 stitches per second — 750 per minute — is the
  starting value.
- Draw filled rectangles and ellipses, run-stitch lines and satin columns
  on the hoop, move, duplicate, reorder and recolour them.
- Live stitch simulation with a speed control, stitch count, colour
  blocks and estimated sewing time.
- Millimetre rulers around all four edges of the hoop: a mark every
  millimetre, a longer one every five, a number every ten, with 0 at the
  centre of the hoop to match the position fields. Hide them with the
  ruler button or View → Hoop rulers.
- Save and open designs, import and export DST stitch files, export SVG
  drawings and CSV stitch lists.
- Three interface languages, chosen from the top-right picker and
  remembered between runs.

## Text and translations

Every label comes from the JSON files in `lang/`. See
[TRANSLATING.md](TRANSLATING.md).

## Built with

Rust and the egui/eframe interface toolkit; no runtime or installer to
add to the computer. The original program was read only: no file in it
was changed, and none of its code is copied into the rebuild.

## Building here

`cargo build` in this folder is enough; `.cargo/config.toml` (here and one
folder up) pins three things that the build checks depend on: the compiler
wrapper `.cargo/rustc-shim.sh`, the target folder, and the C compiler used for
linking. The wrapper only steps into this folder before compiling, so a build
started from a folder that has since been removed still works — without it
rustc stops with "Current directory is invalid". Keep those files.

