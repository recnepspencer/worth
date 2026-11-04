//! Sealed test doors over this module's private members.
use super::*;

impl ConsumedOutputEvidence {
    #[doc(hidden)]
    pub fn own_evidence_is_current_for_test(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        facts: &(impl FactSequence + ?Sized),
        consumed: &[ConsumedOutputEvidence],
        witness: &SealedNativeOutputWitness,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, ConsumedOutputVerificationStop> {
        Self::own_evidence_is_current(execution, facts, consumed, witness, runtime, snapshot)
    }
}

impl ConsumedOutputEvidence {
    #[doc(hidden)]
    pub fn verify_at_observation_for_test(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &Arc<RecordedSettlementIdentity>,
        source_facts: &crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        upstream: &[ConsumedOutputEvidence],
        verification_requirement: Option<FullVerificationReason>,
        native_output_witness: &Arc<OnceLock<SealedNativeOutputWitness>>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        Self::verify_at_observation(
            execution,
            identity,
            source_facts,
            upstream,
            verification_requirement,
            native_output_witness,
            owner,
            runtime,
            snapshot,
            selected,
        )
    }
}
