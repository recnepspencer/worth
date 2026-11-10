use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::preview::PreviewSessionBindingTuple;
use crate::workflow::{
    workflow_canonical_query_digest_evidence, workflow_validated_query_digest_evidence,
};
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

pub(in crate::preview) fn compose_preview_session_binding_tuple_digest(
    binding: &PreviewSessionBindingTuple,
) -> Result<String, IdentityDerivationDenial> {
    let mut encoder =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_preview_session_binding_tuple_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("canonical_query"),
                &workflow_canonical_query_digest_evidence(binding.canonical_query_digest()),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("canonical_result_shape"),
                binding.canonical_result_shape_digest().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("validated_query"),
                &workflow_validated_query_digest_evidence(binding.validated_query_digest()),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("validated_result_shape"),
                binding.validated_result_shape_digest().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("evaluation_class"),
                binding.evaluation_class().as_str(),
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("preview_session"),
                &binding
                    .preview_session_identity()
                    .bridge_admission_evidence(),
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("declaration_identity"),
                &binding.declaration_identity().bridge_admission_evidence(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("declaration_digest"),
                binding.declaration_digest(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lifecycle"),
                super::preview_lifecycle_identity::preview_lifecycle_state_label(
                    binding.lifecycle_state_kind(),
                ),
            )
            .field_shape(WorthQueryEvidenceTag::new("replay_bundle"), "none")
            .field_shape(WorthQueryEvidenceTag::new("promotion_record"), "none")
            .field_shape(WorthQueryEvidenceTag::new("promotion_proof"), "none");
    encoder = match binding.execution_record_identity() {
        Some(identity) => encoder.field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("execution_record"),
            &identity.bridge_admission_evidence(),
        ),
        None => encoder.field_shape(WorthQueryEvidenceTag::new("execution_record"), "none"),
    };
    Ok(encoder.seal()?.as_str().to_string())
}

pub(in crate::preview) fn compose_preview_binding_tuple_workflow_identity(
    binding_tuple: &PreviewSessionBindingTuple,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_preview_binding_tuple_identity_v1",
        )
        .field_shape(
            WorthQueryEvidenceTag::new("binding_digest"),
            binding_tuple.digest(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("canonical_query"),
            &workflow_canonical_query_digest_evidence(binding_tuple.canonical_query_digest()),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("validated_query"),
            &workflow_validated_query_digest_evidence(binding_tuple.validated_query_digest()),
        )
        .seal()
}

pub(in crate::preview) fn compose_preview_declaration_digest_workflow_identity(
    binding_tuple: &PreviewSessionBindingTuple,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_preview_declaration_digest_identity_v1",
        )
        .field_bridge_retained_evidence_identity(
            WorthQueryEvidenceTag::new("declaration"),
            &binding_tuple
                .declaration_identity()
                .bridge_admission_evidence(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("declaration_digest"),
            binding_tuple.declaration_digest(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("canonical_query"),
            &workflow_canonical_query_digest_evidence(binding_tuple.canonical_query_digest()),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("validated_query"),
            &workflow_validated_query_digest_evidence(binding_tuple.validated_query_digest()),
        )
        .seal()
}
