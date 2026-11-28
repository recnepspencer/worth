use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::{ClientKeySymbolPolicy, StringInterner, Symbol};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientKey(ClientKeyText);

// A request key carries only owned text or a symbol ID. Its admitted decode
// pays for that string alone, and its ordered-map meaning has no mutable seal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename = "InternedString")]
enum ClientKeyText {
    Raw(String),
    Symbol(Symbol),
}

impl ClientKey {
    pub fn raw(value: impl Into<String>) -> Self {
        Self(ClientKeyText::Raw(value.into()))
    }

    pub const fn symbol(value: Symbol) -> Self {
        Self(ClientKeyText::Symbol(value))
    }

    pub fn as_raw_str(&self) -> Option<&str> {
        match &self.0 {
            ClientKeyText::Raw(text) => Some(text),
            ClientKeyText::Symbol(_) => None,
        }
    }

    pub fn as_symbol(&self) -> Option<Symbol> {
        match self.0 {
            ClientKeyText::Raw(_) => None,
            ClientKeyText::Symbol(symbol) => Some(symbol),
        }
    }

    pub fn normalize_with(
        self,
        interner: &mut StringInterner,
        policy: ClientKeySymbolPolicy,
    ) -> Result<Self, super::RelationalSymbolExhausted> {
        if policy.interns_requested_strings() {
            self.intern_with(interner).map(Self::symbol)
        } else {
            Ok(self)
        }
    }

    pub fn intern_with(
        &self,
        interner: &mut StringInterner,
    ) -> Result<Symbol, super::RelationalSymbolExhausted> {
        match &self.0 {
            ClientKeyText::Raw(text) => interner.intern(text),
            ClientKeyText::Symbol(symbol) => Ok(*symbol),
        }
    }

    pub fn resolve_with_interner<'a>(&'a self, interner: &'a StringInterner) -> Option<&'a str> {
        self.as_raw_str()
            .or_else(|| self.as_symbol().and_then(|symbol| interner.resolve(symbol)))
    }

    pub fn canonical_text(&self) -> Cow<'_, str> {
        match &self.0 {
            ClientKeyText::Raw(text) => Cow::Borrowed(text),
            ClientKeyText::Symbol(symbol) => Cow::Owned(format!("symbol:{}", symbol.0)),
        }
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

/// Operations used by the Relational owners.
pub trait ClientKeyOwnerPort: Sized {
    fn owned_allocation_capacity_bytes(&self) -> u64;
}

impl ClientKeyOwnerPort for ClientKey {
    fn owned_allocation_capacity_bytes(&self) -> u64 {
        match &self.0 {
            ClientKeyText::Raw(text) => text.capacity() as u64,
            ClientKeyText::Symbol(_) => 0,
        }
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

        let normalized = key
            .normalize_with(&mut interner, ClientKeySymbolPolicy::PreferInterned)
            .expect("symbol fixture has identity capacity");

        assert_eq!(normalized.as_symbol(), Some(Symbol(1)));
        assert_eq!(normalized.resolve_with_interner(&interner), Some("edge-a"));
    }

    #[test]
    fn client_key_readmission_retains_only_the_quoted_raw_string() {
        let original = ClientKey::raw("allocated text");
        let encoded = rmp_serde::to_vec_named(&original).unwrap();
        let decoded: ClientKey = rmp_serde::from_slice(&encoded).unwrap();
        assert_eq!(decoded, original);
        assert!(matches!(decoded.0, super::ClientKeyText::Raw(_)));
    }

    #[test]
    fn disabled_policy_preserves_raw_client_keys() {
        let mut interner = StringInterner::default();
        let key = ClientKey::raw("edge-a");

        let normalized = key
            .normalize_with(&mut interner, ClientKeySymbolPolicy::Disabled)
            .expect("symbol fixture has identity capacity");

        assert_eq!(normalized.as_raw_str(), Some("edge-a"));
        assert_eq!(interner.resolve(Symbol(1)), None);
    }
}
