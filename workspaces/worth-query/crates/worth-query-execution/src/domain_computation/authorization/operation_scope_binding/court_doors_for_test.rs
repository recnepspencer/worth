//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOperationScopeEntityBinding {
    #[doc(hidden)]
    pub fn from_entity_for_test(entity: worth_relational::facade::identity::EntityId) -> Self {
        Self::from_entity(entity)
    }
}
