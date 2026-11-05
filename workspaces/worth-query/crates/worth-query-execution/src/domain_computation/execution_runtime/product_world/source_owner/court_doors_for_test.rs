//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRelationalSourceOwner {
    #[doc(hidden)]
    pub fn invalidation_owner_for_test(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::SourceInvalidationOwner> {
        &self.invalidation_owner
    }
}
