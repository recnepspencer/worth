//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRetiredProductOccurrence {
    #[doc(hidden)]
    pub fn new_for_test(
        branch: worth_runtime_world::facade::ProductBranchIdentity,
        incarnation: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> Self {
        Self::new(branch, incarnation)
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn pending_owner_cleanup_for_test(&self) -> Vec<WorthQueryProductBranchOwnerCleanup> {
        Self::pending_owner_cleanup(self)
    }
}
