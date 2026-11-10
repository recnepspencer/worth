use crate::evidence_identity::{WorthQueryEvidenceIdentityEncoder, WorthQueryEvidenceTag};
use crate::runtime::{
    WorthQueryContinuityMutationEvidence, WorthQueryContinuityOutcomeClass,
    WorthQueryExistingTruthBindingEvidence, WorthQueryMutationCausalityEvidence,
    WorthQueryMutationProvenanceEvidence, WorthQueryNamingMutationEvidence,
    WorthQuerySymbolicTargetReferenceEvidence,
};

pub(super) fn append_mutation_evidence_fields(
    encoder: WorthQueryEvidenceIdentityEncoder,
    existing_truth_binding_evidence: Option<&WorthQueryExistingTruthBindingEvidence>,
    symbolic_target_reference_evidence: Option<&WorthQuerySymbolicTargetReferenceEvidence>,
    naming_mutation_evidence: Option<&WorthQueryNamingMutationEvidence>,
    continuity_mutation_evidence: Option<&WorthQueryContinuityMutationEvidence>,
    causality_evidence: Option<&WorthQueryMutationCausalityEvidence>,
    provenance_evidence: Option<&WorthQueryMutationProvenanceEvidence>,
) -> WorthQueryEvidenceIdentityEncoder {
    let symbolic_target_resolved_entity_identity = symbolic_target_reference_evidence
        .map(WorthQuerySymbolicTargetReferenceEvidence::resolved_entity_identity)
        .map(|identity| identity.evidence_identity());
    let naming_resolved_target_entity_identity = naming_mutation_evidence
        .and_then(WorthQueryNamingMutationEvidence::resolved_target_entity_identity)
        .map(|identity| identity.evidence_identity());
    let continuity_resolved_target_entity_identity = continuity_mutation_evidence
        .and_then(WorthQueryContinuityMutationEvidence::resolved_target_entity_identity)
        .map(|identity| identity.evidence_identity());
    let existing_truth_resolved_entity_identity = existing_truth_binding_evidence
        .map(WorthQueryExistingTruthBindingEvidence::resolved_entity_identity)
        .map(|identity| identity.evidence_identity());
    let successor_authoritative_identities = continuity_mutation_evidence
        .map(|evidence| evidence.successor_authoritative_identities().to_vec())
        .unwrap_or_default();
    let provenance_outcome_class = provenance_evidence
        .and_then(|evidence| evidence.outcome_class().map(|value| format!("{value:?}")));
    let provenance_failure_class = provenance_evidence
        .and_then(|evidence| evidence.failure_class().map(|value| format!("{value:?}")));

    encoder
        .optional_shape(
            WorthQueryEvidenceTag::new("existing_truth_family"),
            existing_truth_binding_evidence.map(|evidence| evidence.family().as_str()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("existing_truth_authoritative_identity"),
            existing_truth_binding_evidence
                .map(WorthQueryExistingTruthBindingEvidence::authoritative_identity)
                .map(|identity| identity.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("existing_truth_resolved_entity_identity"),
            existing_truth_resolved_entity_identity.as_ref(),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("existing_truth_target_collection"),
            existing_truth_binding_evidence
                .and_then(WorthQueryExistingTruthBindingEvidence::target_collection)
                .map(|collection| collection.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("existing_truth_binding_digest"),
            existing_truth_binding_evidence
                .map(WorthQueryExistingTruthBindingEvidence::binding_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("symbolic_target_symbol"),
            symbolic_target_reference_evidence
                .map(WorthQuerySymbolicTargetReferenceEvidence::symbol)
                .map(|symbol| symbol.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("symbolic_target_resolved_entity_identity"),
            symbolic_target_resolved_entity_identity.as_ref(),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("symbolic_target_collection"),
            symbolic_target_reference_evidence
                .and_then(WorthQuerySymbolicTargetReferenceEvidence::target_collection)
                .map(|collection| collection.evidence_identity()),
        )
        .optional_shape(
            WorthQueryEvidenceTag::new("naming_family"),
            naming_mutation_evidence.map(|evidence| evidence.family().as_str()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("naming_attachment_identity"),
            naming_mutation_evidence
                .map(WorthQueryNamingMutationEvidence::attachment_identity)
                .map(|identity| identity.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("naming_prior_authoritative_identity"),
            naming_mutation_evidence
                .and_then(WorthQueryNamingMutationEvidence::prior_authoritative_identity)
                .map(|identity| identity.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("naming_target_authoritative_identity"),
            naming_mutation_evidence
                .and_then(WorthQueryNamingMutationEvidence::target_authoritative_identity)
                .map(|identity| identity.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("naming_resolved_target_entity_identity"),
            naming_resolved_target_entity_identity.as_ref(),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("naming_target_collection"),
            naming_mutation_evidence
                .and_then(WorthQueryNamingMutationEvidence::target_collection)
                .map(|collection| collection.evidence_identity()),
        )
        .optional_shape(
            WorthQueryEvidenceTag::new("naming_outcome"),
            naming_mutation_evidence.map(|evidence| evidence.outcome().as_str()),
        )
        .optional_shape(
            WorthQueryEvidenceTag::new("continuity_family"),
            continuity_mutation_evidence.map(|evidence| evidence.family().as_str()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("continuity_prior_authoritative_identity"),
            continuity_mutation_evidence
                .map(WorthQueryContinuityMutationEvidence::prior_authoritative_identity)
                .map(|identity| identity.evidence_identity()),
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("continuity_successor_authoritative_identity"),
            successor_authoritative_identities
                .iter()
                .map(|identity| identity.evidence_identity()),
        )
        .optional_shape(
            WorthQueryEvidenceTag::new("continuity_outcome_class"),
            continuity_mutation_evidence.map(continuity_outcome_label),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("continuity_basis_binding_digest"),
            continuity_mutation_evidence
                .and_then(WorthQueryContinuityMutationEvidence::basis_binding_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("continuity_resolved_target_entity_identity"),
            continuity_resolved_target_entity_identity.as_ref(),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("continuity_target_collection"),
            continuity_mutation_evidence
                .and_then(WorthQueryContinuityMutationEvidence::target_collection)
                .map(|collection| collection.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("continuity_lineage_digest"),
            continuity_mutation_evidence
                .map(WorthQueryContinuityMutationEvidence::lineage_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("continuity_resolution_digest"),
            continuity_mutation_evidence
                .map(WorthQueryContinuityMutationEvidence::continuity_resolution_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("causality_digest"),
            causality_evidence
                .map(WorthQueryMutationCausalityEvidence::causality_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("causality_truth_trigger_digest"),
            causality_evidence
                .map(WorthQueryMutationCausalityEvidence::truth_trigger_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("causality_route_digest"),
            causality_evidence
                .map(WorthQueryMutationCausalityEvidence::route_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("causality_evaluation_surface_digest"),
            causality_evidence
                .map(WorthQueryMutationCausalityEvidence::evaluation_surface_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("causality_truth_view_digest"),
            causality_evidence
                .map(WorthQueryMutationCausalityEvidence::truth_view_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("provenance_contract_digest"),
            provenance_evidence
                .map(WorthQueryMutationProvenanceEvidence::contract_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("provenance_writeback_effect_artifact_digest"),
            provenance_evidence
                .map(WorthQueryMutationProvenanceEvidence::writeback_effect_artifact_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("provenance_effect_intent_digest"),
            provenance_evidence
                .map(WorthQueryMutationProvenanceEvidence::effect_intent_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("provenance_effect_intent_patch_canonical_basis"),
            provenance_evidence
                .map(WorthQueryMutationProvenanceEvidence::effect_intent_patch_canonical_basis)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("provenance_feedback_digest"),
            provenance_evidence
                .map(WorthQueryMutationProvenanceEvidence::feedback_provenance_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("provenance_causality_digest"),
            provenance_evidence
                .map(WorthQueryMutationProvenanceEvidence::causality_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("provenance_strategy_descriptor_digest"),
            provenance_evidence
                .map(WorthQueryMutationProvenanceEvidence::strategy_descriptor_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("provenance_execution_record_digest"),
            provenance_evidence
                .map(WorthQueryMutationProvenanceEvidence::execution_record_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("provenance_authoritative_artifact_digest"),
            provenance_evidence
                .and_then(WorthQueryMutationProvenanceEvidence::authoritative_artifact_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("provenance_request_digest"),
            provenance_evidence
                .and_then(WorthQueryMutationProvenanceEvidence::request_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_evidence_identity(
            WorthQueryEvidenceTag::new("provenance_receipt_digest"),
            provenance_evidence
                .and_then(WorthQueryMutationProvenanceEvidence::receipt_digest)
                .map(|digest| digest.evidence_identity()),
        )
        .optional_shape(
            WorthQueryEvidenceTag::new("provenance_outcome_class"),
            provenance_outcome_class.as_deref(),
        )
        .optional_shape(
            WorthQueryEvidenceTag::new("provenance_failure_class"),
            provenance_failure_class.as_deref(),
        )
}

fn continuity_outcome_label(evidence: &WorthQueryContinuityMutationEvidence) -> &'static str {
    match evidence.outcome_class() {
        WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor => {
            "continues_as_single_successor"
        }
        WorthQueryContinuityOutcomeClass::ContinuesAsSplitSuccessors => {
            "continues_as_split_successors"
        }
        WorthQueryContinuityOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor => {
            "continues_via_truth_lowered_canonical_merge_successor"
        }
        WorthQueryContinuityOutcomeClass::RejectedNoAuthoritativeSuccessor => {
            "rejected_no_authoritative_successor"
        }
        WorthQueryContinuityOutcomeClass::RejectedAmbiguousSuccessor => {
            "rejected_ambiguous_successor"
        }
        WorthQueryContinuityOutcomeClass::RejectedUnsupportedContinuityClass => {
            "rejected_unsupported_continuity_class"
        }
        WorthQueryContinuityOutcomeClass::RejectedHistoricalResolutionFailure => {
            "rejected_historical_resolution_failure"
        }
    }
}
