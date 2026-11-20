//! Case `plugin-input-clock` (a documented skeleton): which fields the plugin input holds, and whether any of them carries
//! the time or the user's time zone.
//!
//! Today the plugin sandbox has no clock (FigCraft's `docs/plugins.md`: "no file system, network, clock or randomness"), and
//! the input a plugin receives holds only `command`, `params`, `selection`, `event` with `values`, and `widget`. So the
//! case shows `(not available)` for both. It exists to become a real test the day the plugin API hands over a time:
//!
//! 1. Update the SDK tag in `Cargo.toml` to one whose `Input` has the new field (the SDK drops what it does not know).
//! 2. In `probe` below, add that field to the lines, and make `CLOCK_KEYS` / `ZONE_KEYS` name the keys it uses, if they differ.
//! 3. Change the text of `VARIANTS` and the README line for this case.

use crate::case::{Case, Kind, Received, Variant};
use crate::fmt::{EMPTY, NOT_AVAILABLE, show, show_str};
use figcraft_plugin_sdk::{Value, json};

/// Keys, in `params` or in `values`, that would carry the time (compared in lower case).
const CLOCK_KEYS: [&str; 7] = ["time", "timestamp", "now", "date", "datetime", "clock", "utc"];
/// Keys, in `params` or in `values`, that would carry the user's time zone (compared in lower case).
const ZONE_KEYS: [&str; 5] = ["tz", "timezone", "time_zone", "utc_offset", "locale"];

const DESCRIPTION: &str = "Skeleton for a future test. Lists the fields the plugin input holds and looks for any clock or time zone information. Today the sandbox has no clock, so the expected result is (not available) for both. When the plugin API starts to give a time, this case is where it will show up (see the comment at the top of the case's file).";

static VARIANTS: [Variant; 1] = [Variant {
    id: "default",
    label: "Default",
    about: "Press Probe input: the log lists what this callback received. A line with a key and a value in place of (not available) means the API now gives that piece of information.",
    if_bug: "",
    if_ok: "",
}];

pub static CASE: Case = Case {
    id: "plugin-input-clock",
    title: "Plugin input: clock and time zone",
    description: DESCRIPTION,
    variants: &VARIANTS,
    controls,
    handle,
};

fn controls(_variant: &Variant) -> Vec<Value> {
    vec![json!({"type": "button", "name": "probe", "label": "Probe input"})]
}

/// The first `key=value` of `params` or `values` whose key is one of `keys`.
fn find(r: &Received<'_>, keys: &[&str]) -> Option<String> {
    [&r.input.params, r.values()]
        .into_iter()
        .filter_map(Value::as_object)
        .flat_map(|m| m.iter())
        .find(|(k, _)| keys.contains(&k.to_lowercase().as_str()))
        .map(|(k, v)| format!("{k}={}", show(v)))
}

/// What this callback's input holds, and the time information in it.
fn probe(r: &Received<'_>) -> Vec<String> {
    let input = r.input;
    let widget = if input.widget.is_some() { "present" } else { EMPTY };
    let control = input.event.as_ref().map_or("", |e| e.control.as_str());
    vec![
        format!("command: {} | selection: {} | widget: {}", show_str(&input.command), show(&json!(input.selection)), widget),
        format!("params: {} | event.control: {} | event.value: {}", show(&input.params), show_str(control), show(r.value())),
        format!("values: {}", show(r.values())),
        format!(
            "clock: {} | time zone: {}",
            find(r, &CLOCK_KEYS).unwrap_or_else(|| NOT_AVAILABLE.to_string()),
            find(r, &ZONE_KEYS).unwrap_or_else(|| NOT_AVAILABLE.to_string())
        ),
    ]
}

fn handle(r: &Received<'_>) -> Vec<String> {
    match r.kind {
        // Opening shows nothing: the input of that callback is the launcher's, not this case's. Press Probe.
        Kind::Button => probe(r),
        Kind::Open | Kind::Field => Vec::new(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::bench::tests::event_input;
    use figcraft_plugin_sdk::Input;

    fn run(input: &Input, kind: Kind) -> Vec<String> {
        handle(&Received { input, variant: &VARIANTS[0], kind, control: "probe" })
    }

    #[test]
    fn today_there_is_no_clock_and_no_time_zone() {
        let input = event_input("probe", Value::Null, json!({}));
        let l = run(&input, Kind::Button);
        assert_eq!(l.last().unwrap(), "clock: (not available) | time zone: (not available)");
        assert_eq!(l[0], "command: \"open\" | selection: (empty) | widget: (empty)");
        assert_eq!(l[2], "values: (empty)");
    }

    #[test]
    fn a_time_in_the_input_would_be_found() {
        let input = Input::parse(r#"{"command":"open","params":{"Now":"2026-11-20T15:00:00Z"},"event":{"control":"probe","value":null},"values":{"tz":"-03:00"}}"#).unwrap();
        let l = run(&input, Kind::Button);
        assert_eq!(l.last().unwrap(), "clock: Now=\"2026-11-20T15:00:00Z\" | time zone: tz=\"-03:00\"");
    }

    #[test]
    fn a_probe_has_at_most_the_lines_the_log_shows() {
        let input = event_input("probe", Value::Null, json!({}));
        assert!(run(&input, Kind::Button).len() <= crate::bench::MAX_LINES);
        assert!(run(&input, Kind::Open).is_empty() && run(&input, Kind::Field).is_empty());
    }
}
