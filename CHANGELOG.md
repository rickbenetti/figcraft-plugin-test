# Changelog

All notable changes to FigCraft Plugin Test. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Version 0.x: expect rough edges.

## [0.1.0] - 2026-11-20

First version.

### Requirements

- A FigCraft that runs **plugin API 2** (the manifest says `"api": 2`). The plugin is built against `figcraft-plugin-sdk`
  tag `v0.14.0` and is meant for **figcraft-cli / FigCraft 0.18.1**.
- **Minimum FigCraft version: not verified.** A FigCraft that runs a plugin API older than 2 refuses the manifest when it is
  imported (FigCraft's `docs/plugins.md`, the `api` row).

### Added

- **The bench.** One menu command, **Open test bench…**, opens a panel that lists the registered test cases. Pick one and press
  **Open case**. A case shows, on the panel itself, what the plugin received (no console is needed: the plugin sandbox has none):
  a log with the newest event first, each with its number, the panel it came from and the lines the case chose to show. An
  empty or missing value is shown as `(empty)`. Buttons: **Clear log**, **Back to cases**, **Close**.
- **The registry of cases.** A case is one small file in `src/cases/` with an id, a title, a short description, the controls of
  its panel(s) and a handler that says what to show for an event. Adding a case is: copy `src/cases/template.rs`, edit it, and
  add its name to the one list in `src/cases/mod.rs`. The tests check that every case, and the template, follow the rules
  of the host's panel (limits, names, types) and that the README mentions every case.
- **Case `panel-button-field-order`**: three panels to compare, **A** (button `save` above the field `note`), **B** (control:
  `note` above `save`) and **C** (`save1`, `note`, `save2`). Each click shows which button was pressed and the value of `note`
  the plugin received in that very event, and each field change shows the value it carried, so the order in which they arrive
  can be read from the numbers.
- **Case `plugin-input-clock`** (a documented skeleton): lists the fields of the plugin input and looks for any clock or time
  zone information. Today it shows `(not available)` for both.
- `scripts/build.sh` (builds `dist/`, the folder to import) and `scripts/package.sh` (a reproducible release zip).

### Known limitations

- **How it was tested.** `cargo test` (22 tests) and `cargo clippy` pass. The module was also loaded and driven, panel by panel,
  in the plugin sandbox of the FigCraft `v0.14.0` sources (the `figcraft-plugin` crate: manifest and panel validation, the
  WebAssembly interpreter), with the order of the events simulated from the source of the app's panel. It was then run inside the
  FigCraft 0.18.1 window (2026-11-20): with the button above the field (panels A and C) the click reaches the
  plugin before the typed value and `note` arrives empty in the click event; with the field above the button
  (panel B, and `save2` in panel C) the click carries the typed value. See the case `panel-button-field-order`.
- All texts are in English; there is no language setting.
- A case sees only what the SDK exposes of the plugin input (`command`, `params`, `selection`, `event`, `values`, `widget`). A
  field the plugin API adds later is not visible until the SDK tag is updated.
