//! Sealed test doors over this module's private members.
use super::*;

impl super::super::WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn entity_kind_for_test(&self, entity: &str) -> Option<KindId> {
        Self::entity_kind(self, entity)
    }
}
