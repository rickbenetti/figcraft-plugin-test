//! Case `panel-button-field-order`: in which order a panel's button and text field reach the plugin.
//!
//! A text field hands its value to the plugin only when it loses focus; pressing a button takes the focus, so the field
//! commits in the same frame as the click. The host calls the plugin back in the order of the panel's controls
//! (FigCraft v0.14.0, `crates/ui-egui/src/dialogs/plugins_ui.rs`, `controls()` and `panel_windows()`). With the field above
//! the button the plugin hears the field first; with the button above the field it can hear the click first, with the old
//! value of the field in `values`. This case shows both, side by side.

use crate::case::{Case, Kind, Received, Variant};
use crate::fmt::show;
use figcraft_plugin_sdk::{Value, json};

const DESCRIPTION: &str = "Checks in which order a panel's button and text field reach the plugin. A text field gives its value to the plugin only when it loses focus, so with the button above the field the click can reach the plugin before the typed text does, and the click then sees the old value of note. FigCraft PR #1732 fixed only the layout with the field above the button. For each panel: type hello in Note, click the button straight away (no Enter, no Tab) and read the log. The order of the lines, and the note value in the CLICK line, tell if the bug exists. For a new try, type a different text or open another panel.";

static VARIANTS: [Variant; 3] = [
    Variant {
        id: "A",
        label: "A - Save above Note",
        about: "Button save (Save) above the text field note (Note). This is the layout that can show the bug.",
        if_bug: "The log shows CLICK save with note received as (empty), or as an older text, and only after it CHANGE note with hello.",
        if_ok: "The log shows CHANGE note with hello first, then CLICK save with note received as hello.",
    },
    Variant {
        id: "B",
        label: "B - Note above Save",
        about: "Control: the text field note (Note) above the button save (Save). This is the layout PR #1732 fixed.",
        if_bug: "CLICK save shows note received as (empty) here too: the fix of PR #1732 does not work in this version.",
        if_ok: "The log shows CHANGE note with hello first, then CLICK save with note received as hello. This is the expected result.",
    },
    Variant {
        id: "C",
        label: "C - Save 1, Note, Save 2",
        about: "Button save1 (Save 1), text field note (Note), button save2 (Save 2): shows whether the position of a button decides what it receives.",
        if_bug: "CLICK save1 (above the field) shows note received as (empty), while CLICK save2 (below it) shows hello.",
        if_ok: "Both buttons show note received as hello.",
    },
];

pub static CASE: Case = Case {
    id: "panel-button-field-order",
    title: "Panel: button and field order",
    description: DESCRIPTION,
    variants: &VARIANTS,
    controls,
    handle,
};

fn field() -> Value {
    json!({"type": "text", "name": "note", "label": "Note"})
}

fn button(name: &str, label: &str) -> Value {
    json!({"type": "button", "name": name, "label": label})
}

/// The controls of each panel, in the order that matters.
fn controls(variant: &Variant) -> Vec<Value> {
    match variant.id {
        "A" => vec![button("save", "Save"), field()],
        "B" => vec![field(), button("save", "Save")],
        "C" => vec![button("save1", "Save 1"), field(), button("save2", "Save 2")],
        _ => Vec::new(),
    }
}

/// What the plugin received: which button was clicked and the value of `note` in that very event, or which field changed.
fn handle(r: &Received<'_>) -> Vec<String> {
    let selection = show(&json!(r.input.selection));
    match r.kind {
        Kind::Open => Vec::new(),
        Kind::Button => vec![
            format!("CLICK {} -> note received: {}", r.control, show(r.field("note"))),
            format!("event.value: {} | selection: {}", show(r.value()), selection),
        ],
        Kind::Field => vec![
            format!("CHANGE {} -> value: {} | note in values: {}", r.control, show(r.value()), show(r.field("note"))),
            format!("selection: {selection}"),
        ],
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::bench::tests::event_input;

    fn lines(variant: usize, kind: Kind, control: &str, value: Value, values: Value) -> Vec<String> {
        let input = event_input(control, value, values);
        handle(&Received { input: &input, variant: &VARIANTS[variant], kind, control })
    }

    #[test]
    fn the_three_panels_have_the_layouts_of_the_case() {
        let names = |v: usize| -> Vec<String> { controls(&VARIANTS[v]).iter().map(|c| c["name"].as_str().unwrap().to_string()).collect() };
        assert_eq!(names(0), ["save", "note"]);
        assert_eq!(names(1), ["note", "save"]);
        assert_eq!(names(2), ["save1", "note", "save2"]);
        let types = |v: usize| -> Vec<String> { controls(&VARIANTS[v]).iter().map(|c| c["type"].as_str().unwrap().to_string()).collect() };
        assert_eq!(types(0), ["button", "text"]);
        assert_eq!(types(2), ["button", "text", "button"]);
    }

    #[test]
    fn a_click_shows_the_button_and_the_note_it_received() {
        let l = lines(0, Kind::Button, "save", Value::Null, json!({"note": ""}));
        assert_eq!(l[0], "CLICK save -> note received: (empty)");
        let l = lines(1, Kind::Button, "save", Value::Null, json!({"note": "hello"}));
        assert_eq!(l[0], "CLICK save -> note received: \"hello\"");
        let l = lines(2, Kind::Button, "save2", Value::Null, json!({}));
        assert_eq!(l[0], "CLICK save2 -> note received: (empty)", "a missing note is shown like an empty one");
    }

    #[test]
    fn a_change_shows_the_field_and_its_value() {
        let l = lines(0, Kind::Field, "note", json!("hello"), json!({"note": "hello"}));
        assert_eq!(l[0], "CHANGE note -> value: \"hello\" | note in values: \"hello\"");
    }

    #[test]
    fn opening_the_case_shows_nothing() {
        assert!(lines(0, Kind::Open, "h_open", Value::Null, json!({})).is_empty());
    }
}
