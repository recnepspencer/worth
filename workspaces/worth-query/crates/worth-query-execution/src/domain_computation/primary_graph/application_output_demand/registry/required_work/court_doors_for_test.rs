//! Sealed test doors over this module's private members.
use super::*;

impl RequiredWorkMembership {
    #[doc(hidden)]
    pub fn new_for_test(
        key: Arc<WorthQueryOutputDemandKey>,
        queue: &Arc<RequiredWorkQueue>,
    ) -> Self {
        Self::new(key, queue)
    }
}

impl RequiredWorkMembership {
    #[doc(hidden)]
    pub fn set_required_for_test(self: &Arc<Self>, required: bool) {
        Self::set_required(self, required)
    }
}

impl RequiredWorkMembership {
    #[doc(hidden)]
    pub fn key_for_test(&self) -> &WorthQueryOutputDemandKey {
        Self::key(self)
    }
}

impl SelectedRequiredWork {
    #[doc(hidden)]
    pub fn key_for_test(&self) -> &WorthQueryOutputDemandKey {
        Self::key(self)
    }
}

impl SelectedRequiredWork {
    #[doc(hidden)]
    pub fn kind_for_test(&self) -> &SelectedRequiredWorkKind {
        Self::kind(self)
    }
}

impl RequiredWorkMembership {
    #[doc(hidden)]
    pub fn mark_local_required_for_test(
        self: &Arc<Self>,
        settlement: Arc<RecordedSettlementIdentity>,
    ) -> ReplacedRequiredWorkHint {
        Self::mark_local_required(self, settlement)
    }
}

impl RequiredWorkMembership {
    #[doc(hidden)]
    pub fn prepare_hint_for_test(self: &Arc<Self>, hint: Box<NativeHint>) {
        Self::prepare_hint(self, hint)
    }
}

impl super::super::DemandRegistryState {
    #[doc(hidden)]
    pub fn fixture_page_required_discontinuity_for_test(
        &mut self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        branch: &worth_relational::facade::history::BranchId,
        epoch: worth_relational::facade::history::CommitId,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        Self::fixture_page_required_discontinuity(self, occurrence, branch, epoch)
    }
}
