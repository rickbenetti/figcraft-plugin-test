//! FigCraft Plugin Test: a test bench for the behaviour of FigCraft's plugin API.
//!
//! The plugin has one menu command, which opens a panel listing the registered test cases (`cases/`). A case
//! shows, on the panel itself, what the plugin received (the sandbox has no console).

mod bench;
mod case;
mod cases;
mod fmt;

use figcraft_plugin_sdk::{Input, Result, Value, entry};

/// The manifest command "open" (the only one). From the menu it opens the panel; from then on the panel's buttons and
/// fields call back here with an `event`.
fn run(input: Input) -> Result<Value> {
    match input.command.as_str() {
        "open" if input.event.is_some() => bench::event(input),
        "open" => bench::open(input),
        other => Err(format!("unknown command: {other}")),
    }
}

entry!(run);
