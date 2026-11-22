//! Sealed test doors over this module's private members.
use super::*;

impl ConsumedOutputEvidence {
    #[doc(hidden)]
    pub fn verify_candidate_with_admission_for_test(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &Arc<RecordedSettlementIdentity>,
        source_facts: &crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        upstream: &[ConsumedOutputEvidence],
        verification_requirement: Option<FullVerificationReason>,
        observed: &PositionedRelationalSnapshot,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        Self::verify_candidate_with_admission(
            execution,
            identity,
            source_facts,
            upstream,
            verification_requirement,
            observed,
            owner,
            runtime,
            snapshot,
            selected,
        )
    }
}
