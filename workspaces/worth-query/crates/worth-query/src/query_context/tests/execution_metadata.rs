use super::super::{
    admit_and_scope_legacy_query_basis_context_for_test, attach_diff_query_metadata,
    attach_query_basis_metadata, bind_diff_query_context, bind_legacy_query_basis_context,
    execute_query_basis_context, shape_query_diff_change_set, ComparisonBasisFamily,
    QueryBasisContextRequest, QueryContextBindingSource, QueryContextBudgetClass,
    QueryContextCostClass, QueryContextExecutionFamily, QueryContextPredictionDriftOutcome,
};
use crate::facade::foundation::{
    admit_historical_evaluation_path, materialization_metadata_from_resolved,
    resolve_historical_materialization_path, HistoricalCapabilityDescriptor,
    HistoricalEvaluationRequest, HistoricalMaterializationDescriptor,
    HistoricalPathReuseDescriptor,
};
use crate::facade::policy::{
    admit_preview_workflow_foundation, bind_preflight_to_preview_session, PreviewEvaluationClass,
    PreviewSessionQueryContext,
};
use crate::harness::fixtures::{execution_preflights, preview_bridge::active_preview_artifacts};

#[test]
fn preview_derived_execution_is_query_owned_and_provenance_explicit() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) = active_preview_artifacts("query-context-preview");
    let binding = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        ),
    )
    .expect("preview binding should succeed");
    let foundation =
        admit_preview_workflow_foundation(&binding).expect("preview foundation should admit");
    let preview_session_identity = foundation
        .preview_session_identity()
        .bridge_admission_evidence();

    let admitted = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::preview_derived_historical(
                preview_session_identity
                    .terminal_projection_for_reporting()
                    .to_string(),
            ),
            QueryContextBindingSource::PreviewDerivedHistorical(&foundation),
        )
        .expect("preview-derived context should bind"),
    )
    .expect("preview-derived context should admit");
    let execution = execute_query_basis_context(&admitted)
        .expect("preview-derived query-context execution should succeed");
    let metadata = attach_query_basis_metadata(&admitted, &execution)
        .expect("preview-derived metadata should shape");

    assert_eq!(
        execution.family(),
        &QueryContextExecutionFamily::PreviewDerivedHistorical
    );
    assert_eq!(
        execution.preview_provenance_identity(),
        Some(preview_session_identity.terminal_projection_for_reporting())
    );
    assert_eq!(
        execution.prediction_drift_outcome(),
        &QueryContextPredictionDriftOutcome::WithinBudget
    );
    assert_eq!(
        metadata.preview_provenance_identity(),
        Some(preview_session_identity.terminal_projection_for_reporting())
    );
    assert_eq!(
        metadata.prediction_drift_outcome(),
        Some(&QueryContextPredictionDriftOutcome::WithinBudget)
    );
    assert_eq!(
        execution.counters().result_shape_width(),
        foundation.shape_check_width()
    );
    assert_eq!(execution.rows().len(), foundation.shape_check_width());
}

#[test]
fn diff_context_binding_preserves_both_basis_identities() {
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
    let diff = bind_diff_query_context(&left, &right).expect("diff context should bind");
    let left_result =
        execute_query_basis_context(&left).expect("left context execution should succeed");
    let right_result =
        execute_query_basis_context(&right).expect("right context execution should succeed");
    let change_set = shape_query_diff_change_set(&diff, &left_result, &right_result)
        .expect("diff change-set should shape");
    let metadata = attach_diff_query_metadata(&diff, &left_result, &right_result, &change_set)
        .expect("diff metadata should shape");

    assert_eq!(diff.family(), &ComparisonBasisFamily::BranchToBranch);
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
    assert_eq!(diff.prediction_report().comparison_row_width(), 1);
    assert_eq!(
        change_set.comparison_basis_family(),
        &ComparisonBasisFamily::BranchToBranch
    );
    assert_eq!(
        change_set.prediction_drift_outcome(),
        &QueryContextPredictionDriftOutcome::WithinBudget
    );
    assert_eq!(change_set.left_basis_digest(), left.basis_digest());
    assert_eq!(change_set.right_basis_digest(), right.basis_digest());
    assert!(
        !change_set.rows().is_empty(),
        "diff change-set should remain query-shaped and typed"
    );
    assert_eq!(metadata.left_basis_digest(), left.basis_digest());
    assert_eq!(metadata.right_basis_digest(), right.basis_digest());
    assert_eq!(
        metadata.cost_class(),
        &QueryContextCostClass::DiffComparisonBounded
    );
    assert_eq!(
        metadata.budget_class(),
        &QueryContextBudgetClass::ComparisonBounded
    );
    assert_eq!(metadata.prediction_report().comparison_binding_width(), 2);
    assert_eq!(
        metadata.comparison_result_digest(),
        change_set.result_digest()
    );
    assert_eq!(
        metadata.prediction_drift_outcome(),
        &QueryContextPredictionDriftOutcome::WithinBudget
    );
    assert_eq!(diff.counters().comparison_scope_width(), 2);
    assert_eq!(diff.counters().comparison_basis_lookup_count(), 1);
    assert_eq!(diff.counters().comparison_row_width(), 1);
    assert_eq!(diff.counters().comparison_broadening_denial_count(), 0);
    assert_eq!(diff.counters().comparison_family_rediscovery_count(), 0);
}

#[test]
fn historical_execution_artifact_preserves_requested_admitted_and_resolved_path_identity() {
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

    let admitted = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::historical_snapshot("history:snapshot-1"),
            QueryContextBindingSource::Historical {
                query_preflight: &query_preflight,
                admission: &admission,
                metadata: &metadata,
            },
        )
        .expect("historical context should bind"),
    )
    .expect("historical context should admit");
    let execution = execute_query_basis_context(&admitted)
        .expect("historical query-context execution should succeed");
    let shaped = attach_query_basis_metadata(&admitted, &execution)
        .expect("historical metadata should shape");

    assert_eq!(
        execution.family(),
        &QueryContextExecutionFamily::HistoricalMaterialized
    );
    assert_eq!(
        execution.requested_path_identity(),
        Some(metadata.requested_path_class().as_str())
    );
    assert_eq!(
        execution.admitted_path_identity(),
        Some(metadata.admitted_path_class().as_str())
    );
    assert_eq!(
        execution.resolved_path_identity(),
        Some(metadata.resolved_path_class().as_str())
    );
    assert_eq!(
        execution.prediction_drift_outcome(),
        &QueryContextPredictionDriftOutcome::WithinBudget
    );
    assert_eq!(
        shaped.requested_path_identity(),
        Some(metadata.requested_path_class().as_str())
    );
    assert_eq!(
        shaped.admitted_path_identity(),
        Some(metadata.admitted_path_class().as_str())
    );
    assert_eq!(
        shaped.resolved_path_identity(),
        Some(metadata.resolved_path_class().as_str())
    );
    assert_eq!(
        shaped.prediction_drift_outcome(),
        Some(&QueryContextPredictionDriftOutcome::WithinBudget)
    );
}
