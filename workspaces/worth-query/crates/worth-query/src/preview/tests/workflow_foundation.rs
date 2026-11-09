use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::harness::fixtures::execution_preflights;
use crate::harness::fixtures::preview_bridge::active_preview_artifacts;
use crate::preview::workflow_context_identity;
use crate::preview::{
    admit_preview_workflow_foundation, admit_preview_workflow_foundation_request,
    bind_preflight_to_preview_session, PreviewEvaluationClass, PreviewSessionQueryContext,
    PreviewWorkflowFoundationFailureClass, PreviewWorkflowFoundationRequest,
};
use worth_runtime_bridge::facade::BridgePreviewLifecycleStateKind;

fn expected_preview_declaration_digest_identity(
    binding_tuple: &crate::preview::PreviewSessionBindingTuple,
) -> WorthQueryEvidenceIdentity {
    workflow_context_identity::compose_preview_declaration_digest_workflow_identity(binding_tuple)
        .expect("fixture digest must fit")
}

fn expected_preview_binding_tuple_identity(
    binding_tuple: &crate::preview::PreviewSessionBindingTuple,
) -> WorthQueryEvidenceIdentity {
    workflow_context_identity::compose_preview_binding_tuple_workflow_identity(binding_tuple)
        .expect("fixture digest must fit")
}

#[test]
fn preview_workflow_foundation_is_bound_to_the_admitted_preview_tuple() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-workflow-foundation");
    let binding = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        ),
    )
    .expect("promotion-eligible preview binding should succeed");

    let workflow = admit_preview_workflow_foundation(&binding)
        .expect("comparison-basis workflow foundation should admit");

    assert_eq!(
        workflow.artifact().preview_session_identity(),
        binding.basis().binding_tuple().preview_session_identity()
    );
    assert_eq!(
        workflow.artifact().declaration_identity(),
        binding.basis().binding_tuple().declaration_identity()
    );
    assert_eq!(
        workflow.artifact().execution_record_identity(),
        binding
            .basis()
            .binding_tuple()
            .execution_record_identity()
            .expect("active binding should carry execution record identity")
    );
    assert_eq!(
        workflow.artifact().evaluation_class(),
        &PreviewEvaluationClass::promotion_eligible()
    );
    assert_eq!(
        workflow.artifact().binding_identity(),
        &expected_preview_binding_tuple_identity(binding.basis().binding_tuple()),
    );
    assert_eq!(
        workflow.request_family(),
        &PreviewWorkflowFoundationRequest::compare_basis_pair()
    );
    assert_eq!(
        workflow.artifact().declaration_digest_identity(),
        &expected_preview_declaration_digest_identity(binding.basis().binding_tuple()),
    );
    assert_eq!(
        workflow.artifact().lifecycle_state_kind(),
        BridgePreviewLifecycleStateKind::Active
    );
}

#[test]
fn deferred_writeback_workflow_foundation_admits_with_explicit_request_family() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-workflow-foundation-writeback");
    let binding = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        ),
    )
    .expect("promotion-eligible preview binding should succeed");

    let workflow = admit_preview_workflow_foundation_request(
        &binding,
        PreviewWorkflowFoundationRequest::deferred_mutation_writeback(),
    )
    .expect("deferred writeback workflow foundations should admit on the ordinary preview path");

    assert_eq!(
        workflow.request_family(),
        &PreviewWorkflowFoundationRequest::deferred_mutation_writeback()
    );
    assert_eq!(
        workflow.artifact().evaluation_class(),
        &PreviewEvaluationClass::promotion_eligible()
    );
    assert_eq!(
        workflow.artifact().binding_identity(),
        &expected_preview_binding_tuple_identity(binding.basis().binding_tuple()),
    );
    assert_eq!(
        workflow
            .counters()
            .preview_workflow_foundation_admission_count(),
        1
    );
    assert_eq!(
        workflow
            .counters()
            .preview_workflow_foundation_denial_count(),
        0
    );
    assert_eq!(
        workflow
            .counters()
            .preview_workflow_foundation_artifact_lookup_count(),
        1
    );
}

#[test]
fn read_only_preview_denies_deferred_writeback_workflow_foundation_request() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-workflow-foundation-read-only-writeback-denied");
    let binding = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::read_only(),
        ),
    )
    .expect("read-only preview binding should succeed");

    let error = admit_preview_workflow_foundation_request(
        &binding,
        PreviewWorkflowFoundationRequest::deferred_mutation_writeback(),
    )
    .expect_err("read-only preview should not admit deferred writeback foundations");

    assert_eq!(
        error.failure_class(),
        &PreviewWorkflowFoundationFailureClass::ReadOnlyPreviewWritebackFoundationForbidden
    );
    assert_eq!(
        error
            .counters()
            .preview_workflow_foundation_admission_count(),
        0
    );
    assert_eq!(
        error.counters().preview_workflow_foundation_denial_count(),
        1
    );
    assert_eq!(
        error
            .counters()
            .preview_workflow_foundation_artifact_lookup_count(),
        0
    );
}
