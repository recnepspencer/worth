#[cfg(test)]
mod tests {
    use crate::symbols::data::{InternedString, StringInterner, Symbol};

    #[test]
    fn raw_string_helpers_expose_raw_value_without_resolution() {
        let value = InternedString::raw("name");

        assert_eq!(value.raw_text(), Some("name"));
        assert_eq!(value.as_symbol(), None);
        assert!(value.raw_equals("name"));
        assert_eq!(value.canonical_text().as_ref(), "name");
    }

    #[test]
    fn symbolic_helpers_resolve_or_fall_back_through_symbol_identity() {
        let mut interner = StringInterner::default();
        let symbol = interner
            .intern("name")
            .expect("symbol fixture has identity capacity");
        let value = InternedString::Symbol(symbol);

        assert_eq!(value.raw_text(), None);
        assert_eq!(value.as_symbol(), Some(symbol));
        assert_eq!(
            value.resolve_text(|symbol| interner.resolve(symbol)),
            Some("name")
        );
        assert_eq!(value.canonical_text().as_ref(), "symbol:1");
        assert_eq!(
            value.resolve_text(|candidate| Some(if candidate == symbol {
                "name"
            } else {
                "other"
            })),
            Some("name")
        );
        assert_eq!(
            interner.normalize(value),
            Ok(InternedString::Symbol(symbol))
        );
    }

    #[test]
    fn symbolic_helpers_fail_closed_when_resolution_is_missing() {
        let value = InternedString::Symbol(Symbol(99));
        assert_eq!(value.resolve_text(|_| None), None);
    }

    #[test]
    fn raw_strings_intern_through_supplied_symbol_table() {
        let mut interner = StringInterner::default();
        let value = InternedString::raw("name");

        let symbol = interner
            .normalize(value)
            .expect("symbol fixture has identity capacity")
            .as_symbol()
            .unwrap();

        assert_eq!(interner.resolve(symbol), Some("name"));
    }
}
