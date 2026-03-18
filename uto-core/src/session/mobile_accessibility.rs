//! Accessibility-focused label resolution helpers for mobile sessions.

fn xpath_literal(value: &str) -> String {
    if !value.contains('"') {
        return format!("\"{value}\"");
    }
    if !value.contains('\'') {
        return format!("'{value}'");
    }

    let parts: Vec<String> = value.split('"').map(|s| format!("\"{s}\"")).collect();
    format!("concat({})", parts.join(", '\"', "))
}

/// Normalizes a human intent label for case-insensitive matching.
pub(crate) fn normalize_intent_label(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn mobile_accessibility_attrs() -> &'static [&'static str] {
    // Keep cross-platform coverage broad: Android (text/content-desc/hint/resource-id)
    // and iOS (label/name/value).
    &[
        "text",
        "label",
        "name",
        "value",
        "content-desc",
        "hint",
        "resource-id",
    ]
}

fn lower_attr_expr(attr: &str) -> String {
    format!(
        "translate(normalize-space(@{}), 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz')",
        attr
    )
}

/// Builds ordered XPath strategies used by mobile `select(label)` resolution.
pub(crate) fn build_mobile_select_xpaths(label: &str) -> Vec<String> {
    let raw = label.trim();
    let raw_literal = xpath_literal(raw);
    let normalized = normalize_intent_label(raw);
    let normalized_literal = xpath_literal(&normalized);

    let attrs = mobile_accessibility_attrs();

    let exact_raw = attrs
        .iter()
        .map(|a| format!("@{a}={raw_literal}"))
        .collect::<Vec<_>>()
        .join(" or ");

    let exact_normalized = attrs
        .iter()
        .map(|a| format!("{}={}", lower_attr_expr(a), normalized_literal))
        .collect::<Vec<_>>()
        .join(" or ");

    let contains_raw = attrs
        .iter()
        .map(|a| format!("contains(@{a}, {raw_literal})"))
        .collect::<Vec<_>>()
        .join(" or ");

    let contains_normalized = attrs
        .iter()
        .map(|a| format!("contains({}, {})", lower_attr_expr(a), normalized_literal))
        .collect::<Vec<_>>()
        .join(" or ");

    vec![
        format!("//*[{exact_raw}]"),
        format!("//*[{exact_normalized}]"),
        format!("//*[{contains_raw}]"),
        format!("//*[{contains_normalized}]"),
    ]
}

#[cfg(test)]
mod tests {
    use super::{build_mobile_select_xpaths, normalize_intent_label, xpath_literal};

    #[test]
    fn xpath_literal_handles_double_quotes() {
        let lit = xpath_literal("Say \"Hello\"");
        assert_eq!(lit, "'Say \"Hello\"'");
    }

    #[test]
    fn xpath_literal_handles_both_quote_types() {
        let lit = xpath_literal("it's \"fine\"");
        assert!(lit.starts_with("concat("));
    }

    #[test]
    fn normalize_intent_label_collapses_case_and_spaces() {
        assert_eq!(
            normalize_intent_label("  Search   Settings  "),
            "search settings"
        );
    }

    #[test]
    fn build_mobile_select_xpaths_includes_case_insensitive_and_contains_fallbacks() {
        let xpaths = build_mobile_select_xpaths("Search Settings");
        assert_eq!(xpaths.len(), 4);
        assert!(xpaths[0].contains("@text="));
        assert!(xpaths[0].contains("@label="));
        assert!(xpaths[1].contains("translate(normalize-space(@text)"));
        assert!(xpaths[2].contains("contains(@name"));
        assert!(xpaths[3].contains("contains(translate(normalize-space(@content-desc)"));
    }

    #[test]
    fn build_mobile_select_xpaths_covers_hint_and_value_attrs() {
        let xpaths = build_mobile_select_xpaths("email");
        let combined = xpaths.join(" ");
        assert!(combined.contains("@hint="));
        assert!(combined.contains("@value="));
    }
}
