//! TEMPLATE for a new test case. Copy this file to `src/cases/<name>.rs` (the name in snake_case, e.g. `panel_slider_value.rs`),
//! change what the `EDIT:` marks say, then add `<name>` to the list in `src/cases/mod.rs`. This copy itself is never shown in the
//! plugin: it is only compiled by the tests, so it cannot go out of date without a test failing.

use crate::case::{Case, Kind, Received, Variant};
use crate::fmt::show;
use figcraft_plugin_sdk::{Value, json};

/// A case with one panel. For a case that compares several layouts, add a `Variant` for each to `VARIANTS` and
/// return that layout's controls from `controls` (see `panel_button_field_order.rs`).
static VARIANTS: [Variant; 1] = [Variant {
    // EDIT: a short name, shown in the log of what was received.
    id: "default",
    // EDIT: what the panel menu shows for this panel (needed to tell panels apart when there are several).
    label: "Default",
    // EDIT: what this panel is, in a sentence or two.
    about: "One button and one text field: press the button and see what the plugin received.",
    // EDIT: what you will SEE if the behaviour being tested is a bug (may be empty: "").
    if_bug: "",
    // EDIT: what you will SEE if the behaviour is fine (may be empty: "").
    if_ok: "",
}];

pub static CASE: Case = Case {
    // EDIT: unique, lower case, words joined by hyphens (also the name used in the README).
    id: "my-new-case",
    // EDIT: the name shown in the list of cases.
    title: "My new case",
    // EDIT: one or two sentences: what this case verifies.
    description: "What this case verifies, in one or two sentences.",
    variants: &VARIANTS,
    controls,
    handle,
};

/// EDIT: the controls of the panel, in the order they appear. Types: `button`, `text`, `number`, `boolean`, `select`,
/// `list` (the last two need `options`), `slider` (needs `min` and `max`), `color` and `note`. Names: letters, digits, `.`,
/// `-`, `_`; never starting with `h_`.
fn controls(_variant: &Variant) -> Vec<Value> {
    vec![
        json!({"type": "text", "name": "word", "label": "Word"}),
        json!({"type": "button", "name": "go", "label": "Go"}),
    ]
}

/// EDIT: what to show when something happens. Each text in the list is one line of the log. Return an empty list to show nothing.
fn handle(r: &Received<'_>) -> Vec<String> {
    match r.kind {
        Kind::Open => Vec::new(),
        Kind::Button => vec![format!("PRESSED {} -> word received: {}", r.control, show(r.field("word")))],
        Kind::Field => vec![format!("CHANGED {} -> value: {}", r.control, show(r.value()))],
    }
}
