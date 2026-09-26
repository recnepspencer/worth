use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::{StringInterner, Symbol};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InternedString {
    Raw(String),
    Symbol(Symbol),
}

impl InternedString {
    pub fn raw(value: impl Into<String>) -> Self {
        Self::Raw(value.into())
    }

    pub fn as_raw_str(&self) -> Option<&str> {
        match self {
            Self::Raw(raw) => Some(raw.as_str()),
            Self::Symbol(_) => None,
        }
    }

    pub fn as_symbol(&self) -> Option<Symbol> {
        match self {
            Self::Raw(_) => None,
            Self::Symbol(symbol) => Some(*symbol),
        }
    }

    pub fn intern_with(&self, interner: &mut StringInterner) -> Symbol {
        match self {
            Self::Raw(raw) => interner.intern(raw),
            Self::Symbol(symbol) => *symbol,
        }
    }

    pub fn raw_equals(&self, expected: &str) -> bool {
        self.as_raw_str() == Some(expected)
    }

    pub fn resolve_with_interner<'a>(&'a self, interner: &'a StringInterner) -> Option<&'a str> {
        match self {
            Self::Raw(raw) => Some(raw.as_str()),
            Self::Symbol(symbol) => interner.resolve(*symbol),
        }
    }

    pub fn resolve_text<'a, F>(&'a self, resolve_symbol: F) -> Option<Cow<'a, str>>
    where
        F: FnOnce(Symbol) -> Option<&'a str>,
    {
        match self {
            Self::Raw(raw) => Some(Cow::Borrowed(raw.as_str())),
            Self::Symbol(symbol) => resolve_symbol(*symbol).map(Cow::Borrowed),
        }
    }

    pub fn canonical_text(&self) -> Cow<'_, str> {
        match self {
            Self::Raw(raw) => Cow::Borrowed(raw.as_str()),
            Self::Symbol(symbol) => Cow::Owned(format!("symbol:{}", symbol.0)),
        }
    }

    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        match self {
            Self::Raw(raw) => raw.capacity() as u64,
            Self::Symbol(_) => 0,
        }
    }
}

impl From<&str> for InternedString {
    fn from(value: &str) -> Self {
        Self::Raw(value.to_string())
    }
}

impl From<String> for InternedString {
    fn from(value: String) -> Self {
        Self::Raw(value)
    }
}

#[cfg(test)]
mod tests {
    use super::InternedString;
    use crate::symbols::data::{StringInterner, Symbol};

    #[test]
    fn raw_string_helpers_expose_raw_value_without_resolution() {
        let value = InternedString::raw("name");

        assert_eq!(value.as_raw_str(), Some("name"));
        assert_eq!(value.as_symbol(), None);
        assert!(value.raw_equals("name"));
        assert_eq!(value.canonical_text().as_ref(), "name");
    }

    #[test]
    fn symbolic_helpers_resolve_or_fall_back_through_symbol_identity() {
        let mut interner = StringInterner::default();
        let symbol = interner.intern("name");
        let value = InternedString::Symbol(symbol);

        assert_eq!(value.as_raw_str(), None);
        assert_eq!(value.as_symbol(), Some(symbol));
        assert_eq!(value.resolve_with_interner(&interner), Some("name"));
        assert_eq!(value.canonical_text().as_ref(), "symbol:1");
        assert_eq!(
            value
                .resolve_text(|candidate| Some(if candidate == symbol { "name" } else { "other" }))
                .as_deref(),
            Some("name")
        );
        assert_eq!(value.intern_with(&mut interner), symbol);
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

        let symbol = value.intern_with(&mut interner);

        assert_eq!(interner.resolve(symbol), Some("name"));
    }
}
