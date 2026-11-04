//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRestoredAcceptedOutput {
    #[doc(hidden)]
    pub fn checkpoint_for_test(&self) -> &super::super::WorthQueryAcceptedOutputCheckpointIdentity {
        &self.checkpoint
    }
}

impl WorthQueryRestoredAcceptedOutput {
    #[doc(hidden)]
    pub fn source_scope_for_test(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding {
        &self.source_scope
    }
}

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn admit_restored_for_test(
        &self,
        requested_key: WorthQueryOutputDemandKey,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        admission_kind: super::super::DemandAdmissionKind,
        restored: super::super::WorthQueryRestoredAcceptedOutput,
        settlement: &std::sync::Arc<RecordedSettlementIdentity>,
        performed_source: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
    ) -> Result<(WorthQueryOutputDemandInterest, bool), WorthQueryOutputDemandDenial> {
        Self::admit_restored(
            self,
            requested_key,
            source_scope,
            product_occurrence,
            admission_kind,
            restored,
            settlement,
            performed_source,
        )
    }
}
