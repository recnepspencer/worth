use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_runtime_bridge::facade::{
    BridgePreviewLifecycleStateKind, BridgePreviewSessionDeclarationIdentity,
    BridgePreviewSessionIdentity, PreviewExecutionRecordIdentity,
};

use crate::execution::ExecutionCounters;
use crate::identity::{CanonicalQueryDigest, ValidatedQueryDigest};
use crate::preview::{
    AdmittedPreviewWorkflowFoundation, PreviewBindingCounters, PreviewEvaluationClass,
    PreviewExecutionCounters, PreviewWorkflowFoundationArtifact, PreviewWorkflowFoundationError,
    PreviewWorkflowFoundationRequest,
};
use crate::workflow::{
    workflow_canonical_query_digest_evidence, workflow_validated_query_digest_evidence,
};
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

pub(crate) fn materialize_contributed_preview_workflow_foundation_artifact(
    binding_identity: WorthQueryEvidenceIdentity,
    canonical_query_digest: CanonicalQueryDigest,
    validated_query_digest: ValidatedQueryDigest,
    request_family: PreviewWorkflowFoundationRequest,
    preview_session_identity: BridgePreviewSessionIdentity,
    declaration_identity: BridgePreviewSessionDeclarationIdentity,
    declaration_digest_identity: WorthQueryEvidenceIdentity,
    evaluation_class: PreviewEvaluationClass,
    shape_check_width: usize,
) -> Result<PreviewWorkflowFoundationArtifact, IdentityDerivationDenial> {
    let lifecycle_state_kind = BridgePreviewLifecycleStateKind::Active;
    let execution_record_identity_basis =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_domain_preview_execution_record_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("binding"), &binding_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("canonical"),
                &workflow_canonical_query_digest_evidence(&canonical_query_digest),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("validated"),
                &workflow_validated_query_digest_evidence(&validated_query_digest),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("request"),
                request_family.as_str(),
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("preview_session"),
                &preview_session_identity.bridge_admission_evidence(),
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("declaration"),
                &declaration_identity.bridge_admission_evidence(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("evaluation"),
                evaluation_class.as_str(),
            )
            .seal()?;
    let execution_record_identity = PreviewExecutionRecordIdentity::from_bridge_evidence(
        &execution_record_identity_basis.bridge_evidence_identity(),
    );
    let artifact_identity =
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_domain_preview_workflow_foundation_artifact_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("binding"), &binding_identity)
            .field_shape(
                WorthQueryEvidenceTag::new("request"),
                request_family.as_str(),
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("preview_session"),
                &preview_session_identity.bridge_admission_evidence(),
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("declaration_identity"),
                &declaration_identity.bridge_admission_evidence(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration_digest"),
                &declaration_digest_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("lifecycle"),
                format!("{lifecycle_state_kind:?}"),
            )
            .field_bridge_retained_evidence_identity(
                WorthQueryEvidenceTag::new("execution_record"),
                &execution_record_identity.bridge_admission_evidence(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("evaluation_class"),
                evaluation_class.as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("shape_check_width"),
                shape_check_width.to_string(),
            )
            .seal()?;

    Ok(PreviewWorkflowFoundationArtifact {
        artifact_identity,
        binding_identity,
        canonical_query_digest,
        validated_query_digest,
        request_family,
        preview_session_identity,
        declaration_identity,
        declaration_digest_identity,
        lifecycle_state_kind,
        evaluation_class,
        execution_record_identity,
        shape_check_width,
    })
}

pub(crate) fn admit_contributed_preview_workflow_foundation(
    artifact: PreviewWorkflowFoundationArtifact,
) -> Result<AdmittedPreviewWorkflowFoundation, PreviewWorkflowFoundationError> {
    match artifact.request_family() {
        PreviewWorkflowFoundationRequest::CompareBasisPair
        | PreviewWorkflowFoundationRequest::DeferredMutationWriteback => {
            Ok(AdmittedPreviewWorkflowFoundation {
                artifact,
                counters: admitted_contributed_preview_workflow_foundation_counters(),
            })
        }
    }
}

fn admitted_contributed_preview_workflow_foundation_counters() -> PreviewExecutionCounters {
    PreviewExecutionCounters {
        binding_counters: PreviewBindingCounters::default(),
        execution_counters: ExecutionCounters::default(),
        preview_execution_envelope_count: 0,
        preview_execution_count: 0,
        preview_promotable_execution_count: 0,
        preview_read_only_execution_count: 0,
        preview_comparison_eligibility_proof_count: 0,
        preview_comparison_shape_check_width: 0,
        preview_workflow_foundation_admission_count: 1,
        preview_workflow_foundation_denial_count: 0,
        preview_workflow_foundation_artifact_lookup_count: 1,
        preview_work_avoided_by_explicit_basis_count: 1,
    }
}
