//! Ordinary currentness and native observational candidate entry points.
use super::*;
impl ConsumedOutputEvidence {
    /// The source owner decides whether this settlement is clean, which exact
    /// fact ordinals are dirty, or whether retained evidence must be verified.
    /// Every actor lookup and fact comparison is paid from the caller's meter.
    /// The iterative worklist visits each exact settlement at most once.
    pub fn observe_candidate_at(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &Arc<RecordedSettlementIdentity>,
        source_facts: &crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        upstream: &[ConsumedOutputEvidence],
        verification_requirement: Option<FullVerificationReason>,
        native_output_witness: &Arc<OnceLock<SealedNativeOutputWitness>>,
        observed: &PositionedRelationalSnapshot,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
        historical: Option<&crate::domain_computation::primary_graph::output_lineage::SelectionCoordinate>,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        Self::observe_views(
            execution,
            std::iter::once(EvidenceView {
                identity,
                source_facts: Some(source_facts),
                upstream,
                verification_requirement,
                native_output_witness: Some(native_output_witness),
                selected_native_root: observed,
                historical,
            }),
            owner,
            runtime,
            basis,
            selected,
        )
    }

    pub fn verify_candidate_with_admission(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &Arc<RecordedSettlementIdentity>,
        source_facts: &crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        upstream: &[ConsumedOutputEvidence],
        verification_requirement: Option<FullVerificationReason>,
        observed: &PositionedRelationalSnapshot,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
        historical: Option<&crate::domain_computation::primary_graph::output_lineage::SelectionCoordinate>,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        Self::verify_views(
            execution,
            std::iter::once(EvidenceView {
                identity,
                source_facts: Some(source_facts),
                upstream,
                verification_requirement,
                native_output_witness: None,
                selected_native_root: observed,
                historical,
            }),
            owner,
            runtime,
            basis,
            selected,
        )
    }
}

mod court_doors_for_test;
