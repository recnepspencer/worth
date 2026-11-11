use super::WorthQueryBatchWriteComponentInspection;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQueryCommitIdentity;
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryBatchMutationEvidence, WorthQueryGraphCompositionBreadth,
    WorthQueryGraphCompositionEvidence, WorthQueryGraphCompositionLifecycleOutcomes,
    WorthQueryGraphCompositionProgram, WorthQueryGraphCompositionResolutionMap,
};

#[path = "batch_write_digest_components.rs"]
mod batch_write_digest_components;

pub(super) struct WorthQueryBatchWriteDigestInputs<'a> {
    pub authority_lane: &'a str,
    pub basis_lane: &'a str,
    pub batch_digest: &'a WorthQueryEvidenceIdentity,
    pub graph_composition_breadth: &'a WorthQueryGraphCompositionBreadth,
    pub graph_composition_lifecycle_outcomes:
        Option<&'a WorthQueryGraphCompositionLifecycleOutcomes>,
    pub graph_composition_program: Option<&'a WorthQueryGraphCompositionProgram>,
    pub graph_composition_evidence: Option<&'a WorthQueryGraphCompositionEvidence>,
    pub batch_mutation_evidence: &'a WorthQueryBatchMutationEvidence,
    pub commit_identities: &'a [WorthQueryCommitIdentity],
    pub journal_position_identities: &'a [WorthQueryEvidenceIdentity],
    pub component_operations: &'a [WorthQueryBatchWriteComponentInspection],
    pub graph_composition_resolution_map: &'a WorthQueryGraphCompositionResolutionMap,
    pub touched_aspects: &'a [WorthQueryAspectTouch],
    pub affected_live_view_ids: &'a [String],
    pub affected_derived_view_ids: &'a [String],
}

pub(super) fn build_batch_write_receipt_inspection_digest(
    inputs: WorthQueryBatchWriteDigestInputs<'_>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let component_artifact_identities =
        batch_write_digest_components::component_artifact_identities(inputs.component_operations)?;
    let graph_resolution_identities = batch_write_digest_components::graph_resolution_identities(
        inputs.graph_composition_resolution_map,
    )?;
    let commit_identities = inputs
        .commit_identities
        .iter()
        .map(|identity| identity.evidence_identity())
        .collect::<Vec<_>>();

    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::BatchWriteReceiptInspectionArtifact)
            .field_shape(
                WorthQueryEvidenceTag::new("authority_lane"),
                inputs.authority_lane,
            )
            .field_shape(WorthQueryEvidenceTag::new("basis_lane"), inputs.basis_lane)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("batch_digest"),
                inputs.batch_digest,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("graph_component_count"),
                inputs.graph_composition_breadth.component_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("graph_symbolic_entity_declaration_count"),
                inputs
                    .graph_composition_breadth
                    .symbolic_entity_declaration_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("graph_symbolic_relation_declaration_count"),
                inputs
                    .graph_composition_breadth
                    .symbolic_relation_declaration_count(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("graph_breadth_digest"),
                inputs.graph_composition_breadth.breadth_evidence_digest(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("graph_lifecycle_digest"),
                inputs
                    .graph_composition_lifecycle_outcomes
                    .map(WorthQueryGraphCompositionLifecycleOutcomes::lifecycle_evidence_digest),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("graph_lifecycle_counter_snapshot"),
                inputs
                    .graph_composition_lifecycle_outcomes
                    .map(WorthQueryGraphCompositionLifecycleOutcomes::counter_snapshot),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("graph_program_digest"),
                inputs
                    .graph_composition_program
                    .map(WorthQueryGraphCompositionProgram::program_evidence_digest),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("graph_composition_digest"),
                inputs
                    .graph_composition_evidence
                    .map(WorthQueryGraphCompositionEvidence::graph_composition_evidence_digest),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("graph_symbolic_resolution_digest"),
                inputs.graph_composition_evidence.map(
                    WorthQueryGraphCompositionEvidence::graph_symbolic_resolution_evidence_digest,
                ),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("graph_counter_snapshot"),
                inputs
                    .graph_composition_evidence
                    .map(WorthQueryGraphCompositionEvidence::counter_snapshot),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("graph_assumption_digest"),
                inputs
                    .graph_composition_evidence
                    .and_then(WorthQueryGraphCompositionEvidence::graph_assumption_evidence_digest),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("graph_lineage_digest"),
                inputs
                    .graph_composition_evidence
                    .and_then(WorthQueryGraphCompositionEvidence::graph_lineage_evidence_digest),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("graph_assumption_summary_counter"),
                inputs.graph_composition_evidence.and_then(|evidence| {
                    evidence
                        .assumption_summary()
                        .map(|summary| summary.counter_snapshot())
                }),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("graph_lineage_summary_counter"),
                inputs.graph_composition_evidence.and_then(|evidence| {
                    evidence
                        .lineage_summary()
                        .map(|summary| summary.counter_snapshot())
                }),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_component_count"),
                inputs.batch_mutation_evidence.component_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_target_evidence_count"),
                inputs.batch_mutation_evidence.target_evidence_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_existing_truth_assertion_count"),
                inputs
                    .batch_mutation_evidence
                    .existing_truth_assertion_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_retained_authoritative_assertion_count"),
                inputs
                    .batch_mutation_evidence
                    .retained_authoritative_assertion_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_backend_verified_assertion_count"),
                inputs
                    .batch_mutation_evidence
                    .backend_verified_assertion_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_backend_verified_update_count"),
                inputs
                    .batch_mutation_evidence
                    .backend_verified_update_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_backend_verified_delete_count"),
                inputs
                    .batch_mutation_evidence
                    .backend_verified_delete_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_existing_truth_binding_count"),
                inputs
                    .batch_mutation_evidence
                    .existing_truth_binding_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_symbolic_target_reference_count"),
                inputs
                    .batch_mutation_evidence
                    .symbolic_target_reference_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_symbolic_resolution_count"),
                inputs.batch_mutation_evidence.symbolic_resolution_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_naming_mutation_count"),
                inputs.batch_mutation_evidence.naming_mutation_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_continuity_mutation_count"),
                inputs.batch_mutation_evidence.continuity_mutation_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_resolved_target_count"),
                inputs.batch_mutation_evidence.resolved_target_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_target_collection_count"),
                inputs.batch_mutation_evidence.target_collection_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_target_entity_count"),
                inputs.batch_mutation_evidence.target_entity_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_causality_bundle_count"),
                inputs.batch_mutation_evidence.causality_bundle_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_provenance_bundle_count"),
                inputs.batch_mutation_evidence.provenance_bundle_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_outcome_class_count"),
                inputs.batch_mutation_evidence.outcome_class_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_authority_request_count"),
                inputs.batch_mutation_evidence.authority_request_count(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_authority_receipt_count"),
                inputs.batch_mutation_evidence.authority_receipt_count(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("batch_aggregate_target_digest"),
                inputs
                    .batch_mutation_evidence
                    .aggregate_target_digest()
                    .evidence_identity(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("batch_aggregate_existing_truth_assertion_digest"),
                inputs
                    .batch_mutation_evidence
                    .aggregate_existing_truth_assertion_digest()
                    .map(|digest| digest.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("batch_aggregate_existing_truth_mode_digest"),
                inputs
                    .batch_mutation_evidence
                    .aggregate_existing_truth_mode_digest()
                    .map(|digest| digest.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("batch_aggregate_continuity_mutation_digest"),
                inputs
                    .batch_mutation_evidence
                    .aggregate_continuity_mutation_digest()
                    .map(|digest| digest.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("batch_aggregate_existing_truth_binding_digest"),
                inputs
                    .batch_mutation_evidence
                    .aggregate_existing_truth_binding_digest()
                    .map(|digest| digest.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("batch_aggregate_symbolic_target_reference_digest"),
                inputs
                    .batch_mutation_evidence
                    .aggregate_symbolic_target_reference_digest()
                    .map(|digest| digest.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("batch_aggregate_symbolic_resolution_digest"),
                inputs
                    .batch_mutation_evidence
                    .aggregate_symbolic_resolution_digest()
                    .map(|digest| digest.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("batch_aggregate_naming_mutation_digest"),
                inputs
                    .batch_mutation_evidence
                    .aggregate_naming_mutation_digest()
                    .map(|digest| digest.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("batch_aggregate_causality_digest"),
                inputs
                    .batch_mutation_evidence
                    .aggregate_causality_digest()
                    .map(|digest| digest.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("batch_aggregate_provenance_digest"),
                inputs
                    .batch_mutation_evidence
                    .aggregate_provenance_digest()
                    .map(|digest| digest.evidence_identity()),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("commit_identity"),
                commit_identities.iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("journal_position_identity"),
                inputs.journal_position_identities.iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("component_artifact_identity"),
                component_artifact_identities.iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("graph_resolution_identity"),
                graph_resolution_identities.iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("touched_aspect"),
                terminal_touch_projection_identities(
                    "batch-inspection-touched-aspect",
                    inputs.touched_aspects,
                )?
                .iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("affected_live_view_id"),
                evidence_value_identities(
                    "batch-inspection-affected-live-view",
                    inputs.affected_live_view_ids,
                )?
                .iter(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("affected_derived_view_id"),
                evidence_value_identities(
                    "batch-inspection-affected-derived-view",
                    inputs.affected_derived_view_ids,
                )?
                .iter(),
            )
            .seal()?,
    )
}

fn evidence_value_identities(
    role: &'static str,
    values: &[String],
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    Ok(values
        .iter()
        .map(|value| {
            worth_query_evidence_identity(
                WorthQueryEvidenceScope::BatchWriteReceiptInspectionArtifact,
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_value(WorthQueryEvidenceTag::new("value"), value)
            .seal()
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn terminal_touch_projection_identities(
    role: &'static str,
    touches: &[WorthQueryAspectTouch],
) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    Ok(touches
        .iter()
        .map(|touch| {
            worth_query_evidence_identity(
                WorthQueryEvidenceScope::BatchWriteReceiptInspectionArtifact,
            )
            .field_shape(WorthQueryEvidenceTag::new("role"), role)
            .field_value(
                WorthQueryEvidenceTag::new("value"),
                touch.admitted_touch_digest_part(),
            )
            .seal()
        })
        .collect::<Result<Vec<_>, _>>()?)
}
