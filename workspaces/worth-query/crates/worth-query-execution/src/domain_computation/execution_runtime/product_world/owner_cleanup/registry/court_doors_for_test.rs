//! Sealed test doors over this module's private members.
use super::*;

impl CleanupEntry {
    #[doc(hidden)]
    pub fn record_for_test(&self) -> &Mutex<Option<WorthQueryProductBranchOwnerCleanupRecord>> {
        &self.record
    }
}

impl WorthQueryProductBranchOwnerCleanupRegistry {
    #[doc(hidden)]
    pub fn state_for_test(&self) -> &Arc<Mutex<CleanupRegistryState>> {
        &self.state
    }
}

impl WorthQueryProductBranchOwnerCleanupRegistry {
    #[doc(hidden)]
    pub fn new_for_test(maximum: Option<usize>) -> Self {
        Self::new(maximum)
    }
}

impl WorthQueryProductBranchOwnerCleanupRegistry {
    #[doc(hidden)]
    pub fn pending_application_retired_product_occurrences_for_test(
        &self,
    ) -> Vec<WorthQueryRetiredProductOccurrence> {
        Self::pending_application_retired_product_occurrences(self)
    }
}

impl WorthQueryProductBranchOwnerCleanupRegistry {
    #[doc(hidden)]
    pub fn replace_after_component_progress_hook_for_test(&self, hook: Option<Box<dyn Fn()>>) {
        Self::replace_after_component_progress_hook(self, hook)
    }
}

impl WorthQueryProductBranchOwnerCleanupRegistry {
    #[doc(hidden)]
    pub fn reserve_for_test(
        &self,
        scope: CleanupScope,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReservation,
        WorthQueryProductBranchOwnerCleanupCapacityExhausted,
    > {
        Self::reserve(self, scope)
    }
}

impl CleanupRegistryState {
    #[doc(hidden)]
    pub fn entries_for_test(&self) -> &BTreeMap<u64, Arc<CleanupEntry>> {
        &self.entries
    }
}

impl CleanupRegistryState {
    #[doc(hidden)]
    pub fn reserved_for_test(&self) -> &usize {
        &self.reserved
    }
}

impl WorthQueryProductBranchOwnerCleanupReservation {
    #[doc(hidden)]
    pub fn install_retirement_for_test(
        self,
        report: worth_runtime_world::facade::ProductBranchRetirementReport,
    ) -> u64 {
        Self::install_retirement(self, report)
    }
}
