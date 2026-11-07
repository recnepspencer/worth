use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryAuthorityLane, WorthQueryBatchMutationEvidence,
    WorthQueryDerivedMaterializationTarget, WorthQueryGraphCompositionBreadth,
    WorthQueryGraphCompositionLifecycleOutcomes, WorthQueryGraphCompositionProgram,
    WorthQueryGraphCompositionResolutionMap, WorthQueryIntentDecisionTraceEnvelope,
    WorthQueryIntentExecutionProvenance, WorthQueryJournalPosition, WorthQueryLiveArtifactTarget,
    WorthQueryRuntimeError,
};

use super::batch_receipt_aggregates::{
    batch_bridge_evidence_from_receipts, derive_batch_receipt_aggregates,
};
use super::batch_receipt_identity::{
    evidence_value_identities, terminal_touch_projection_identities,
};
use super::WorthQueryWriteReceipt;

mod graph_composition_accessors;
mod terminal_affected_view_accessors;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBatchWriteReceipt {
    write_receipts: Vec<WorthQueryWriteReceipt>,
    authority_lane: WorthQueryAuthorityLane,
    basis_lane: WorthQueryAuthorityLane,
    batch_mutation_evidence: WorthQueryBatchMutationEvidence,
    graph_composition_breadth: WorthQueryGraphCompositionBreadth,
    graph_composition_program: WorthQueryGraphCompositionProgram,
    graph_composition_resolution_map: WorthQueryGraphCompositionResolutionMap,
    batch_digest: WorthQueryEvidenceIdentity,
    touched_aspects: Vec<WorthQueryAspectTouch>,
    affected_live_view_targets: Vec<WorthQueryLiveArtifactTarget>,
    affected_derived_view_targets: Vec<WorthQueryDerivedMaterializationTarget>,
    considered_computed_view_count: usize,
    considered_effect_count: usize,
    delivered_effect_count: usize,
    pending_write_intent_count: usize,
    suppressed_effect_count: usize,
    meaningful_effect_suppression_count: usize,
    effect_expression_failure_count: usize,
    refresh_fallback: bool,
    decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
    execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
}

impl WorthQueryBatchWriteReceipt {
    pub(in crate::runtime) fn new(
        write_receipts: Vec<WorthQueryWriteReceipt>,
        authority_lane: WorthQueryAuthorityLane,
        basis_lane: WorthQueryAuthorityLane,
        graph_composition_breadth: WorthQueryGraphCompositionBreadth,
        graph_composition_program: WorthQueryGraphCompositionProgram,
        touched_aspects: Vec<WorthQueryAspectTouch>,
        affected_live_view_targets: Vec<WorthQueryLiveArtifactTarget>,
        affected_derived_view_targets: Vec<WorthQueryDerivedMaterializationTarget>,
        considered_computed_view_count: usize,
        considered_effect_count: usize,
        delivered_effect_count: usize,
        pending_write_intent_count: usize,
        suppressed_effect_count: usize,
        meaningful_effect_suppression_count: usize,
        effect_expression_failure_count: usize,
        refresh_fallback: bool,
        decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
        execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
    ) -> Result<Self, WorthQueryRuntimeError> {
        if write_receipts.is_empty() {
            return Err(WorthQueryRuntimeError::Workspace(
                crate::memory_workspace::WorthQueryWorkspaceError::new(
                    "mutation batch must produce at least one write receipt",
                ),
            ));
        }
        if write_receipts
            .iter()
            .any(|receipt| receipt.authority_lane() != authority_lane)
        {
            return Err(WorthQueryRuntimeError::Workspace(
                crate::memory_workspace::WorthQueryWorkspaceError::new(
                    "mutation batch may not mix authority lanes",
                ),
            ));
        }

        let graph_lifecycle_digest =
            WorthQueryGraphCompositionLifecycleOutcomes::derive(&graph_composition_program)
                .map(|outcomes| outcomes.lifecycle_evidence_digest().clone());
        let journal_position_identities = write_receipts
            .iter()
            .map(WorthQueryWriteReceipt::journal_position)
            .map(WorthQueryJournalPosition::evidence_identity)
            .collect::<Vec<_>>();
        let batch_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::BatchWriteReceipt)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("graph_breadth_digest"),
                    graph_composition_breadth.breadth_evidence_digest(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("graph_program_digest"),
                    graph_composition_program.program_evidence_digest(),
                )
                .optional_evidence_identity(
                    WorthQueryEvidenceTag::new("graph_lifecycle_digest"),
                    graph_lifecycle_digest.as_ref(),
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("write_commit_identity"),
                    write_receipts
                        .iter()
                        .map(WorthQueryWriteReceipt::commit_evidence_identity),
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("journal_position_identity"),
                    journal_position_identities.iter(),
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("touched_aspect"),
                    terminal_touch_projection_identities("batch-touched-aspect", &touched_aspects)
                        .iter(),
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("affected_live_view_id"),
                    evidence_value_identities(
                        "batch-affected-live-view",
                        &terminal_live_view_ids(&affected_live_view_targets),
                    )
                    .iter(),
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("affected_derived_view_id"),
                    evidence_value_identities(
                        "batch-affected-derived-view",
                        &terminal_derived_view_ids(&affected_derived_view_targets),
                    )
                    .iter(),
                )
                .seal()?;

        let target_evidence = write_receipts
            .iter()
            .map(|receipt| receipt.target_evidence().clone())
            .collect::<Vec<_>>();
        let mutation_families = write_receipts
            .iter()
            .map(WorthQueryWriteReceipt::mutation_family)
            .collect::<Vec<_>>();
        let existing_truth_assertions = write_receipts
            .iter()
            .map(|receipt| receipt.existing_truth_assertion_evidence().cloned())
            .collect::<Vec<_>>();
        let existing_truth_bindings = write_receipts
            .iter()
            .map(|receipt| receipt.existing_truth_binding_evidence().cloned())
            .collect::<Vec<_>>();
        let symbolic_target_references = write_receipts
            .iter()
            .map(|receipt| receipt.symbolic_target_reference_evidence().cloned())
            .collect::<Vec<_>>();
        let symbolic_aspect_resolutions = write_receipts
            .iter()
            .map(|receipt| receipt.symbolic_aspect_resolution_evidence().to_vec())
            .collect::<Vec<_>>();
        let naming_mutations = write_receipts
            .iter()
            .map(|receipt| receipt.naming_mutation_evidence().cloned())
            .collect::<Vec<_>>();
        let continuity_mutations = write_receipts
            .iter()
            .map(|receipt| receipt.continuity_mutation_evidence().cloned())
            .collect::<Vec<_>>();
        let causality_evidence = write_receipts
            .iter()
            .map(|receipt| receipt.causality_evidence().cloned())
            .collect::<Vec<_>>();
        let provenance_evidence = write_receipts
            .iter()
            .map(|receipt| receipt.provenance_evidence().cloned())
            .collect::<Vec<_>>();
        let aggregate_bridge = batch_bridge_evidence_from_receipts(&write_receipts);
        let batch_mutation_evidence = WorthQueryBatchMutationEvidence::from_components(
            &mutation_families,
            &target_evidence,
            &existing_truth_assertions,
            &existing_truth_bindings,
            &symbolic_target_references,
            &symbolic_aspect_resolutions,
            &naming_mutations,
            &continuity_mutations,
            &causality_evidence,
            &provenance_evidence,
            aggregate_bridge.as_ref(),
        )?
        .expect("non-empty mutation batch must produce batch evidence");
        let graph_composition_resolution_map =
            WorthQueryGraphCompositionResolutionMap::from_write_receipts(&write_receipts);

        Ok(Self {
            write_receipts,
            authority_lane,
            basis_lane,
            batch_mutation_evidence,
            graph_composition_breadth,
            graph_composition_program,
            graph_composition_resolution_map,
            batch_digest,
            touched_aspects,
            affected_live_view_targets,
            affected_derived_view_targets,
            considered_computed_view_count,
            considered_effect_count,
            delivered_effect_count,
            pending_write_intent_count,
            suppressed_effect_count,
            meaningful_effect_suppression_count,
            effect_expression_failure_count,
            refresh_fallback,
            decision_trace_envelope,
            execution_provenance,
        })
    }

    pub fn from_write_receipts(
        write_receipts: Vec<WorthQueryWriteReceipt>,
    ) -> Result<Self, WorthQueryRuntimeError> {
        if write_receipts.is_empty() {
            return Err(WorthQueryRuntimeError::Workspace(
                crate::memory_workspace::WorthQueryWorkspaceError::new(
                    "mutation batch must produce at least one write receipt",
                ),
            ));
        }
        let authority_lane = write_receipts[0].authority_lane();
        let basis_lane = write_receipts[0].basis_lane();
        if write_receipts
            .iter()
            .any(|receipt| receipt.authority_lane() != authority_lane)
        {
            return Err(WorthQueryRuntimeError::Workspace(
                crate::memory_workspace::WorthQueryWorkspaceError::new(
                    "mutation batch may not mix authority lanes",
                ),
            ));
        }
        if write_receipts
            .iter()
            .any(|receipt| receipt.basis_lane() != basis_lane)
        {
            return Err(WorthQueryRuntimeError::Workspace(
                crate::memory_workspace::WorthQueryWorkspaceError::new(
                    "mutation batch may not mix basis lanes",
                ),
            ));
        }

        let aggregates = derive_batch_receipt_aggregates(&write_receipts);

        Self::new(
            write_receipts,
            authority_lane,
            basis_lane,
            WorthQueryGraphCompositionBreadth::empty(),
            WorthQueryGraphCompositionProgram::empty()?,
            aggregates.touched_aspects,
            aggregates.affected_live_view_targets,
            aggregates.affected_derived_view_targets,
            aggregates.considered_computed_view_count,
            aggregates.considered_effect_count,
            aggregates.delivered_effect_count,
            aggregates.pending_write_intent_count,
            aggregates.suppressed_effect_count,
            aggregates.meaningful_effect_suppression_count,
            aggregates.effect_expression_failure_count,
            aggregates.refresh_fallback,
            None,
            None,
        )
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn basis_lane(&self) -> WorthQueryAuthorityLane {
        self.basis_lane
    }

    pub fn batch_digest(&self) -> &str {
        self.batch_digest.as_str()
    }

    pub(in crate::runtime) fn batch_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.batch_digest
    }

    pub fn batch_mutation_evidence(&self) -> &WorthQueryBatchMutationEvidence {
        &self.batch_mutation_evidence
    }

    pub fn graph_composition_breadth(&self) -> &WorthQueryGraphCompositionBreadth {
        &self.graph_composition_breadth
    }

    pub fn graph_composition_program(&self) -> Option<&WorthQueryGraphCompositionProgram> {
        (!self.graph_composition_program.is_empty()).then_some(&self.graph_composition_program)
    }

    pub fn write_count(&self) -> usize {
        self.write_receipts.len()
    }

    pub fn write_receipts(&self) -> &[WorthQueryWriteReceipt] {
        &self.write_receipts
    }

    pub fn journal_positions(&self) -> impl Iterator<Item = &WorthQueryJournalPosition> {
        self.write_receipts
            .iter()
            .map(WorthQueryWriteReceipt::journal_position)
    }

    pub fn admitted_touched_aspects(&self) -> &[WorthQueryAspectTouch] {
        &self.touched_aspects
    }

    pub fn considered_computed_view_count(&self) -> usize {
        self.considered_computed_view_count
    }

    pub fn considered_effect_count(&self) -> usize {
        self.considered_effect_count
    }

    pub fn delivered_effect_count(&self) -> usize {
        self.delivered_effect_count
    }

    pub fn pending_write_intent_count(&self) -> usize {
        self.pending_write_intent_count
    }

    pub fn suppressed_effect_count(&self) -> usize {
        self.suppressed_effect_count
    }

    pub fn meaningful_effect_suppression_count(&self) -> usize {
        self.meaningful_effect_suppression_count
    }

    pub fn effect_expression_failure_count(&self) -> usize {
        self.effect_expression_failure_count
    }

    pub fn refresh_fallback(&self) -> bool {
        self.refresh_fallback
    }

    pub fn decision_trace_envelope(&self) -> Option<&WorthQueryIntentDecisionTraceEnvelope> {
        self.decision_trace_envelope.as_ref()
    }

    pub fn execution_provenance(&self) -> Option<&WorthQueryIntentExecutionProvenance> {
        self.execution_provenance.as_ref()
    }
}

pub(in crate::runtime) fn terminal_live_view_ids(
    targets: &[WorthQueryLiveArtifactTarget],
) -> Vec<String> {
    targets
        .iter()
        .map(|target| target.view_name().to_string())
        .collect()
}

pub(in crate::runtime) fn terminal_derived_view_ids(
    targets: &[WorthQueryDerivedMaterializationTarget],
) -> Vec<String> {
    targets
        .iter()
        .map(|target| target.view_name().to_string())
        .collect()
}
