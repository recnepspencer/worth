//! Sealed test doors over this module's private members.
use super::*;

impl AggregateKey {
    #[doc(hidden)]
    pub fn member_for_test(&self) -> EntityId {
        Self::member(self)
    }
}
