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

impl WorthQueryRetiredProductOccurrence {
    #[doc(hidden)]
    pub fn branch_for_test(&self) -> &worth_runtime_world::facade::ProductBranchIdentity {
        Self::branch(self)
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn replace_owner_cleanup_after_install_hook_for_test(&self, hook: Option<Box<dyn Fn()>>) {
        Self::replace_owner_cleanup_after_install_hook(self, hook)
    }
}

impl WorthQueryProductRuntime {
    #[doc(hidden)]
    pub fn reserve_owner_cleanup_for_creation_for_test(
        &self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        Self::reserve_owner_cleanup_for_creation(self)
    }
}
