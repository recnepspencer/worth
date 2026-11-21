//! Sealed test doors over this module's private members.
use super::*;

impl<Query> WorthQueryObservedResultSet<Query> {
    #[doc(hidden)]
    pub fn idempotency_identity_for_test(&self) -> [u8; 32] {
        Self::idempotency_identity(self)
    }
}
