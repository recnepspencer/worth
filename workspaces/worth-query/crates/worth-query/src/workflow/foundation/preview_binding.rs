use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::preview::{
    AdmittedPreviewWorkflowFoundation, PreviewEvaluationClass, PreviewWorkflowFoundationRequest,
    PromotionParityPreviewComparisonAdmission,
};
use crate::workflow::WorkflowCounters;
use worth_runtime_bridge::facade::BridgePreviewSessionIdentity;

use super::context_binding::{
    WorkflowBasisFamily, WorkflowContextBinding, WorkflowPreviewEvaluationClass,
};
use super::context_identity::{
    apply_binding_scope_field, binding_scope_for_context_binding, workflow_context_basis_identity,
    workflow_context_binding_identity, workflow_context_query_identity,
    workflow_context_source_identity, workflow_scope_from_label,
    workflow_validated_query_digest_evidence, WorkflowBindingScopeField,
};
use super::preview_identity::{
    preview_promotion_comparison_basis_inner_identity,
    preview_promotion_comparison_source_identity, preview_workflow_foundation_basis_inner_identity,
    preview_workflow_foundation_source_identity,
};

fn synthetic_preview_workflow_query_identity(
    source_label: &str,
    binding_scope: &WorkflowBindingScopeField<'_>,
    preview_session_identity: &BridgePreviewSessionIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(apply_binding_scope_field(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "synthetic_preview_workflow_query_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("source_label"), source_label),
        binding_scope,
    )
    .field_bridge_authority_identity(
        WorthQueryEvidenceTag::new("preview_session"),
        &preview_session_identity.bridge_trust_boundary(),
    )
    .seal()?)
}

fn synthetic_preview_workflow_source_identity(
    source_label: &str,
    binding_scope: &WorkflowBindingScopeField<'_>,
    evaluation_class: &WorkflowPreviewEvaluationClass,
    preview_session_identity: &BridgePreviewSessionIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(apply_binding_scope_field(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "synthetic_preview_workflow_source_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("source_label"), source_label)
            .field_shape(
                WorthQueryEvidenceTag::new("evaluation_class"),
                evaluation_class.as_str(),
            ),
        binding_scope,
    )
    .field_bridge_authority_identity(
        WorthQueryEvidenceTag::new("preview_session"),
        &preview_session_identity.bridge_trust_boundary(),
    )
    .seal()?)
}

fn synthetic_preview_workflow_basis_identity(
    source_label: &str,
    binding_scope: &WorkflowBindingScopeField<'_>,
    preview_session_identity: &BridgePreviewSessionIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(apply_binding_scope_field(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowContextBinding)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "synthetic_preview_workflow_basis_v1",
            )
            .field_shape(
                WorthQueryEvidenceTag::new("basis_family"),
                WorkflowBasisFamily::PreviewFoundation.as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("source_label"), source_label),
        binding_scope,
    )
    .field_bridge_authority_identity(
        WorthQueryEvidenceTag::new("preview_session"),
        &preview_session_identity.bridge_trust_boundary(),
    )
    .seal()?)
}

pub(crate) fn synthetic_preview_workflow_binding(
    source_label: &str,
    preview_session_identity: BridgePreviewSessionIdentity,
    evaluation_class: WorkflowPreviewEvaluationClass,
) -> Result<WorkflowContextBinding, IdentityDerivationDenial> {
    Ok(synthetic_preview_workflow_binding_scoped(
        source_label,
        "unscoped",
        preview_session_identity,
        evaluation_class,
    )?)
}

pub(crate) fn synthetic_preview_workflow_binding_scoped(
    source_label: &str,
    binding_scope_digest: &str,
    preview_session_identity: BridgePreviewSessionIdentity,
    evaluation_class: WorkflowPreviewEvaluationClass,
) -> Result<WorkflowContextBinding, IdentityDerivationDenial> {
    Ok(synthetic_preview_workflow_binding_request_scoped(
        source_label,
        binding_scope_digest,
        preview_session_identity,
        evaluation_class,
        PreviewWorkflowFoundationRequest::compare_basis_pair(),
    )?)
}

pub(crate) fn synthetic_preview_workflow_binding_request_scoped(
    source_label: &str,
    binding_scope_label: &str,
    preview_session_identity: BridgePreviewSessionIdentity,
    evaluation_class: WorkflowPreviewEvaluationClass,
    request_family: PreviewWorkflowFoundationRequest,
) -> Result<WorkflowContextBinding, IdentityDerivationDenial> {
    let binding_scope = workflow_scope_from_label(binding_scope_label);
    let source_identity = synthetic_preview_workflow_source_identity(
        source_label,
        &binding_scope,
        &evaluation_class,
        &preview_session_identity,
    )?;
    let query_identity = synthetic_preview_workflow_query_identity(
        source_label,
        &binding_scope,
        &preview_session_identity,
    )?;
    let basis_identity = synthetic_preview_workflow_basis_identity(
        source_label,
        &binding_scope,
        &preview_session_identity,
    )?;
    let binding_identity = workflow_context_binding_identity(
        &source_identity,
        &query_identity,
        WorkflowBasisFamily::PreviewFoundation,
        &basis_identity,
        None,
        binding_scope_for_context_binding(&binding_scope),
    )?;
    Ok(WorkflowContextBinding {
        binding_identity,
        source_identity,
        query_identity,
        basis_family: WorkflowBasisFamily::PreviewFoundation,
        basis_identity,
        runtime_snapshot_identity: None,
        runtime_target_branch: None,
        preview_evaluation_class: Some(evaluation_class),
        preview_request_family: Some(request_family),
        preview_session_identity: Some(preview_session_identity),
        counters: WorkflowCounters {
            workflow_basis_binding_count: 1,
            workflow_basis_binding_width: 1,
            workflow_executor_rediscovery_count: 0,
            ..WorkflowCounters::default()
        },
    })
}

pub(super) fn bind_preview_foundation(
    foundation: &AdmittedPreviewWorkflowFoundation,
) -> Result<WorkflowContextBinding, super::admission_reporting::WorkflowAdmissionError> {
    let evaluation_class = match foundation.evaluation_class() {
        PreviewEvaluationClass::ReadOnly(_) => WorkflowPreviewEvaluationClass::ReadOnly,
        PreviewEvaluationClass::PromotionEligible(_) => {
            WorkflowPreviewEvaluationClass::PromotionEligible
        }
    };
    let source_identity =
        workflow_context_source_identity(&preview_workflow_foundation_source_identity(foundation));
    let query_identity = workflow_context_query_identity(
        &workflow_validated_query_digest_evidence(foundation.validated_query_digest()),
    );
    let basis_identity = workflow_context_basis_identity(
        &WorkflowBasisFamily::PreviewFoundation,
        &preview_workflow_foundation_basis_inner_identity(foundation),
    );
    let binding_identity = workflow_context_binding_identity(
        &source_identity,
        &query_identity,
        WorkflowBasisFamily::PreviewFoundation,
        &basis_identity,
        None,
        None,
    );

    Ok(WorkflowContextBinding {
        binding_identity,
        source_identity,
        query_identity,
        basis_family: WorkflowBasisFamily::PreviewFoundation,
        basis_identity,
        runtime_snapshot_identity: None,
        runtime_target_branch: None,
        preview_evaluation_class: Some(evaluation_class),
        preview_request_family: Some(foundation.request_family().clone()),
        preview_session_identity: Some(foundation.preview_session_identity().clone()),
        counters: WorkflowCounters {
            workflow_basis_binding_count: 1,
            workflow_basis_binding_width: 1,
            workflow_executor_rediscovery_count: 0,
            ..WorkflowCounters::default()
        },
    })
}

pub(super) fn bind_preview_promotion_comparison(
    comparison: &PromotionParityPreviewComparisonAdmission,
) -> Result<WorkflowContextBinding, super::admission_reporting::WorkflowAdmissionError> {
    let source_identity =
        workflow_context_source_identity(&preview_promotion_comparison_source_identity(comparison));
    let query_identity = workflow_context_query_identity(
        &workflow_validated_query_digest_evidence(comparison.validated_query_digest()),
    );
    let basis_identity = workflow_context_basis_identity(
        &WorkflowBasisFamily::PreviewPromotionComparison,
        &preview_promotion_comparison_basis_inner_identity(comparison),
    );
    let binding_identity = workflow_context_binding_identity(
        &source_identity,
        &query_identity,
        WorkflowBasisFamily::PreviewPromotionComparison,
        &basis_identity,
        None,
        None,
    );

    Ok(WorkflowContextBinding {
        binding_identity,
        source_identity,
        query_identity,
        basis_family: WorkflowBasisFamily::PreviewPromotionComparison,
        basis_identity,
        runtime_snapshot_identity: None,
        runtime_target_branch: None,
        preview_evaluation_class: Some(WorkflowPreviewEvaluationClass::PromotionEligible),
        preview_request_family: None,
        preview_session_identity: None,
        counters: WorkflowCounters {
            workflow_basis_binding_count: 1,
            workflow_basis_binding_width: 1,
            workflow_executor_rediscovery_count: 0,
            ..WorkflowCounters::default()
        },
    })
}
