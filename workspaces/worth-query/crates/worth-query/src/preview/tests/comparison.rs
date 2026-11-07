use crate::harness::fixtures::execution_preflights;
use crate::harness::fixtures::preview_bridge::active_preview_artifacts;
use crate::preview::{
    admit_authoritative_preview_comparison_candidate, admit_preview_promotion_parity_comparison,
    admit_promotion_eligible_preview_session_plan_binding,
    admit_read_only_preview_session_plan_binding, bind_preflight_to_preview_session,
    derive_preview_comparison_eligibility, execute_promotion_eligible_preview_session_plan,
    execute_read_only_preview_session_plan, PreviewComparisonFailureClass, PreviewEvaluationClass,
    PreviewSessionQueryContext,
};

#[test]
fn preview_execution_comparison_admits_shape_compatible_runtime_result() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-comparison-admitted");
    let binding = bind_preflight_to_preview_session(
        preflight.clone(),
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        ),
    )
    .expect("preview binding should succeed");
    let preview_execution = admit_promotion_eligible_preview_session_plan_binding(binding)
        .expect("promotion-eligible binding should admit");
    let preview_execution = execute_promotion_eligible_preview_session_plan(&preview_execution)
        .expect("preview execution should succeed");
    let candidate_execution = crate::execution::execute_preflight_bundle(&preflight)
        .expect("candidate execution should succeed");
    let candidate =
        admit_authoritative_preview_comparison_candidate(&preflight, &candidate_execution)
            .expect("runtime candidate should admit");

    let admission = admit_preview_promotion_parity_comparison(&preview_execution, &candidate)
        .expect("shape-compatible runtime result should admit comparison");
    let admission = admission.as_preview_comparison();

    assert!(!admission.digest().is_empty());
    assert_eq!(
        admission.preview_execution_digest(),
        preview_execution
            .as_preview_execution()
            .report()
            .preview_execution_digest()
    );
    assert_eq!(
        admission.candidate_result_digest(),
        candidate_execution.report().result_digest()
    );
    assert_eq!(
        admission.candidate_basis_digest(),
        preflight.basis().proof().digest().as_str()
    );
    assert!(admission.shape_check_width() > 0);
    assert_eq!(admission.counters().preview_promotion_comparison_count(), 1);
    assert_eq!(
        admission
            .counters()
            .preview_promotion_comparison_denial_count(),
        0
    );
    assert_eq!(admission.counters().preview_basis_pair_width(), 2);
}

#[test]
fn preview_execution_comparison_rejects_query_digest_mismatch_before_shape_checks() {
    let preview_preflight = execution_preflights::ordered_collection_without_traversal_preflight();
    let candidate_preflight = execution_preflights::ordered_collection_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-comparison-ordering-mismatch");
    let binding = bind_preflight_to_preview_session(
        preview_preflight.clone(),
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        ),
    )
    .expect("ordered preview binding should succeed");
    let preview_execution = admit_promotion_eligible_preview_session_plan_binding(binding)
        .expect("promotion binding should admit");
    let preview_execution = execute_promotion_eligible_preview_session_plan(&preview_execution)
        .expect("preview execution should succeed");
    let candidate_execution = crate::execution::execute_preflight_bundle(&candidate_preflight)
        .expect("candidate execution should succeed");
    let candidate = admit_authoritative_preview_comparison_candidate(
        &candidate_preflight,
        &candidate_execution,
    )
    .expect("shape-mismatched runtime candidate should still admit as authoritative");

    let error = admit_preview_promotion_parity_comparison(&preview_execution, &candidate)
        .expect_err("materially different collection shape should reject comparison");

    assert_eq!(
        error.failure_class(),
        &PreviewComparisonFailureClass::QueryDigestMismatch
    );
    assert!(!error.preview_digest().is_empty());
    assert!(!error.candidate_digest().is_empty());
    assert_eq!(
        error.counters().preview_promotion_comparison_denial_count(),
        1
    );
}

#[test]
fn preview_execution_comparison_rejects_store_backed_candidates() {
    let preview_preflight = execution_preflights::direct_runtime_preflight();
    let candidate_preflight = execution_preflights::store_detail_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-comparison-store-candidate");
    let binding = bind_preflight_to_preview_session(
        preview_preflight.clone(),
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::read_only(),
        ),
    )
    .expect("preview binding should succeed");
    let binding = admit_read_only_preview_session_plan_binding(binding)
        .expect("read-only binding should admit");
    let _preview_execution =
        execute_read_only_preview_session_plan(&binding).expect("preview execution should succeed");
    let candidate_execution = crate::execution::execute_preflight_bundle(&candidate_preflight)
        .expect("candidate execution should succeed");
    let error = admit_authoritative_preview_comparison_candidate(
        &candidate_preflight,
        &candidate_execution,
    )
    .expect_err("store-backed comparison candidates must reject before comparison admission");

    assert_eq!(
        error.failure_class(),
        &PreviewComparisonFailureClass::CandidateBasisAuthorityMismatch
    );
}

#[test]
fn read_only_preview_execution_stays_read_only_at_comparison_boundary() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-comparison-read-only-boundary");
    let binding = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::read_only(),
        ),
    )
    .expect("read-only preview binding should succeed");
    let read_only_binding = admit_read_only_preview_session_plan_binding(binding)
        .expect("read-only binding should admit");
    let read_only_execution = execute_read_only_preview_session_plan(&read_only_binding)
        .expect("read-only execution should succeed");

    assert_eq!(
        read_only_execution
            .as_preview_execution()
            .binding()
            .basis()
            .binding_tuple()
            .evaluation_class(),
        &PreviewEvaluationClass::read_only()
    );
}

#[test]
fn preview_comparison_candidate_rejects_inconsistent_execution_preflight_pair() {
    let candidate_preflight = execution_preflights::direct_runtime_preflight();
    let mismatched_execution_preflight = execution_preflights::ordered_collection_preflight();
    let mismatched_execution =
        crate::execution::execute_preflight_bundle(&mismatched_execution_preflight)
            .expect("mismatched execution should succeed");

    let error = admit_authoritative_preview_comparison_candidate(
        &candidate_preflight,
        &mismatched_execution,
    )
    .expect_err("candidate proof should reject execution from a different admitted plan");

    assert!(
        matches!(
            error.failure_class(),
            PreviewComparisonFailureClass::CandidateExecutionPlanMismatch
                | PreviewComparisonFailureClass::CandidateExecutionBasisMismatch
        ),
        "expected candidate proof to reject inconsistent execution pair, got {:?}",
        error.failure_class()
    );
}

#[test]
fn preview_comparison_candidate_tracks_candidate_shape_contracts() {
    let preflight = execution_preflights::ordered_collection_without_traversal_preflight();
    let execution = crate::execution::execute_preflight_bundle(&preflight)
        .expect("candidate execution should succeed");
    let candidate = admit_authoritative_preview_comparison_candidate(&preflight, &execution)
        .expect("authoritative runtime candidate should admit");
    let artifact = candidate.artifact();

    assert_eq!(
        artifact.validated_query_digest(),
        preflight.plan().query().validated_query_digest()
    );
    assert_eq!(artifact.result_digest(), execution.report().result_digest());
    assert_eq!(
        artifact.basis_digest(),
        preflight.basis().proof().digest().as_str()
    );
    assert!(artifact.collection_digest().is_some());
    assert_eq!(artifact.result_family(), "ordinary_collection");
    assert!(artifact.shape_check_width() > 0);
}

#[test]
fn preview_comparison_eligibility_uses_collection_shape_contracts() {
    let preflight = execution_preflights::ordered_collection_without_traversal_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("preview-comparison-eligibility");
    let binding = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::read_only(),
        ),
    )
    .expect("ordered collection preview should bind");

    let artifact =
        derive_preview_comparison_eligibility(&binding).expect("fixture digest must fit");

    assert_eq!(artifact.result_family(), "ordinary_collection");
    assert_eq!(
        &artifact.canonical_query_digest,
        binding.basis().binding_tuple().canonical_query_digest()
    );
    assert_eq!(
        &artifact.canonical_result_shape_digest,
        binding
            .basis()
            .binding_tuple()
            .canonical_result_shape_digest()
    );
    assert!(artifact.collection_digest().is_some());
    assert!(artifact.shape_check_width() > 0);
    assert!(!artifact.ordering_digest().is_empty());
    assert!(!artifact.materialization_boundary_digest().is_empty());
}
