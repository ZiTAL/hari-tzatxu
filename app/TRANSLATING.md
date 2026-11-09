# Translating Stitch Studio

All the text in the window lives in the `lang` folder, one file per
language:

    lang/en.json   English
    lang/es.json   Español
    lang/eu.json   Euskara

Each line of a file is one label:

    "menu.file": "File",

The part on the left (`menu.file`) is the label's name; the part on the
right, in quotes, is what you see in the window. Change the text on the
right and that label changes. Do not change the name on the left and
keep the commas and quotation marks exactly where they are.

A few lines contain `{n}`, `{path}`, `{e}`, `{name}` or `{what}`. Those
are filled in by the program (a number, a folder name, an error). Keep
them as they are; you may move them around inside the sentence.

## Where the program looks

1. The `lang` folder next to the program.
2. The `lang` folder in the folder you start the program from.
3. The copy built into the program (used when no folder is found).

A file found on disk wins over the built-in copy, label by label, so a
half-finished translation still works: any label you have not translated
yet stays in the built-in text.

## Adding a fourth language

Copy `lang/en.json` to `lang/xx.json`, translate the text on the right,
and change the first line:

    "_name": "Your language name",

The language then appears in the picker at the top right of the window
(after a restart).

## Choosing the language

- The picker at the top right of the window, or
- `stitch_studio --lang es` on the command line, or
- leave it alone and the program remembers the last choice (saved in
  `~/.config/stitch-studio/settings.json`).

## Checking a file

After editing, start the program. If a file has a misplaced comma or
quote, the program says nothing and simply uses the built-in text for
that language, so the window keeps working. `cargo test` also checks that
all three files have the same labels and the same `{...}` placeholders.
