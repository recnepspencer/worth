//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn default_branch_for_test(&self) -> &ProductBranchIdentity {
        Self::default_branch(self)
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn owner_for_test(&self) -> &Arc<RuntimeWorldOwner<(), (), (), (), ()>> {
        &self.owner
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn owner_cleanup_for_test(&self) -> &WorthQueryProductBranchOwnerCleanupRegistry {
        &self.owner_cleanup
    }
}
