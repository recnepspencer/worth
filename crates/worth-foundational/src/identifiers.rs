//! Portable schema and naming identifier grammar.

/// One nonempty segment without dots, whitespace, or control characters.
pub fn is_valid_simple_identifier(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| ch == '.' || ch.is_whitespace() || ch.is_control())
}

/// Two or more valid segments separated by dots.
pub fn is_valid_dotted_identifier(value: &str) -> bool {
    value.contains('.') && is_valid_role_identifier(value)
}

/// One or more valid segments separated by dots.
pub fn is_valid_role_identifier(value: &str) -> bool {
    value.split('.').all(is_valid_simple_identifier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_grammar_distinguishes_simple_dotted_and_role_names() {
        for invalid in [
            "", "a b", "a\tb", "a\u{a0}b", "a\0b", "a\u{85}b", ".", ".a", "a.", "a..b",
        ] {
            assert!(!is_valid_simple_identifier(invalid), "{invalid:?}");
            assert!(!is_valid_dotted_identifier(invalid), "{invalid:?}");
            assert!(!is_valid_role_identifier(invalid), "{invalid:?}");
        }
        assert!(is_valid_simple_identifier("面"));
        assert!(is_valid_role_identifier("面"));
        assert!(!is_valid_dotted_identifier("面"));
        assert!(!is_valid_simple_identifier("面.境界"));
        assert!(is_valid_dotted_identifier("面.境界"));
        assert!(is_valid_role_identifier("面.境界"));
    }
}
