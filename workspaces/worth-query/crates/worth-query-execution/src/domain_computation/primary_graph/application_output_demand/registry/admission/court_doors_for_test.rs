//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryOutputDemandRegistry {
    #[doc(hidden)]
    pub fn admit_for_test(
        &self, requested_key: WorthQueryOutputDemandKey, selected_output: Option<SelectedOutputAdmission<'_>>, source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding, product_occurrence: worth_runtime_world::facade::ProductBranchIncarnation, admission_kind: DemandAdmissionKind, expected_source_commit: Option<&worth_runtime_world::facade::CompositeCommitIdentity>, successor_of: Option<OutputRefreshPredecessor<'_>>) -> Result<
        (
            WorthQueryOutputDemandInterest,
            Option<super::super::join_observation::JoinLatch>,
        ),
        WorthQueryOutputDemandDenial,
    > {
        Self::admit(self, requested_key, selected_output, source_scope, product_occurrence, admission_kind, expected_source_commit, successor_of)
    }
}
