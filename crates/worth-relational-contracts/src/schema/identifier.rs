//! Identifier grammar for declarations registered by Relational.

pub(crate) fn is_simple(name: &str) -> bool {
    !name.is_empty()
        && !name
            .chars()
            .any(|ch| ch == '.' || ch.is_whitespace() || ch.is_control())
}

pub(crate) fn is_dotted(name: &str) -> bool {
    name.split('.').all(is_simple)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_names_disallow_periods_and_both_grammars_disallow_whitespace_and_controls() {
        for invalid in ["", "a b", "a\tb", "a\nb", "a\u{00a0}b", "a\0b"] {
            assert!(!is_simple(invalid), "{invalid:?}");
            assert!(!is_dotted(invalid), "{invalid:?}");
        }
        assert!(is_simple("a-b"));
        assert!(!is_simple("a.b"));
        assert!(is_dotted("a.b"));
        for invalid in [".", ".a", "a.", "a..b"] {
            assert!(!is_dotted(invalid), "{invalid:?}");
        }
    }
}
