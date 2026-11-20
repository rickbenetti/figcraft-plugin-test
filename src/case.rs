//! What a test case is: the contract between the bench (`bench.rs`) and each case (`cases/*.rs`).
//!
//! A case is a small, independent module with an id, a title, a short description of what it verifies, the controls of
//! its panel and a handler that says what to show for each event. It never draws the panel itself: the bench puts the
//! case's controls between its own (the case picker, the description, the log of what was received, the buttons).

use figcraft_plugin_sdk::{Input, Value};

/// One panel of a case. A case that compares several layouts has one variant for each; a case with a single panel
/// has one variant.
#[derive(Debug)]
pub struct Variant {
    /// A short, unique name shown in the log (`A`, `B`, `default`).
    pub id: &'static str,
    /// What the panel menu shows (at most 80 characters, unique inside the case).
    pub label: &'static str,
    /// What this panel is (a sentence or two). May be empty.
    pub about: &'static str,
    /// What to expect when the behaviour being tested is a bug. May be empty.
    pub if_bug: &'static str,
    /// What to expect when the behaviour is fine. May be empty.
    pub if_ok: &'static str,
}

/// What kind of thing just happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The case was just opened (nothing was pressed in it).
    Open,
    /// A button of the case's panel was pressed.
    Button,
    /// A field of the case's panel changed.
    Field,
}

/// What the plugin received, for the case's handler.
pub struct Received<'a> {
    /// The raw input of this callback, as the SDK parsed it.
    pub input: &'a Input,
    /// The panel (variant) that is open. Not every case needs it.
    #[allow(dead_code)]
    pub variant: &'a Variant,
    pub kind: Kind,
    /// The control's name (the button pressed or the field changed); the bench's own control when `kind` is `Open`.
    pub control: &'a str,
}

static NULL: Value = Value::Null;

impl Received<'_> {
    /// `event.value`: the field's new value (null for a button).
    pub fn value(&self) -> &Value {
        self.input.event.as_ref().map_or(&NULL, |e| &e.value)
    }

    /// Every field's value, by name (`values` of the input).
    pub fn values(&self) -> &Value {
        self.input.event.as_ref().map_or(&NULL, |e| &e.values)
    }

    /// The value of field `name` in `values`; null when the field is not there.
    pub fn field(&self, name: &str) -> &Value {
        self.values().get(name).unwrap_or(&NULL)
    }
}

/// A test case.
pub struct Case {
    /// Unique, kebab-case (`panel-button-field-order`). Also the name used in the README.
    pub id: &'static str,
    /// What the case list shows (at most 80 characters, unique).
    pub title: &'static str,
    /// A short description of what the case verifies (at most 1,000 characters).
    pub description: &'static str,
    /// The panels of the case (at least one).
    pub variants: &'static [Variant],
    /// The controls of one panel, in the host's panel spec (`{"type": "button", "name": ..., "label": ...}`...). Names may
    /// not start with `h_` (the bench's own).
    pub controls: fn(&Variant) -> Vec<Value>,
    /// What to show for an event: lines of text, shown newest first in the log. An empty list shows nothing.
    pub handle: fn(&Received<'_>) -> Vec<String>,
}
