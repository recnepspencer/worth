//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRealizedProjectionScope {
    #[doc(hidden)]
    pub fn contains_for_test(&self, entity_id: EntityId) -> bool {
        Self::contains(self, entity_id)
    }
}
