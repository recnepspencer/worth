//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRelationalSourceOwner {
    #[doc(hidden)]
    pub fn invalidation_owner_for_test(
        &self,
    ) -> &Arc<dyn crate::ports::RelationalSourceInvalidationPort> {
        &self.invalidation_owner
    }
}
