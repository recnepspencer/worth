use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::{ClientKeySymbolPolicy, InternedString, StringInterner, Symbol};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientKey(InternedString);

impl ClientKey {
    pub fn raw(value: impl Into<String>) -> Self {
        Self(InternedString::raw(value))
    }

    pub const fn symbol(value: Symbol) -> Self {
        Self(InternedString::Symbol(value))
    }

    pub fn as_raw_str(&self) -> Option<&str> {
        self.0.as_raw_str()
    }

    pub fn as_symbol(&self) -> Option<Symbol> {
        self.0.as_symbol()
    }

    pub fn normalize_with(
        self,
        interner: &mut StringInterner,
        policy: ClientKeySymbolPolicy,
    ) -> Self {
        if policy.interns_requested_strings() {
            Self(interner.normalize(self.0))
        } else {
            self
        }
    }

    pub fn intern_with(&self, interner: &mut StringInterner) -> Symbol {
        self.0.intern_with(interner)
    }

    pub fn resolve_with_interner<'a>(&'a self, interner: &'a StringInterner) -> Option<&'a str> {
        self.0.resolve_with_interner(interner)
    }

    pub fn canonical_text(&self) -> Cow<'_, str> {
        self.0.canonical_text()
    }

    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.0.owned_allocation_capacity_bytes()
    }
}

impl From<&str> for ClientKey {
    fn from(value: &str) -> Self {
        Self::raw(value)
    }
}

impl From<String> for ClientKey {
    fn from(value: String) -> Self {
        Self::raw(value)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientKey;
    use crate::symbols::data::{ClientKeySymbolPolicy, StringInterner, Symbol};

    #[test]
    fn raw_client_keys_expose_raw_surface() {
        let key = ClientKey::raw("edge-a");

        assert_eq!(key.as_raw_str(), Some("edge-a"));
        assert_eq!(key.as_symbol(), None);
        assert_eq!(key.canonical_text().as_ref(), "edge-a");
    }

    #[test]
    fn client_keys_normalize_through_policy_when_requested() {
        let mut interner = StringInterner::default();
        let key = ClientKey::raw("edge-a");

        let normalized = key.normalize_with(&mut interner, ClientKeySymbolPolicy::PreferInterned);

        assert_eq!(normalized.as_symbol(), Some(Symbol(1)));
        assert_eq!(normalized.resolve_with_interner(&interner), Some("edge-a"));
    }

    #[test]
    fn disabled_policy_preserves_raw_client_keys() {
        let mut interner = StringInterner::default();
        let key = ClientKey::raw("edge-a");

        let normalized = key.normalize_with(&mut interner, ClientKeySymbolPolicy::Disabled);

        assert_eq!(normalized.as_raw_str(), Some("edge-a"));
        assert_eq!(interner.resolve(Symbol(1)), None);
    }
}
