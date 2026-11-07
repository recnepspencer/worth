use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryAspectMutationOperation, WorthQueryContinuityMutationEvidence,
    WorthQueryExistingTruthAssertionEvidence, WorthQueryExistingTruthBindingEvidence,
    WorthQueryMutationCausalityEvidence, WorthQueryMutationMetadata,
    WorthQueryMutationProvenanceEvidence, WorthQueryMutationTargetEvidence,
    WorthQueryNamingMutationEvidence, WorthQueryRuntimeInspectionEvidence,
    WorthQuerySymbolicTargetReferenceEvidence, WorthQueryWriteReceipt,
};

#[path = "digest_components.rs"]
mod digest_components;
#[path = "digest_mutation_evidence.rs"]
mod digest_mutation_evidence;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_write_receipt_inspection_digest(
    receipt: &WorthQueryWriteReceipt,
    target_evidence: &WorthQueryMutationTargetEvidence,
    existing_truth_assertion_evidence: Option<&WorthQueryExistingTruthAssertionEvidence>,
    existing_truth_binding_evidence: Option<&WorthQueryExistingTruthBindingEvidence>,
    symbolic_target_reference_evidence: Option<&WorthQuerySymbolicTargetReferenceEvidence>,
    naming_mutation_evidence: Option<&WorthQueryNamingMutationEvidence>,
    continuity_mutation_evidence: Option<&WorthQueryContinuityMutationEvidence>,
    causality_evidence: Option<&WorthQueryMutationCausalityEvidence>,
    provenance_evidence: Option<&WorthQueryMutationProvenanceEvidence>,
    runtime_evidence: &WorthQueryRuntimeInspectionEvidence,
    declared_aspect_operations: &[WorthQueryAspectMutationOperation],
    declared_aspect_value_digest: Option<&WorthQueryEvidenceIdentity>,
    mutation_metadata: &WorthQueryMutationMetadata,
    live_patch_artifacts: &[WorthQueryEvidenceIdentity],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let commit_identity = receipt.commit_identity().evidence_identity();
    let snapshot_identity = receipt.snapshot_identity().evidence_identity();
    let declared_entity_identity = receipt
        .declared_entity_identity()
        .map(|identity| identity.evidence_identity())
        .transpose()?;
    let target_entity_identity = receipt
        .target_entity_identity()
        .map(|identity| identity.evidence_identity())
        .transpose()?;
    let target_declared_entity_identity = target_evidence
        .declared()
        .entity_identity()
        .map(|identity| identity.evidence_identity())
        .transpose()?;
    let target_resolved_entity_identity = target_evidence
        .resolved()
        .entity_identity()
        .map(|identity| identity.evidence_identity())
        .transpose()?;
    let declared_operations =
        digest_components::declared_aspect_operation_identities(declared_aspect_operations);
    let mutation_metadata_entries =
        digest_components::mutation_metadata_entry_identities(mutation_metadata);

    let encoder =
        worth_query_evidence_identity(WorthQueryEvidenceScope::WriteReceiptInspectionArtifact)
            .field_shape(
                WorthQueryEvidenceTag::new("mutation_family"),
                receipt.mutation_family().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authority_lane"),
                receipt.authority_lane().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis_lane"),
                receipt.basis_lane().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("commit_identity"),
                &commit_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("snapshot_token"),
                &snapshot_identity,
            )
            .optional_value(
                WorthQueryEvidenceTag::new("declared_collection"),
                receipt.terminal_declared_collection_projection(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("declared_entity_identity"),
                declared_entity_identity.as_ref(),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("target_collection"),
                receipt.terminal_target_collection_projection(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("target_entity_identity"),
                target_entity_identity.as_ref(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("target_declared_class"),
                target_evidence.declared().target_class().as_str(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("target_declared_collection"),
                target_evidence
                    .declared()
                    .collection()
                    .map(|collection| collection.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("target_declared_entity_identity"),
                target_declared_entity_identity.as_ref(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("target_resolved_class"),
                target_evidence.resolved().target_class().as_str(),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("target_resolved_collection"),
                target_evidence
                    .resolved()
                    .collection()
                    .map(|collection| collection.evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("target_resolved_entity_identity"),
                target_resolved_entity_identity.as_ref(),
            )
            .optional_shape(
                WorthQueryEvidenceTag::new("existing_assertion_mode"),
                existing_truth_assertion_evidence.map(|evidence| evidence.mode().as_str()),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("existing_assertion_aspect_count"),
                existing_truth_assertion_evidence
                    .map_or(0, |evidence| evidence.asserted_aspect_count()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("existing_assertion_verification_digest"),
                existing_truth_assertion_evidence
                    .map(|evidence| evidence.verification_evidence_identity()),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("existing_assertion_assumption_snapshot_digest"),
                existing_truth_assertion_evidence.and_then(
                    WorthQueryExistingTruthAssertionEvidence::assumption_snapshot_evidence_digest,
                ),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("existing_assertion_verified_precondition_digest"),
                existing_truth_assertion_evidence.and_then(
                    WorthQueryExistingTruthAssertionEvidence::verified_precondition_evidence_digest,
                ),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("existing_assertion_read_set_breadth"),
                existing_truth_assertion_evidence
                    .and_then(
                        WorthQueryExistingTruthAssertionEvidence::verification_read_set_breadth,
                    )
                    .map(|breadth| breadth.counter_snapshot()),
            );

    Ok(digest_mutation_evidence::append_mutation_evidence_fields(
        encoder,
        existing_truth_binding_evidence,
        symbolic_target_reference_evidence,
        naming_mutation_evidence,
        continuity_mutation_evidence,
        causality_evidence,
        provenance_evidence,
    )?
    .field_shape(
        WorthQueryEvidenceTag::new("runtime_artifact_family"),
        runtime_evidence.artifact_family(),
    )
    .field_shape(
        WorthQueryEvidenceTag::new("runtime_authority_lane"),
        runtime_evidence.authority_lane().as_str(),
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("runtime_evidence"),
        runtime_evidence.evidence().iter().map(String::as_str),
    )
    .field_evidence_identity_sequence(
        WorthQueryEvidenceTag::new("declared_aspect_operation"),
        declared_operations.iter(),
    )
    .optional_evidence_identity(
        WorthQueryEvidenceTag::new("declared_aspect_value_digest"),
        declared_aspect_value_digest,
    )
    .field_evidence_identity_sequence(
        WorthQueryEvidenceTag::new("mutation_metadata"),
        mutation_metadata_entries.iter(),
    )
    .field_evidence_identity_sequence(
        WorthQueryEvidenceTag::new("live_patch_artifact"),
        live_patch_artifacts.iter(),
    )
    .seal()?)
}
