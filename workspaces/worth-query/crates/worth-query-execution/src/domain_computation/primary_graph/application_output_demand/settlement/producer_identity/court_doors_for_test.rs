//! Sealed test doors over this module's private members.
use super::*;

impl SettlementProducerIdentity {
    #[doc(hidden)]
    pub fn as_str_for_test(&self) -> &str {
        Self::as_str(self)
    }
}
