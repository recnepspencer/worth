use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientKeySymbolPolicy {
    Disabled,
    PreferInterned,
    RequireInterned,
}

impl ClientKeySymbolPolicy {
    pub const fn allows_raw_strings(self) -> bool {
        matches!(self, Self::Disabled)
    }

    pub const fn interns_requested_strings(self) -> bool {
        matches!(self, Self::PreferInterned | Self::RequireInterned)
    }

    pub const fn requires_interned_strings(self) -> bool {
        matches!(self, Self::RequireInterned)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientKeySymbolPolicy;

    #[test]
    fn client_key_symbol_policy_semantics_match_declared_variants() {
        assert!(ClientKeySymbolPolicy::Disabled.allows_raw_strings());
        assert!(!ClientKeySymbolPolicy::Disabled.interns_requested_strings());
        assert!(!ClientKeySymbolPolicy::Disabled.requires_interned_strings());

        assert!(!ClientKeySymbolPolicy::PreferInterned.allows_raw_strings());
        assert!(ClientKeySymbolPolicy::PreferInterned.interns_requested_strings());
        assert!(!ClientKeySymbolPolicy::PreferInterned.requires_interned_strings());

        assert!(!ClientKeySymbolPolicy::RequireInterned.allows_raw_strings());
        assert!(ClientKeySymbolPolicy::RequireInterned.interns_requested_strings());
        assert!(ClientKeySymbolPolicy::RequireInterned.requires_interned_strings());
    }
}
