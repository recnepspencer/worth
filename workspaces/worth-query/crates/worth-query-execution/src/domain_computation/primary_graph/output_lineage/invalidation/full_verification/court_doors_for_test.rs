//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;

impl SourceInvalidationOwner {
    #[doc(hidden)]
    pub fn capture_full_verification_for_test<'selected>(
        &'selected self,
        runtime: &'selected RelationalRuntime,
        snapshot: &'selected SnapshotHandle,
        selected: &'selected PositionedRelationalSnapshot,
    ) -> Result<FullVerificationImage<'selected>, FullVerificationStop> {
        Self::capture_full_verification(self, runtime, snapshot, selected)
    }
}

impl FullVerificationImage<'_> {
    #[doc(hidden)]
    pub fn verify_root_for_test(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &Arc<RecordedSettlementIdentity>,
        fallback_witness: Option<&SealedNativeOutputWitness>,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        Self::verify_root(self, request_execution, identity, fallback_witness)
    }
}
