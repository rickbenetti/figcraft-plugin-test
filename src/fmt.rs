//! How the bench writes a value on the panel: one place, so every case shows an empty or missing value the same way.

use figcraft_plugin_sdk::Value;

/// What an empty or missing value looks like on the panel.
pub const EMPTY: &str = "(empty)";
/// What a piece of information the plugin API does not give looks like on the panel.
pub const NOT_AVAILABLE: &str = "(not available)";

/// Longest value shown in one line of the log, in characters.
const MAX_SHOWN: usize = 80;

/// `s` cut to `max` characters, with `...` at the end when something was left out.
pub fn cut(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(3)).collect();
    out.push_str("...");
    out
}

/// Text the host accepts in a label, an option or a note: no control characters (they become spaces), trimmed and cut
/// to `max` characters.
pub fn safe_text(s: &str, max: usize) -> String {
    let cleaned: String = s.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
    cut(cleaned.trim(), max)
}

/// A value as the panel shows it: `(empty)` for null, an empty text, list or object; a text between quotes; anything else
/// as compact JSON. Cut to a readable length.
pub fn show(v: &Value) -> String {
    match v {
        Value::Null => EMPTY.to_string(),
        Value::String(s) if s.is_empty() => EMPTY.to_string(),
        Value::Array(a) if a.is_empty() => EMPTY.to_string(),
        Value::Object(m) if m.is_empty() => EMPTY.to_string(),
        other => cut(&other.to_string(), MAX_SHOWN),
    }
}

/// A plain text as the panel shows it, like [`show`] does for a JSON text.
pub fn show_str(s: &str) -> String {
    show(&Value::String(s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use figcraft_plugin_sdk::json;

    #[test]
    fn empty_and_missing_values_are_called_empty() {
        for v in [json!(null), json!(""), json!([]), json!({})] {
            assert_eq!(show(&v), "(empty)", "{v}");
        }
        assert_eq!(show_str(""), "(empty)");
    }

    #[test]
    fn other_values_are_shown_as_json() {
        assert_eq!(show(&json!("hello")), "\"hello\"");
        assert_eq!(show(&json!(0)), "0");
        assert_eq!(show(&json!(false)), "false");
        assert_eq!(show(&json!([12, 15])), "[12,15]");
        assert_eq!(show(&json!({"note": "x"})), "{\"note\":\"x\"}");
    }

    #[test]
    fn a_line_break_in_a_value_cannot_reach_the_panel() {
        assert_eq!(show(&json!("a\nb")), "\"a\\nb\"");
        assert!(!safe_text("a\nb\tc", 20).chars().any(char::is_control));
    }

    #[test]
    fn long_values_are_cut() {
        let long = "x".repeat(500);
        let shown = show_str(&long);
        assert_eq!(shown.chars().count(), 80);
        assert!(shown.ends_with("..."));
        assert_eq!(cut("abc", 3), "abc");
        assert_eq!(safe_text("  hi  ", 10), "hi");
        assert_eq!(safe_text(&"y".repeat(30), 10).chars().count(), 10);
    }
}
