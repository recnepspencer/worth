//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Entity> WorthQueryApplicationEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn entity_id_for_test(&self) -> EntityId {
        Self::entity_id(self)
    }
}
