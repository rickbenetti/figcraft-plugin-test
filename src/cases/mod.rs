//! The registry of test cases: the ONE place a new case is added.
//!
//! To add a case: create `src/cases/<name>.rs` (copy `template.rs`), then add `<name>` to the list below. Nothing else in the
//! code changes. The list order is the order the panel shows the cases in.

use crate::case::Case;

macro_rules! register {
    ($($module:ident),+ $(,)?) => {
        $(mod $module;)+

        /// Every registered case, in the order the panel lists them.
        pub fn all() -> Vec<&'static Case> {
            vec![$(&$module::CASE),+]
        }
    };
}

register! {
    panel_button_field_order,
    plugin_input_clock,
}

/// The template is compiled only by the tests, which check that it follows the rules every case follows.
#[cfg(test)]
mod template;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::{MAX_LABEL, MAX_NOTE};
    use crate::fmt::safe_text;
    use std::collections::BTreeSet;

    const README: &str = include_str!("../../README.md");
    const CHANGELOG: &str = include_str!("../../CHANGELOG.md");

    /// The rules every case follows (the template too).
    fn check_case(case: &Case) {
        let id = case.id;
        assert!(!id.is_empty() && id.len() <= 64, "case id {id:?}");
        assert!(id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'), "case id {id:?} is not kebab-case");
        assert!(!id.starts_with('-') && !id.ends_with('-'), "case id {id:?}");
        assert!(!case.title.trim().is_empty() && case.title.chars().count() <= MAX_LABEL, "title of {id}");
        assert_eq!(case.title, safe_text(case.title, MAX_LABEL), "the title of {id} has control characters, spaces at its ends or is too long");
        assert!(!case.description.trim().is_empty() && case.description.chars().count() <= MAX_NOTE, "description of {id}");
        assert!(!case.variants.is_empty(), "{id} has no panel");
        let mut variant_ids = BTreeSet::new();
        let mut variant_labels = BTreeSet::new();
        for v in case.variants {
            assert!(!v.id.is_empty() && v.id.chars().count() <= 24, "variant id {:?} of {id}", v.id);
            assert!(variant_ids.insert(v.id), "two panels of {id} have the id {}", v.id);
            assert_eq!(v.label, safe_text(v.label, MAX_LABEL), "label of panel {} of {id}", v.id);
            assert!(!v.label.is_empty() && variant_labels.insert(v.label), "label of panel {} of {id} is empty or repeated", v.id);
            for text in [v.about, v.if_bug, v.if_ok] {
                assert!(text.chars().count() <= MAX_NOTE - 30, "a text of panel {} of {id} is too long", v.id);
            }
            let controls = (case.controls)(v);
            let mut names = BTreeSet::new();
            for c in &controls {
                let kind = c["type"].as_str().unwrap_or_default();
                assert!(["button", "text", "number", "boolean", "select", "list", "slider", "color", "note"].contains(&kind), "control type {kind:?} in {id}");
                if kind == "note" {
                    continue;
                }
                let name = c["name"].as_str().unwrap_or_default();
                assert!(!name.is_empty() && name.len() <= 64, "control name {name:?} in {id}");
                assert!(name.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_')), "control name {name:?} in {id}");
                assert!(!name.starts_with("h_"), "control name {name:?} in {id}: names starting with h_ belong to the bench");
                assert!(names.insert(name.to_string()), "two controls of panel {} of {id} are called {name}", v.id);
            }
            assert!(controls.iter().any(|c| c["type"] == "button" || c["type"] == "text"), "panel {} of {id} has nothing to use", v.id);
        }
    }

    #[test]
    fn the_registry_is_not_empty_and_ids_and_titles_are_unique() {
        let all = all();
        assert!(!all.is_empty());
        let ids: BTreeSet<&str> = all.iter().map(|c| c.id).collect();
        let titles: BTreeSet<&str> = all.iter().map(|c| c.title).collect();
        assert_eq!(ids.len(), all.len(), "two cases have the same id");
        assert_eq!(titles.len(), all.len(), "two cases have the same title");
    }

    #[test]
    fn every_registered_case_follows_the_rules() {
        for case in all() {
            check_case(case);
        }
    }

    #[test]
    fn the_template_follows_the_rules_too() {
        check_case(&template::CASE);
    }

    #[test]
    fn the_readme_mentions_every_case_and_the_changelog_the_first_release() {
        for case in all() {
            assert!(README.contains(case.id), "README.md does not mention the case {}", case.id);
        }
        assert!(CHANGELOG.contains("## [0.1.0]"));
    }
}
