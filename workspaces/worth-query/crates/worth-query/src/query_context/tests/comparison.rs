use super::super::{
    admit_and_scope_legacy_query_basis_context_for_test, bind_diff_query_context,
    bind_legacy_query_basis_context, execute_query_basis_context, shape_query_diff_change_set,
    ComparisonBasisFamily, QueryBasisContextRequest, QueryContextAdmissionFailureClass,
    QueryContextBindingSource, QueryContextBudgetClass, QueryContextCostClass,
    QueryContextPredictionDriftOutcome,
};
use crate::facade::foundation::{
    admit_historical_evaluation_path, materialization_metadata_from_resolved,
    resolve_historical_materialization_path, HistoricalCapabilityDescriptor,
    HistoricalEvaluationRequest, HistoricalMaterializationDescriptor,
    HistoricalPathReuseDescriptor,
};
use crate::harness::fixtures::execution_preflights;

#[test]
fn diff_scope_mismatch_rejects_before_broadening() {
    let left_preflight = execution_preflights::direct_runtime_preflight();
    let right_preflight = execution_preflights::ordered_collection_preflight();
    let left = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::current_branch_head(),
            QueryContextBindingSource::RuntimeCurrent(&left_preflight),
        )
        .expect("left context should bind"),
    )
    .expect("left context should admit");
    let right = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::branch_head("branch:ordered"),
            QueryContextBindingSource::RuntimeBranch(&right_preflight),
        )
        .expect("right context should bind"),
    )
    .expect("right context should admit");
    let error =
        bind_diff_query_context(&left, &right).expect_err("mismatched query scopes must reject");

    assert_eq!(
        error.failure_class(),
        &QueryContextAdmissionFailureClass::DiffScopeMismatch
    );
    assert_eq!(error.counters().basis_substitution_denial_count(), 1);
}

#[test]
fn basis_substitution_attempt_rejects_typed_and_early() {
    let query_preflight = execution_preflights::direct_runtime_preflight();
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "history:snapshot-1",
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::retained_snapshot_for_test(
        "history:snapshot-1",
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let admission = admit_historical_evaluation_path(request, capability)
        .expect("retained history should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::retained_snapshot_for_test("history:snapshot-1")
            .expect("fixture digest must fit"),
    )
    .expect("retained history should resolve");
    let metadata = materialization_metadata_from_resolved(resolved);

    let error = bind_legacy_query_basis_context(
        QueryBasisContextRequest::historical_snapshot("history:other"),
        QueryContextBindingSource::Historical {
            query_preflight: &query_preflight,
            admission: &admission,
            metadata: &metadata,
        },
    )
    .expect_err("basis substitution must deny");

    assert_eq!(
        error.failure_class(),
        &QueryContextAdmissionFailureClass::BasisSubstitutionForbidden
    );
    assert_eq!(error.counters().basis_substitution_denial_count(), 1);
    assert_eq!(error.counters().historical_lookup_width(), 1);
    assert_eq!(error.counters().denial_width(), 1);
}

#[test]
fn current_to_historical_diff_admission_attaches_cost_budget_and_prediction_posture() {
    let current_preflight = execution_preflights::direct_runtime_preflight();
    let historical_preflight = execution_preflights::direct_runtime_preflight();
    let current = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::current_branch_head(),
            QueryContextBindingSource::RuntimeCurrent(&current_preflight),
        )
        .expect("current context should bind"),
    )
    .expect("current context should admit");
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "history:snapshot-1",
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::retained_snapshot_for_test(
        "history:snapshot-1",
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let admission = admit_historical_evaluation_path(request, capability)
        .expect("retained history should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::retained_snapshot_for_test("history:snapshot-1")
            .expect("fixture digest must fit"),
    )
    .expect("retained history should resolve");
    let metadata = materialization_metadata_from_resolved(resolved);
    let historical = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::historical_snapshot("history:snapshot-1"),
            QueryContextBindingSource::Historical {
                query_preflight: &historical_preflight,
                admission: &admission,
                metadata: &metadata,
            },
        )
        .expect("historical context should bind"),
    )
    .expect("historical context should admit");

    let diff = bind_diff_query_context(&current, &historical).expect("diff should bind");

    assert_eq!(diff.family(), &ComparisonBasisFamily::CurrentToHistorical);
    assert_eq!(
        diff.cost_class(),
        &QueryContextCostClass::DiffComparisonBounded
    );
    assert_eq!(
        diff.budget_class(),
        &QueryContextBudgetClass::ComparisonBounded
    );
    assert_eq!(
        diff.prediction_drift_outcome(),
        &QueryContextPredictionDriftOutcome::PendingComparison
    );
    assert_eq!(diff.prediction_report().comparison_binding_width(), 2);
    assert_eq!(diff.counters().comparison_basis_lookup_count(), 1);
    assert_eq!(diff.counters().comparison_scope_width(), 2);
    assert_eq!(diff.counters().comparison_family_rediscovery_count(), 0);
}

#[test]
fn diff_change_set_rejects_basis_mismatched_execution_artifacts_before_materialization() {
    let left_preflight = execution_preflights::direct_runtime_preflight();
    let right_preflight = execution_preflights::alternate_basis_runtime_preflight();
    let left = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::current_branch_head(),
            QueryContextBindingSource::RuntimeCurrent(&left_preflight),
        )
        .expect("left context should bind"),
    )
    .expect("left context should admit");
    let right = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::branch_head("branch:snapshot-2"),
            QueryContextBindingSource::RuntimeBranch(&right_preflight),
        )
        .expect("right context should bind"),
    )
    .expect("right context should admit");
    let diff = bind_diff_query_context(&left, &right).expect("diff should bind");
    let left_result =
        execute_query_basis_context(&left).expect("left context execution should succeed");
    let wrong_right_result =
        execute_query_basis_context(&left).expect("left context execution should succeed");

    let error = shape_query_diff_change_set(&diff, &left_result, &wrong_right_result)
        .expect_err("basis-mismatched execution artifact must reject before row shaping");

    assert_eq!(
        error.failure_class(),
        &QueryContextAdmissionFailureClass::ComparisonShapeMismatch
    );
    assert_eq!(error.counters().comparison_basis_lookup_count(), 1);
    assert_eq!(error.counters().comparison_broadening_denial_count(), 0);
    assert_eq!(error.counters().denial_width(), 1);
}

#[test]
fn broadening_required_comparison_denies_before_rich_artifact_shaping() {
    let left_preflight = execution_preflights::ordered_collection_without_traversal_preflight();
    let right_preflight = execution_preflights::alternate_basis_ordered_collection_preflight();
    let left = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::current_branch_head(),
            QueryContextBindingSource::RuntimeCurrent(&left_preflight),
        )
        .expect("left context should bind"),
    )
    .expect("left context should admit");
    let right = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::branch_head("branch:ordered-collection"),
            QueryContextBindingSource::RuntimeBranch(&right_preflight),
        )
        .expect("right context should bind"),
    )
    .expect("right context should admit");
    let diff = bind_diff_query_context(&left, &right).expect("diff should bind");
    let left_result =
        execute_query_basis_context(&left).expect("left context execution should succeed");
    let right_result =
        execute_query_basis_context(&right).expect("right context execution should succeed");

    let error = shape_query_diff_change_set(&diff, &left_result, &right_result)
        .expect_err("ordered collection diff should deny hidden broadening");

    assert_eq!(
        error.failure_class(),
        &QueryContextAdmissionFailureClass::ComparisonBroadeningRequired
    );
    assert_eq!(error.counters().comparison_basis_lookup_count(), 1);
    assert_eq!(error.counters().comparison_broadening_denial_count(), 1);
    assert_eq!(error.counters().comparison_family_rediscovery_count(), 0);
    assert_eq!(error.counters().denial_width(), 1);
}

#[test]
fn historical_reconstruction_broadening_denies_before_rich_execution() {
    let preflight = execution_preflights::ordered_collection_preflight();
    let request = HistoricalEvaluationRequest::full_reconstruction_for_test(
        "history:reconstruction",
        4,
        8,
        HistoricalPathReuseDescriptor::no_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::full_reconstruction_for_test(
        "history:reconstruction",
        HistoricalPathReuseDescriptor::no_reuse(),
    )
    .expect("fixture digest must fit");
    let admission =
        admit_historical_evaluation_path(request, capability).expect("reconstruction should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::full_reconstruction_for_test("history:reconstruction")
            .expect("fixture digest must fit"),
    )
    .expect("reconstruction should resolve");
    let metadata = materialization_metadata_from_resolved(resolved);
    let admitted = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::historical_snapshot("history:reconstruction"),
            QueryContextBindingSource::Historical {
                query_preflight: &preflight,
                admission: &admission,
                metadata: &metadata,
            },
        )
        .expect("historical reconstruction context should bind"),
    )
    .expect("historical reconstruction context should admit");

    let error = execute_query_basis_context(&admitted)
        .expect_err("reconstruction lane should deny broadening before execution");

    assert_eq!(
        error.failure_class(),
        &QueryContextAdmissionFailureClass::HistoricalPathTooBroadDenied
    );
    assert_eq!(
        error
            .counters()
            .materialization_path_compatibility_check_count(),
        1
    );
    assert_eq!(error.counters().historical_broadening_denial_count(), 1);
    assert_eq!(error.counters().denial_width(), 1);
}
