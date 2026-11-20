//! The bench: the panel that lists the cases, opens one and shows what the plugin received (plugin API 2).
//!
//! Two screens, both drawn from `State`:
//!
//! * the launcher: a list of the registered cases, the description of the one picked, and the buttons Open case and Close;
//! * a case: its description, a menu of its panels (when it has more than one), the notes of the panel picked, the case's own
//!   controls, a log of what the plugin received (newest first) and the buttons Clear log, Back to cases and Close.
//!
//! The bench's own controls are all named `h_...`, so they can never clash with a case's. Events of the bench's controls
//! are not logged; every other event is handed to the case, and the lines it returns go to the log.
//!
//! The first half of this file is pure logic (the state, the panel specs, what an event changes); the second half is the glue
//! to the host (`show_ui`, `close_ui`). The tests drive the first half only: off WebAssembly every host call fails.

use crate::case::{Case, Kind, Received};
use crate::cases;
use crate::fmt::safe_text;
use figcraft_plugin_sdk::{Input, Result, Value, close_ui, json, show_ui};
use std::cell::RefCell;

/// Longest label or option the host accepts, and longest note.
pub const MAX_LABEL: usize = 80;
pub const MAX_NOTE: usize = 1000;
/// Most controls the host accepts in a panel, and longest title.
pub const MAX_CONTROLS: usize = 64;
pub const MAX_TITLE: usize = 64;
/// How many received events the log keeps, and how many lines of one event it shows.
pub const MAX_ENTRIES: usize = 6;
pub const MAX_LINES: usize = 5;

/// The bench's own controls.
pub const H_CASE: &str = "h_case";
pub const H_OPEN: &str = "h_open";
pub const H_VARIANT: &str = "h_variant";
pub const H_CLEAR: &str = "h_clear";
pub const H_BACK: &str = "h_back";
pub const H_CLOSE: &str = "h_close";

// ---------------------------------------------------------------------------------------------
// Pure logic
// ---------------------------------------------------------------------------------------------

/// The screen being shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// The list of cases, with the one at this position picked.
    Launcher { selected: usize },
    /// A case (its position in the registry) and the panel of it (its position in the case) that is open.
    Case { index: usize, variant: usize },
}

/// One received event, as the log keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// 1, 2, 3... counted from the moment the case was opened.
    pub seq: u32,
    /// The id of the panel the event came from.
    pub variant: String,
    pub lines: Vec<String>,
}

/// What the bench remembers between callbacks (the module stays loaded while the panel is open).
#[derive(Debug, PartialEq, Eq)]
pub struct State {
    pub screen: Screen,
    pub log: Vec<Entry>,
    pub seq: u32,
    /// The next drawing empties the text fields (a new panel starts clean). A drawing without it keeps what was typed.
    pub reset: bool,
}

impl Default for State {
    fn default() -> State {
        State { screen: Screen::Launcher { selected: 0 }, log: Vec::new(), seq: 0, reset: false }
    }
}

fn note(text: &str) -> Value {
    json!({"type": "note", "label": safe_text(text, MAX_NOTE)})
}

fn button(name: &str, label: &str) -> Value {
    json!({"type": "button", "name": name, "label": safe_text(label, MAX_LABEL)})
}

/// The panel's spec for the state.
pub fn spec(state: &State, all: &[&'static Case]) -> Value {
    match state.screen {
        Screen::Launcher { selected } => launcher_spec(all, selected),
        Screen::Case { index, variant } => {
            let found = all.get(index).and_then(|case| case.variants.get(variant).map(|v| (*case, v)));
            match found {
                Some((case, v)) => case_spec(case, v, state),
                None => launcher_spec(all, 0),
            }
        }
    }
}

fn launcher_spec(all: &[&'static Case], selected: usize) -> Value {
    let titles: Vec<String> = all.iter().map(|c| safe_text(c.title, MAX_LABEL)).collect();
    let shown = selected.min(all.len().saturating_sub(1));
    let description = match all.get(shown) {
        Some(c) => format!("{} ({}): {}", c.title, c.id, c.description),
        None => "No test cases are registered.".to_string(),
    };
    let controls = vec![
        note("Pick a test case and press Open case. A case shows on this panel what the plugin received."),
        json!({"type": "list", "name": H_CASE, "label": "Test cases", "options": titles, "value": titles.get(shown).cloned().unwrap_or_default()}),
        note(&description),
        button(H_OPEN, "Open case"),
        button(H_CLOSE, "Close"),
    ];
    json!({"title": "FigCraft Plugin Test", "controls": controls})
}

fn case_spec(case: &Case, v: &crate::case::Variant, state: &State) -> Value {
    let mut c: Vec<Value> = vec![note(&format!("Case: {} ({})", case.title, case.id)), note(case.description)];
    if case.variants.len() > 1 {
        let labels: Vec<String> = case.variants.iter().map(|x| safe_text(x.label, MAX_LABEL)).collect();
        c.push(json!({"type": "select", "name": H_VARIANT, "label": "Panel", "options": labels, "value": safe_text(v.label, MAX_LABEL)}));
    }
    for (prefix, text) in [("About this panel: ", v.about), ("If the bug exists: ", v.if_bug), ("If it does not exist: ", v.if_ok)] {
        if !text.is_empty() {
            c.push(note(&format!("{prefix}{text}")));
        }
    }
    for mut control in (case.controls)(v) {
        if state.reset
            && control.get("type").and_then(Value::as_str) == Some("text")
            && let Some(obj) = control.as_object_mut()
        {
            obj.insert("value".to_string(), json!(""));
        }
        c.push(control);
    }
    c.push(note("Received (newest first):"));
    // The log takes what is left of the host's limit of controls; the three buttons come after it.
    let room = MAX_CONTROLS.saturating_sub(c.len() + 3);
    let mut log: Vec<Value> = Vec::new();
    if state.log.is_empty() {
        log.push(note("(nothing yet)"));
    }
    for e in state.log.iter().rev() {
        for (i, line) in e.lines.iter().enumerate() {
            log.push(note(&if i == 0 { format!("#{} [{}] {}", e.seq, e.variant, line) } else { format!("> {line}") }));
        }
    }
    log.truncate(room);
    c.extend(log);
    c.push(button(H_CLEAR, "Clear log"));
    c.push(button(H_BACK, "Back to cases"));
    c.push(button(H_CLOSE, "Close"));
    json!({"title": safe_text(&format!("Test: {}", case.title), MAX_TITLE), "controls": c})
}

/// What kind of control `name` is, from the case's own controls.
fn kind_of(controls: &[Value], name: &str) -> Kind {
    let is_button = controls
        .iter()
        .any(|c| c.get("name").and_then(Value::as_str) == Some(name) && c.get("type").and_then(Value::as_str) == Some("button"));
    if is_button { Kind::Button } else { Kind::Field }
}

/// Hand an event to the open case and keep what it says in the log (nothing, when it says nothing).
fn record(state: &mut State, all: &[&'static Case], input: &Input, control: &str, kind: Option<Kind>) {
    let Screen::Case { index, variant } = state.screen else { return };
    let Some(case) = all.get(index) else { return };
    let Some(v) = case.variants.get(variant) else { return };
    let kind = kind.unwrap_or_else(|| kind_of(&(case.controls)(v), control));
    let received = Received { input, variant: v, kind, control };
    let mut lines = (case.handle)(&received);
    lines.truncate(MAX_LINES);
    if lines.is_empty() {
        return;
    }
    state.seq += 1;
    state.log.push(Entry { seq: state.seq, variant: v.id.to_string(), lines });
    let extra = state.log.len().saturating_sub(MAX_ENTRIES);
    state.log.drain(..extra);
}

/// What an event changes in the state. `control` is the control that was used.
pub fn apply(input: &Input, control: &str, state: &mut State, all: &[&'static Case]) {
    let text = input.event.as_ref().and_then(|e| e.value.as_str()).unwrap_or_default();
    match state.screen {
        Screen::Launcher { selected } => match control {
            H_CASE => {
                let picked = all.iter().position(|c| safe_text(c.title, MAX_LABEL) == text);
                state.screen = Screen::Launcher { selected: picked.unwrap_or(selected) };
            }
            H_OPEN if all.get(selected).is_some() => {
                state.screen = Screen::Case { index: selected, variant: 0 };
                state.log.clear();
                state.seq = 0;
                state.reset = true;
                record(state, all, input, control, Some(Kind::Open));
            }
            _ => {}
        },
        Screen::Case { index, .. } => match control {
            H_VARIANT => {
                let found = all.get(index).and_then(|case| case.variants.iter().position(|v| safe_text(v.label, MAX_LABEL) == text));
                if let (Some(i), Screen::Case { variant, .. }) = (found, &mut state.screen) {
                    *variant = i;
                    state.reset = true;
                }
            }
            H_CLEAR => state.log.clear(),
            H_BACK => state.screen = Screen::Launcher { selected: index },
            _ => record(state, all, input, control, None),
        },
    }
}

/// What a callback returns to the caller (a script sees it in `output`): where the bench is and what the log holds.
fn summary(state: &State, all: &[&'static Case], control: &str) -> Value {
    let log: Vec<Value> = state.log.iter().map(|e| json!({"seq": e.seq, "variant": e.variant, "lines": e.lines})).collect();
    match state.screen {
        Screen::Launcher { selected } => {
            json!({"phase": control, "screen": "launcher", "selected": all.get(selected).map(|c| c.id)})
        }
        Screen::Case { index, variant } => {
            let case = all.get(index);
            let id = case.map(|c| c.id);
            let panel = case.and_then(|c| c.variants.get(variant)).map(|v| v.id);
            json!({"phase": control, "screen": "case", "case": id, "variant": panel, "log": log})
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The glue to the host
// ---------------------------------------------------------------------------------------------

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// Run from the menu: show the list of cases.
pub fn open(_input: Input) -> Result<Value> {
    let all = cases::all();
    if all.is_empty() {
        return Err("no test cases are registered".to_string());
    }
    let state = State::default();
    show_ui(&spec(&state, &all))?;
    STATE.with(|s| *s.borrow_mut() = state);
    let ids: Vec<&str> = all.iter().map(|c| c.id).collect();
    Ok(json!({"phase": "open", "cases": ids}))
}

/// A button was pressed or a field changed.
pub fn event(input: Input) -> Result<Value> {
    let Some(ev) = &input.event else { return Ok(Value::Null) };
    let control = ev.control.clone();
    if control == H_CLOSE {
        close_ui();
        return Ok(json!({"phase": "close"}));
    }
    let all = cases::all();
    let mut state = STATE.with(|s| std::mem::take(&mut *s.borrow_mut()));
    apply(&input, &control, &mut state, &all);
    let drawn = show_ui(&spec(&state, &all));
    state.reset = false;
    let out = drawn.map(|()| summary(&state, &all, &control));
    STATE.with(|s| *s.borrow_mut() = state);
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// An input as the host builds it for a panel callback.
    pub fn event_input(control: &str, value: Value, values: Value) -> Input {
        let text = json!({"api": 2, "command": "open", "params": {}, "selection": [], "event": {"control": control, "value": value}, "values": values});
        Input::parse(&text.to_string()).unwrap()
    }

    /// The host's checks of a panel spec (`crates/plugin/src/manifest.rs` in FigCraft v0.14.0), as far as the bench can break them.
    pub fn check_spec(spec: &Value) {
        let controls = spec["controls"].as_array().unwrap();
        assert!(controls.len() <= MAX_CONTROLS, "{} controls", controls.len());
        assert!(spec.to_string().len() < 1024 * 1024);
        let title = spec["title"].as_str().unwrap();
        assert!(title.chars().count() <= MAX_TITLE && !title.chars().any(char::is_control), "title {title:?}");
        let mut names = BTreeSet::new();
        for c in controls {
            let kind = c["type"].as_str().unwrap();
            assert!(["note", "button", "text", "select", "list"].contains(&kind), "kind {kind}");
            let label = c["label"].as_str().unwrap_or_default();
            let max = if kind == "note" { MAX_NOTE } else { MAX_LABEL };
            assert!(label.chars().count() <= max, "label too long: {label}");
            assert!(!label.chars().any(char::is_control), "control character in {label:?}");
            assert!(kind == "note" || !label.is_empty(), "a control without a label");
            if kind != "note" {
                let name = c["name"].as_str().unwrap();
                assert!(!name.is_empty() && name.len() <= 64, "bad name {name}");
                assert!(name.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_')), "bad name {name}");
                assert!(names.insert(name.to_string()), "two controls called {name}");
            }
            if matches!(kind, "select" | "list") {
                let options: Vec<&str> = c["options"].as_array().unwrap().iter().map(|o| o.as_str().unwrap()).collect();
                assert!(!options.is_empty() && options.len() <= 64);
                assert!(options.iter().all(|o| o.chars().count() <= MAX_LABEL && !o.chars().any(char::is_control)));
                assert!(options.contains(&c["value"].as_str().unwrap()), "{} is not one of {options:?}", c["value"]);
            }
        }
    }

    fn texts(spec: &Value) -> Vec<String> {
        spec["controls"].as_array().unwrap().iter().filter(|c| c["type"] == "note").map(|c| c["label"].as_str().unwrap().to_string()).collect()
    }

    #[test]
    fn every_screen_of_every_case_is_a_valid_panel() {
        let all = cases::all();
        check_spec(&spec(&State::default(), &all));
        for selected in 0..all.len() {
            check_spec(&spec(&State { screen: Screen::Launcher { selected }, ..State::default() }, &all));
        }
        for (index, case) in all.iter().enumerate() {
            for variant in 0..case.variants.len() {
                for reset in [false, true] {
                    check_spec(&spec(&State { screen: Screen::Case { index, variant }, reset, ..State::default() }, &all));
                }
            }
        }
    }

    #[test]
    fn a_full_log_still_fits_the_host_limits() {
        let all = cases::all();
        let long = "z".repeat(3000);
        let entries: Vec<Entry> = (1..=MAX_ENTRIES as u32 * 3).map(|seq| Entry { seq, variant: "A".into(), lines: vec![long.clone(); MAX_LINES] }).collect();
        for index in 0..all.len() {
            let state = State { screen: Screen::Case { index, variant: 0 }, log: entries.clone(), ..State::default() };
            check_spec(&spec(&state, &all));
        }
    }

    #[test]
    fn the_launcher_lists_every_case_and_opens_the_one_picked() {
        let all = cases::all();
        let mut state = State::default();
        let second = all[1].title;
        apply(&event_input(H_CASE, json!(second), json!({"h_case": second})), H_CASE, &mut state, &all);
        assert_eq!(state.screen, Screen::Launcher { selected: 1 });
        assert!(texts(&spec(&state, &all)).iter().any(|t| t.contains(all[1].id)), "the description of the case picked is shown");
        apply(&event_input(H_OPEN, Value::Null, json!({"h_case": second})), H_OPEN, &mut state, &all);
        assert_eq!(state.screen, Screen::Case { index: 1, variant: 0 });
        assert!(state.reset);
        apply(&event_input(H_BACK, Value::Null, json!({})), H_BACK, &mut state, &all);
        assert_eq!(state.screen, Screen::Launcher { selected: 1 });
    }

    #[test]
    fn a_title_that_is_not_in_the_list_changes_nothing() {
        let all = cases::all();
        let mut state = State::default();
        apply(&event_input(H_CASE, json!("nope"), json!({})), H_CASE, &mut state, &all);
        assert_eq!(state, State::default());
    }

    fn open_case(all: &[&'static Case], id: &str) -> State {
        let index = all.iter().position(|c| c.id == id).unwrap();
        let mut state = State { screen: Screen::Launcher { selected: index }, ..State::default() };
        apply(&event_input(H_OPEN, Value::Null, json!({})), H_OPEN, &mut state, all);
        state
    }

    #[test]
    fn the_bench_own_controls_are_not_logged_and_the_cases_are() {
        let all = cases::all();
        let mut state = open_case(&all, "panel-button-field-order");
        assert!(state.log.is_empty(), "opening this case says nothing");
        apply(&event_input(H_CLEAR, Value::Null, json!({})), H_CLEAR, &mut state, &all);
        assert!(state.log.is_empty());
        apply(&event_input("save", Value::Null, json!({"note": "hi"})), "save", &mut state, &all);
        assert_eq!(state.log.len(), 1);
        assert_eq!((state.log[0].seq, state.log[0].variant.as_str()), (1, "A"));
        apply(&event_input(H_CLEAR, Value::Null, json!({})), H_CLEAR, &mut state, &all);
        assert!(state.log.is_empty());
    }

    #[test]
    fn the_log_keeps_the_newest_entries_and_a_new_panel_resets_the_fields() {
        let all = cases::all();
        let mut state = open_case(&all, "panel-button-field-order");
        for n in 0..(MAX_ENTRIES + 3) {
            apply(&event_input("save", Value::Null, json!({"note": format!("n{n}")})), "save", &mut state, &all);
        }
        assert_eq!(state.log.len(), MAX_ENTRIES);
        assert_eq!(state.log.last().unwrap().seq as usize, MAX_ENTRIES + 3);
        state.reset = false;
        let variant_b = all[0].variants[1].label;
        apply(&event_input(H_VARIANT, json!(variant_b), json!({})), H_VARIANT, &mut state, &all);
        assert_eq!(state.screen, Screen::Case { index: 0, variant: 1 });
        assert!(state.reset, "a new panel starts with empty text fields");
        let drawn = spec(&state, &all);
        let note_field = drawn["controls"].as_array().unwrap().iter().find(|c| c["name"] == "note").unwrap();
        assert_eq!(note_field["value"], "");
        state.reset = false;
        let kept = spec(&state, &all);
        let note_field = kept["controls"].as_array().unwrap().iter().find(|c| c["name"] == "note").unwrap();
        assert!(note_field.get("value").is_none(), "a redraw keeps what the user typed");
    }

    #[test]
    fn an_event_before_anything_is_open_or_after_a_bad_screen_is_harmless() {
        let all = cases::all();
        let mut state = State { screen: Screen::Case { index: 99, variant: 99 }, ..State::default() };
        check_spec(&spec(&state, &all));
        apply(&event_input("save", Value::Null, json!({})), "save", &mut state, &all);
        assert!(state.log.is_empty());
    }
}
