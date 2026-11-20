# FigCraft Plugin Test

A permanent test bench for FigCraft's plugin API, as a plugin for FigCraft. It lists **test cases**; you pick one, use its
panel, and the plugin shows **on the panel itself** what it received (the plugin sandbox has no console). It grows by adding
cases: one small file each.

> FigCraft Plugin Test is an independent, community-made plugin for FigCraft. It is not affiliated with, endorsed by, or supported by the ArtCraft team. FigCraft and ArtCraft are names and trademarks of their respective owners.

## Test cases

| Case id | What it verifies |
|---|---|
| `panel-button-field-order` | In which order a panel's button and text field reach the plugin: with the button above the field, does the click arrive before the typed text? Three panels to compare. |
| `plugin-input-clock` | A documented skeleton: which fields the plugin input holds, and whether any of them carries the time or the time zone. Today: `(not available)`. |

## Requirements

- FigCraft, [a design app from the ArtCraft team](https://getartcraft.com/apps/figcraft). Meant for **FigCraft / figcraft-cli 0.18.1**,
  with a plugin SDK at tag `v0.14.0` and `"api": 2` in the manifest.
- To build the plugin (you do not need this to add cases, only to get the `.wasm`): Rust 1.95 (`rust-version = "1.95"` in `Cargo.toml`)
  with the `wasm32-unknown-unknown` target, for example with rustup:

  ```sh
  rustup toolchain install 1.95
  rustup override set 1.95          # run inside this repository
  rustup target add wasm32-unknown-unknown
  ```

  The build downloads the SDK from GitHub (a git dependency) and the other crates from crates.io. `Cargo.lock` is committed, so
  builds use the same versions.

## Install

```sh
scripts/build.sh
```

It writes `dist/` (`figcraft-plugin.json` and `figcraft_plugin_test.wasm`, about 100 KB) and prints the path of the manifest. In FigCraft:
**Plugins ▸ Development ▸ Import plugin from manifest…** (in the Portuguese app: **Plugins ▸ Desenvolvimento ▸ Importar plugin a
partir do manifesto…**), pick `dist/figcraft-plugin.json`.

FigCraft keeps the plugin in its preferences and **reads the module again on every run**, so after you rebuild (`scripts/build.sh`) the
new code runs without importing again.

To make a release zip, run `scripts/package.sh`: it builds, then writes `dist/figcraft-plugin-test-<version>.zip` (the manifest, the
`.wasm`, `LICENSE` and `THIRD_PARTY_NOTICES.md` at the root, with fixed timestamps) and prints its SHA-256. It does not tag or publish anything.

## Run a case

1. Run **Plugins ▸ FigCraft Plugin Test ▸ Open test bench…**. A panel opens with the list of cases and a short description of the one picked.
2. Pick a case and press **Open case**.
3. Use the panel (each case says what to do). The **Received (newest first)** notes show what the plugin received. Each event has a number,
   `#1`, `#2`, ... counted from the moment the case was opened, so the order in which things arrived can be read from the numbers. An empty or
   missing value is shown as `(empty)`.
4. **Clear log** empties the log, **Back to cases** returns to the list, **Close** closes the panel.

### `panel-button-field-order`

The case has three panels (the menu **Panel** switches between them; switching also empties the text field):

| Panel | Layout |
|---|---|
| A | button `save` ("Save"), then the field `note` ("Note") |
| B (control) | the field `note`, then the button `save` |
| C | button `save1`, the field `note`, button `save2` |

For each panel: type `hello` in Note and click the button **straight away**, without pressing Enter or Tab (those would make the field
confirm its value first). Then read the log:

- **Bug exists** (panel A): `CLICK save -> note received: (empty)` has a **lower** number than `CHANGE note -> value: "hello"`. The click
  arrived before the typed text, and saw the old value. In panel C, `save1` shows `(empty)` and `save2` shows `"hello"`.
- **Bug does not exist**: `CHANGE note` arrives first, and `CLICK save` shows `note received: "hello"`. This is what panel B should show (it is the
  layout that FigCraft PR #1732 fixed).

Background: a text field gives its value to the plugin only when it loses focus, which happens in the same moment as the click on the button; the
app then calls the plugin back in the order of the panel's controls (read in the FigCraft `v0.14.0` sources, `plugins_ui.rs`; not stated in
`docs/plugins.md`). The case measures what the version you run does.

### `plugin-input-clock`

Open the case and press **Probe input**: the log lists the fields the callback received (`command`, `selection`, `widget`, `params`,
`event.control`, `event.value`, `values`) and ends with `clock: (not available) | time zone: (not available)`. The plugin sandbox has no clock
(FigCraft's `docs/plugins.md`), and the plugin input has no time field. When the plugin API gives a time, the line shows the key and value instead.
The comment at the top of `src/cases/plugin_input_clock.rs` says what to change then.

## How to add a case (no programming needed)

You edit text in three places and run two commands. A case is one file; nothing else in the code changes.

1. **Copy the template.** Copy `src/cases/template.rs` to `src/cases/<name>.rs`, with the name in lower case and words joined by `_`
   (for example `panel_slider_value.rs`).
2. **Fill in the texts.** Open your copy and change what each line marked `EDIT:` asks for: the case's `id` (lower case, words joined by `-`),
   `title`, `description`, the panel's `label` and `about`, and what to expect if the behaviour is a bug or is fine. Then change the list of
   controls (the panel) and the lines the case shows for each event (`handle`). The comments in the file say what is allowed. If you
   get stuck here, this is a good step to hand to Claude Code: it only has to change that one file.
3. **Register it.** In `src/cases/mod.rs`, add the file name (without `.rs`) to the list in `register! { ... }`, one name per line.
   This is the only place where cases are registered.
4. **Document it.** Add a row to the table of cases above, with the case's `id`, and add a line to `CHANGELOG.md`. The tests fail if the
   README does not mention a registered case.
5. **Check it.** Run `cargo test`. It checks the rules of the host's panel for every case (at most 64 controls, names with letters, digits, `.`, `-`,
   `_`, lengths, options) and says which case and which rule when something is wrong.
6. **Build and try it.** Run `scripts/build.sh`, then run the plugin again in FigCraft (no new import is needed).

Rules a case follows:

- The sandbox has no console, so everything a case wants to say goes in the **lines it returns** from `handle`. Show an empty or missing value
  with `show(...)` (from `src/fmt.rs`): it writes `(empty)`. A piece of information the API does not give is `NOT_AVAILABLE`, `(not available)`.
- Control names may not start with `h_` (the bench's own). A log keeps the last 6 events, 5 lines each; a line is cut at 1,000 characters.
- A case never changes the document: the manifest asks only for `read` access and the plugin calls no FigCraft command.

## Tests

```sh
cargo test        # 22 tests: the formatting of values, the panels, the rules every case follows, the two cases
cargo clippy --all-targets
```

The tests do not call the host (off WebAssembly every host call fails); they drive the pure logic. What the app itself does with the panel is what the
cases measure.

## Known limitations

- Version 0.1.x: expect rough edges.
- All texts are in English; there is no language setting.
- A case sees only what the SDK exposes of the plugin input (`command`, `params`, `selection`, `event`, `values`, `widget`). A field that the plugin API
  adds later is not visible until the SDK tag in `Cargo.toml` is updated to one that knows it.
- The panel is small and its layout changes a little from one event to the next (the log grows). Nothing here depends on a layout.

## License

MIT, see [LICENSE](LICENSE). The plugin is built from Rust crates with their own licences; their notices are in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Credits

Built with Claude Code (Anthropic).
