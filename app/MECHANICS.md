# JANOME — mechanics sheet

What this rebuild reproduces, and where every number and rule came from.

The original is `JANOME.EXE` — the Janome DigitizerPro 3.0 main program (its manifest
calls it "Wilcom Embroidery System Software", Wilcom International Pty Ltd), a 32-bit
Windows GUI program of 7,639,040 bytes:

    sha256  cab5d0d28c0eafeb9ecbdb15997dc913eda3c9f51784ef283e1aa2c51554cde8
    PE32, linker 7.0, image base 0x400000, entry point RVA 0x6fe280 (VA 0xafe280)
    TimeDateStamp 2009-11-09 10:37:50 UTC

The program itself is packed (see below), so its own functions cannot be read. The
engine it loads can: the installed folder `/home/projects/janome/DigitizerPro` (the
same bytes as the Windows install in `~/.wine`) holds the DLLs that do the work —
`RESOURCE.DLL` for the hoop names and labels, `MOD1.DLL` for the hoop library and the
stitch criteria, `IMPORT.DLL` for the DST reader, `DEFMGR.DLL` for the defaults file.
Those were opened with Ghidra 12.1.4 (headless) and every function below was read as
pseudo-C, function by function; the addresses are virtual addresses in each file.
The 38 hoop definitions in `RES/*.DSC` and the registry keys were read as data.

    RESOURCE.DLL  sha256 8078d02899c433d416153d708f4945a07a3a71741efd0585f5d78c7893f66739
    MOD1.DLL      sha256 25988c9b9b7d18fac7fa11b3bee93f8da47413940d272f6a78003949c65bbd81
    IMPORT.DLL    sha256 faabe3e689f5f07c59eb283be2e797db4c7b233bb2b4888691641fe9c4320eb3
    DEFMGR.DLL    sha256 7e4898b8aa1a6e862b839867faa92569b09322965fc990953e6c65a38e0eae78

## The functions that were read

| File | Function / address | What it gave |
|---|---|---|
| JANOME.EXE | PE header, entry point 0xafe280 | it is a PE32 program whose code is packed (below) |
| RESOURCE.DLL | `?GetHoopDisplayName@@YA?AVString@@ABVDataItem@@@Z` **0x100027c0** | reads the hoop's property **0x9ca**, then maps it through `FUN_10002640` |
| RESOURCE.DLL | `FUN_10002640` | bounds-checks the index against `DAT_1000c36c`, looks it up in `DAT_1000c364` (display) and `DAT_1000c370` (internal); returns the input unchanged when the index is out of range or the display string differs |
| RESOURCE.DLL | `?GetHoopInternalName@@YA?AVString@@ABV1@@Z` **0x10002700** | the same lookup against `DAT_1000c380` / `DAT_1000c38c` / `DAT_1000c388` |
| RESOURCE.DLL | `FUN_100024c0` (initialiser) | walks **45** slots of **0x104** bytes from the string table at **0x10007120** (offset array at 0x10007220) and fills the display and internal name tables |
| MOD1.DLL | `?GetBestFittingHoop@HoopLibrary@@QAEPBVDataItem@@INNN@Z` **0x1009ba60** | the fit rule (strict `<`, allowance on both sides, smallest area, earlier hoop wins a tie) |
| MOD1.DLL | `?GetMatchingHoop@HoopLibrary@@QBEPBVDataItem@@INN@Z` **0x1009b810** | same width/height properties 0x9cc / 0x9cd; a tolerance test, not the fit rule |
| MOD1.DLL | `?CreateUserDefinedHoopPrototype@HoopLibrary@@QBEXAAVDataItem@@@Z` **0x1009bd90** | sets properties 0x9cc / 0x9cd to 0 — they are the hoop's two sides |
| MOD1.DLL | `?GetDesignStitchCriteria@DesignDocument@@QBE?AVStitchCriteria@@_N@Z` **0x10018390** | design criteria come from machine property **0x99** |
| MOD1.DLL | `?GetObjectStitchCriteria@DesignDocument@@QBE?AVStitchCriteria@@PBVEmbroideryObject@@@Z` **0x10018400** | object criteria: `EmbroideryObject::GetFuncProperty(obj, 1, 0x99, …)` plus the tufting and chenille flags |
| MOD1.DLL | `?GetStitchCriteria@FillStyle@@UBE?AVStitchCriteria@@XZ` **0x101dd4a0** | a fill style's three spacing fields are at **+0x10, +0x14, +0x18** |
| MOD1.DLL | `??0StitchGenerationCriteria@@QAE@ABVStitchCriteria@@0@Z` **0x103e1f80** | how a design criteria and an object criteria are merged |
| MOD1.DLL | `?FixLargeStitchLength@StraightFill@@MAEXXZ` **0x1022a3b0** | only writes `INT_MAX` (0x7fffffff) to +0x90 |
| IMPORT.DLL | `?LoadStitch@DSTStitchFormat@@…@Z` **0x1002ddc0** | the 3-byte record, the exact end record and the 0x40 / 0x80 / 0xC0 branches |
| IMPORT.DLL | `FUN_1002dba0` (decode helper) | three 256-entry tables of `(dx, dy)` byte pairs and the 0x7F stop marker |
| DEFMGR.DLL | `?GetGlobalDefaultData@@YA_NIAAVDataItem@@@Z` **0x10002210**, `?PutGlobalDefaultData…` **0x10002230** | thin accessors; the DLL statically links the HASP runtime (`enter hasp_encrypt`, `enter hasp_decrypt`, `classic_key_info`) |

## The main program is wrapped: what that means

`JANOME.EXE` cannot be decompiled function by function, because its code is encrypted
on disk. Every section was re-measured (Shannon entropy over the section's virtual
bytes; a raw-size figure is lower where the section is padded with zeros):

| Section | VA | Virtual size | Entropy | Meaning |
|---|---|---|---|---|
| `.text` | 0x401000 | 5,558,394 | 8.0000 | encrypted program code |
| `CONST` | 0x94f000 | 48 | 4.96 | the only plain code bytes: a tiny constant block |
| `.rdata` | 0x950000 | 622,810 | 7.9997 | encrypted data |
| `.data` | 0x9e9000 | 1,126,768 (192,512 on disk) | 7.9978 | encrypted data |
| `.protect` | 0xafe000 | 1,243,134 | 7.9966 | the decrypting stub and its own data |
| `.rsrc` | 0xafd000 | 752 | 5.17 | only a stub's resources (a real MFC app of this size has far more) |

An entropy of 8.0 is random-looking data; real x86 code sits around 6.0–6.4. The
entry point lands in `.protect`, not in `.text`: at **0xafe280** the stub saves the
registers, calls `GetModuleHandleA` on the strings `kernel32` (at 0xafeedc) and
`user32` (at 0xafeee8), stores the two handles at 0xafe4f8 / 0xafe4fc, copies
`GetProcAddress` and `MessageBoxA` into 0xafe500 / 0xafe504, decrypts the code
(`FUN_00afe479`) and calls the real entry through `DAT_00afeef0`. `0x401000`, the
first byte of `.text`, disassembles to noise (`das`, `pop %ebp`, `bound`), which is
the signature of packed code. A string search of the file finds **0** occurrences of
`Hoop`, `Janome`, `JANOME`, `stitch`, `DST`, `Update` and `http://`; the 18 `mm` hits
are random byte pairs inside the encrypted sections. The only readable text is the
manifest at 0xafd060, which names Wilcom.

The dongle is part of that wrapper. The folder ships `HASPDINST.EXE`
(sha256 113e714e…, 8,917,560 bytes) and `JNDongleUpdateDll.dll`
(sha256 b3717a24…, 1,875,968 bytes), which has the same `.protect` section as the
main program — an Aladdin/SafeNet HASP envelope around the Janome code. The USB-key
check therefore runs inside the encrypted stub and cannot be read or patched from the
file; there is no plain function in `JANOME.EXE` that says "is the key present".

That is why the rebuild is a new program rather than a patched copy: it never loads
the HASP runtime and has no key check at all, so it opens without the dongle. The
original file was read only — it was not modified.

## What it imports

32 libraries. The ones that describe the program:

- its own engine DLLs: `ezstitch.dll`, `geometry.dll`, `import.dll`, `defmgr.dll`,
  `dialog.dll`, `desview.dll`, `image.dll`, `product.dll`, `controls.dll`, `mod1.dll`,
  `resource.dll`, `hwrsetup.dll`, `ediskdll.dll`, `wessys.dll`, `ng.dll`,
  `oleintf.dll`, `versndll.dll`, `gdiplus.dll`
- `?GetMachineDetails@JanomeFeedbackInterface@@QBEXAAVDataItem@@@Z` from `Import.dll`
  — the machine-feedback interface (what the sewing machine reports)
- Windows: `DeviceIoControl`, `CreateFileA`, `FindFirstFileA`, `GetVersion`,
  `LoadLibraryA`, `GetProcAddress`, `GetLocalTime` — the stub needs these to unpack
  and to talk to the key.

## Internet addresses the program knows

From `HKCU\Software\Janome\DigitizerPro\3.0N\Internet` (the program's own registry
keys, read from `~/.wine/user.reg`):

| Name | Value |
|---|---|
| Update URL | `http://packagedfilestorage.com/download/janome` |
| Order URL | `http://www.janome.co.jp/Dealers/DealerUpdate` |

These are the only network endpoints the installed program records. The rebuild opens
no socket to them or anywhere else; it contains no network code at all.

## Hoops

There are **39** hoops: 38 are defined in the program's own `RES/*.DSC` files as
`HOOP RECTANGLE (w,h) Name` or `HOOP 3_POSITION (w,h) Name`, and
`M4:MB Hoop (100 x 60)` exists only in the display string table (its file is not
installed with this copy).

    RES/HoopsCommon(Digitizer).DSC        19 hoops
    RES/HoopsProMBOnly(Digitizer).DSC     19 hoops
    RESOURCE.DLL display strings          39 names (M4 included)

The display names are the resource strings at **0x10007120**, one every **0x104**
bytes; `FUN_100024c0` walks 45 slots and fills the display and internal tables.
`?GetHoopDisplayName@@YA?AVString@@ABVDataItem@@@Z` at **RESOURCE.DLL 0x100027c0**
reads the hoop's property **0x9ca** and passes it to `FUN_10002640`, which takes the
index (`vtable+8`), bounds-checks it against `DAT_1000c36c`, looks the index up in
`DAT_1000c364` (display) and `DAT_1000c370` (internal), and returns the input
unchanged when the index is out of range or the internal names differ.
`?GetHoopInternalName@@YA?AVString@@ABV1@@Z` at **0x10002700** does the same with
`DAT_1000c380` / `DAT_1000c38c` / `DAT_1000c388`.

The table order below is the program's own (the order of the strings at 0x10007120);
it matters for the fit rule's tie-break. The numeric sizes are taken from the `.DSC`
lines, not parsed out of the labels:

| # | Hoop | Width × height |
|---|---|---|
| 0 | Hoop ST (126 x 110) | 126 × 110 |
| 1 | Hoop RE (140 x 200) | 140 × 200 |
| 2 | Hoop SQ (200 x 200) | 200 × 200 |
| 3 | Hoop FA (50 x 50) | 50 × 50 |
| 4 | Hoop MA (200 x 280) | 200 × 280 |
| 5 | Hoop AQ (120 x 180) | 120 × 180 |
| 6 | Hoop ASQ (180 x 180) | 180 × 180 |
| 7 | Hoop HH (100 x 90) | 100 × 90 |
| 8 | 200 Hoop (140 x 140) | 140 × 140 |
| 9 | 200 Hoop (50 x 50) | 50 × 50 |
| 10 | M1:MB Hoop (240 x 200) | 240 × 200 |
| 11 | M2:MB Hoop (126 x 110) | 126 × 110 |
| 12 | M3:MB Hoop (50 x 50) | 50 × 50 |
| 13 | M4:MB Hoop (100 x 60) | 100 × 60 |
| 14 | Hoop A(F) (126 x 110) Ver.3.0 | 126 × 110 |
| 15 | Hoop A (110 x 110) Ver.2.21 | 110 × 110 |
| 16 | Hoop B (140 x 200) | 140 × 200 |
| 17 | Hoop C (50 x 50) | 50 × 50 |
| 18 | Hoop D (220 x 190) (Giga Hoop) | 220 × 190 |
| 19 | Hoop No. 1 & 3 (126 x 90) | 126 × 90 |
| 20 | H1:MB Hoop (100 x 90) | 100 × 90 |
| 21 | J1 No.11:MB Hoop (D30) | 30 × 30 |
| 22 | J1 No.12:MB Hoop (D30) | 30 × 30 |
| 23 | J2 No.6:MB Hoop (24 x 54) | 24 × 54 |
| 24 | J3 No.13:MB Hoop (64 x 28) | 64 × 28 |
| 25 | J4 No.8:MB Hoop (42 x 67) | 42 × 67 |
| 26 | J5 No.3:MB Hoop (47 x 47) | 47 × 47 |
| 27 | J6 No.2:MB Hoop (66 x 66) | 66 × 66 |
| 28 | J6 No.9:MB Hoop (66 x 66) | 66 × 66 |
| 29 | J7 No.1:MB Hoop (110 x 95) | 110 × 95 |
| 30 | J8 No.7:MB Hoop (120 x 121) | 120 × 121 |
| 31 | S1:MB Hoop (37 x 59) | 37 × 59 |
| 32 | S2:MB Hoop (28 x 50) | 28 × 50 |
| 33 | T1:MB Hoop (D34) | 34 × 34 |
| 34 | T2:MB Hoop (D54) | 54 × 54 |
| 35 | T3:MB Hoop (D84) | 84 × 84 |
| 36 | T4:MB Hoop (D112) | 112 × 112 |
| 37 | T5:MB Hoop (D142) | 142 × 142 |
| 38 | T6:MB Hoop (78 x 168) | 78 × 168 |

The seven `D..` hoops are round on the machine, but the program's own hoop file
stores each one as `HOOP RECTANGLE (d,d)`, so its fit test sees a square of that
diameter. The rebuild offers them as those squares (39 presets). `Hoop MA` and
`Hoop D` are `HOOP 3_POSITION` entries: the declared overall field is 200 × 280 and
220 × 190, while one position measures 200 × 200 and 130 × 190. The rebuild uses the
declared size, which is the size in the name.

Old design files that point at the five sizes the first rebuild invented still open
at their saved size (see `LEGACY_HOOPS` in `src/model.rs`).

## The "does it fit" rule (MOD1.DLL 0x1009ba60)

`?GetBestFittingHoop@HoopLibrary@@QAEPBVDataItem@@INNN@Z` decompiles to:

1. get the hoop list for the machine (`FUN_1009b720`); an empty list returns nothing;
2. walk the items in 0x1c-byte steps, and only consider an item whose type field
   (`+8`) is **6**;
3. read its two sides from properties **0x9cc** and **0x9cd**
   (`FUN_10404fc0(0x9cc, …)` and `0x9cd`); a hoop that lacks either is skipped;
4. a hoop fits only when `design_width + allowance < hoop_width` **and**
   `design_height + allowance < hoop_height` — strictly smaller, the same allowance
   added to both sides;
5. of all the hoops that pass, keep the one with the smallest area; the comparison is
   `best_area <= new_area`, so on equal areas the **earlier hoop in the library's
   order** stays;
6. if none passes, return nothing.

`CreateUserDefinedHoopPrototype` at **0x1009bd90** initialises those same 0x9cc /
0x9cd properties to 0, which is what confirms they are the hoop's two sides.
`GetMatchingHoop` at **0x1009b810** reads the same two properties but its test is a
tolerance comparison, not the fit rule.

The allowance is a parameter of the caller, and the caller lives in the encrypted
`JANOME.EXE`, so its value could not be read. The rebuild applies the same rule with
an allowance of 0 mm (`HOOP_ALLOWANCE_MM` in `src/model.rs`), and uses it when a DST
file is imported to choose the smallest hoop that holds the design
(`best_preset_for`). Because the comparison is strict, a 50 × 50 mm design does **not**
fit a 50 × 50 mm hoop, and an 80 × 80 mm design picks the 84 × 84 mm T3.

Because the tie-break depends on the library order, `src/model.rs` carries
`LIBRARY_RANK` (the position of each preset in the 0x10007120 table) and
`best_preset_for` uses it: of the four 50 × 50 hoops, a 48 × 48 mm design picks
**Hoop FA**, which comes first in the original's table.

## DST stitch files (IMPORT.DLL 0x1002ddc0 and 0x1002dba0)

`?LoadStitch@DSTStitchFormat@@UAE?AW4LoadStitchResult@StitchFormat@@AAVistream@@AAVLongVect@@AAVUIntIxList@@AA_N@Z`
at **0x1002ddc0** reads one record:

- exactly **3 bytes** per stitch (`istream::read(…,3)`);
- the record `00 00 F3` is the end of the file, and the check is on all three bytes
  (`b0 == 0 && b1 == 0 && b2 == 0xF3`), not on the third byte alone;
- the movement is decoded **before** the kind is looked at, by the helper at
  **0x1002dba0**; if that helper returns 0, `LoadStitch` returns a non-zero result
  (1) and produces **no stitch** — it does not keep a partial delta;
- then `kind = b2 & 0xC0`:
  - `0x40` — colour change (calls the colour handler when `this[0xc]` is set);
  - `0x80` — jump: sets the caller's bool to true and returns;
  - `0xC0` — the combined branch (calls the handler when `this[0xb]` is set);
  - anything else — an ordinary stitch.

The helper holds three 256-entry tables of `(dx, dy)` byte pairs, one per record
byte: byte 0 is added first, then byte 1, then byte 2. `0x7F` is the table's stop
marker: when a looked-up pair starts with `0x7F`, the helper returns 0 immediately,
so that byte and every byte after it contribute nothing and the record is dropped.
For an ordinary record the pairs are the familiar 1/9, 3/27 and 81 decomposition, i.e.
deltas in 0.1 mm with a maximum of 121 units (12.1 mm) per record — which is what
`src/io.rs` writes and reads (`dst_encode` clamps at ±121). The rebuild reproduces
the 0x7F stop for any of the three bytes (`dst_decode` returns `None`), the exact
`00 00 F3` end, the 0x40 / 0x80 / 0xC0 branches, and ends the stitch list at a
record the helper rejects.

The rebuild's own writer still emits three end records, as machines expect; that is
a writing convention, not something read from `LoadStitch`.

## Stitch settings and criteria (MOD1.DLL)

The original keeps one `StitchCriteria` object per design and one per object: eight
4-byte fields (offsets 0, 4, 8, 0xc, 0x18, 0x1c, 0x20), a double at +0x10 and a bool
at +0x24.

- `DesignDocument::GetDesignStitchCriteria` **0x10018390** builds it from the
  machine's property **0x99** (`FUN_104163d0(1, 0x99)`, taking the dword at +8 of the
  result) and passes the tufting flag;
- `DesignDocument::GetObjectStitchCriteria` **0x10018400** reads
  `EmbroideryObject::GetFuncProperty(obj, 1, 0x99, …)` and the chenille/tufting
  flags (`IsTuftingObject`, `IsChenilleObject`);
- `FillStyle::GetStitchCriteria` **0x101dd4a0** copies the style's three spacing
  fields at **+0x10, +0x14, +0x18**: +0x10 goes to slot 0 or slot 1 depending on the
  flag at +0x1c, +0x14 to slot 2 and +0x18 to slot 3, leaving the rest zero;
- `StitchGenerationCriteria::StitchGenerationCriteria` **0x103e1f80** merges a design
  criteria and an object criteria: the bool at +0x24 is the OR of the two bools,
  +0x28 is set to 1, slots 0, 1, 3 and 6 take the smaller non-zero value, slot 2 and
  the double at +0x10 take the **larger**, slots at +0x1c and +0x20 come from the
  first set, and the field at +0x2c is the slot-1 value squared;
- `StraightFill::FixLargeStitchLength` **0x1022a3b0** only writes `INT_MAX`
  (0x7fffffff) to +0x90.

The numeric values that fill those fields live in `DEFAULTS.DFL`
(`BIN/DEFAULTS.DFL`, 3,633 bytes, identical to `RES/Defaults.DFL`). That file is
encrypted: entropy 7.9476 over all 3,633 bytes and all 256 byte values present, and
the index of coincidence per byte class is ~0.0039 for every key length 1–40 (random
is 0.0039), so it is not a short repeating-key XOR. `DEFMGR.DLL` does not decrypt it
in readable code either: it statically links the HASP runtime, whose wrapper strings
(`enter hasp_encrypt`, `enter hasp_decrypt`, `enter hasp_get_size`,
`classic_key_info`) are in the file, and its exported accessors
`GetGlobalDefaultData` / `PutGlobalDefaultData` (0x10002210 / 0x10002230) are
22-byte thunks. Without the dongle the defaults cannot be recovered. The rebuild
therefore keeps its own spacing numbers (2.5 mm run, 0.4 mm satin, 0.45 mm fill) and
does not claim they are Janome's.

Nothing in the readable code is a sewing-time estimate. The program's own speed
label is **stitches per second**, not stitches per minute:

    RESOURCE.DLL 0x1007b634   " Speed:              (s/s)"
    RESOURCE.DLL 0x100b60ec   " Speed: %5d (s/s) "

No numeric default speed appears in any readable string or function. The rebuild's
12.5 stitches/second (750 per minute) is a reconstruction, and the time is simply
stitches ÷ rate.

## Labels and messages (RESOURCE.DLL)

The controls behind the rebuild's "stitch length" and "density" fields are the
program's own labels. These are the `.rsrc` strings (UTF-16 dialog templates), so
their addresses are where the text lives, not a function:

| Text | Address |
|---|---|
| `Stitch Length:` | 0x1005de40, 0x10061ef0, 0x1006335e, 0x10063682, 0x1006538a, 0x1006809a |
| `Min Stitch length:` | 0x1005df0c |
| `Min Stitch Length:` | 0x10061fbc |
| `Stitch Density:` | 0x10087e7c, 0x100897c6 |
| `Maximum Stitch Length:` | 0x10068136 |
| `Run Stitch Length:` | 0x10067c16, 0x1006e34e |
| `This design does not fit the Hoop…` | 0x100b3ffe |
| `Speed: … (s/s)` | 0x1007b634, 0x100b60ec |

## Values the rebuild uses now

| Value | Rebuild | Source |
|---|---|---|
| Hoop catalogue | 39 presets, names and mm sizes above | 38 `HOOP RECTANGLE` lines in RES/*.DSC; `M4` from the 39-name table at RESOURCE.DLL 0x10007120 |
| Hoop order | the original's display-table order for tie-breaks | `LIBRARY_RANK` in `src/model.rs`, from RESOURCE.DLL 0x10007120 (stride 0x104) |
| Display / internal name lookup | by hoop index, bounds-checked, property 0x9ca | `GetHoopDisplayName` 0x100027c0, `FUN_10002640`, `GetHoopInternalName` 0x10002700 |
| Default hoop | Hoop B, 140 × 200 mm | the standard rectangular hoop in that catalogue (the original's own default is in the encrypted executable) |
| Fit rule | design + 0 mm **<** hoop side, both sides; smallest area wins; ties keep the earlier hoop in library order | `GetBestFittingHoop` 0x1009ba60 |
| DST record | 3 bytes, 0.1 mm, ±121 units, exact `00 00 F3` end, 0x7F drops the record, 0x40/0x80/0xC0 kinds | `DSTStitchFormat::LoadStitch` 0x1002ddc0, helper 0x1002dba0 |
| Stitch criteria fields | style spacing at +0x10/+0x14/+0x18; merge takes the min of the non-zero values and the max of the doubles | `FillStyle::GetStitchCriteria` 0x101dd4a0, `StitchGenerationCriteria` 0x103e1f80 |
| Units | millimetres in the model, mm labels | `126 x 110 mm`-style strings and `HOOP RECTANGLE (w,h)` lines |
| Speed unit | stitches per second (s/s) | RESOURCE.DLL 0x1007b634, 0x100b60ec |
| Stitch spacing | 2.5 mm run, 0.4 mm satin, 0.45 mm fill | reconstruction: the defaults file is HASP-encrypted (above) |
| Default speed | 12.5 s/s | reconstruction: no readable default |
| Network | none opened | registry endpoints above are not contacted |
| Dongle | none | the original's HASP envelope is not reproduced |

## What could not be read, and what the rebuild does instead

- **Stitch length, satin spacing and fill density defaults.** They are members of the
  engine's `StitchCriteria` class and the values that fill them come from the
  encrypted `DEFAULTS.DFL` (HASP, see above). The rebuild keeps its own spacing
  numbers and does not claim they are Janome's.
- **The fit allowance.** It is the fourth argument of `GetBestFittingHoop`; the
  caller is in the encrypted executable, so the rebuild uses 0 mm.
- **Machine speed and sewing time.** No speed value, "SPM", or time estimate exists in
  any readable string or function of the program; the machine supplies its own speed.
  The rebuild's 12.5 s/s is a reconstruction.
- **The USB-key check itself.** Inside the encrypted stub (see above); only its
  packaging (HASP files, `.protect` section, entropy) is visible. Nothing in the
  original was changed.
- **`Hoop D` and `Hoop MA` are multi-position hoops.** Their `.DSC` blocks declare
  the overall field (220 × 190 and 200 × 280) and a single position measures
  130 × 190 and 200 × 200. The rebuild offers the declared overall size, matching the
  display name.

## Build

`cargo test` — 27 tests pass; `cargo build` — succeeds; `--screenshot` writes a
1280×820 PNG. The program starts with no key, no network and the Hoop B default.
