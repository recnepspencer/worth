use super::super::{
    admit_and_scope_legacy_query_basis_context_for_test, bind_legacy_query_basis_context,
    execute_query_basis_context, HistoricalAdmissionClass, HistoricalMaterializationCostClass,
    QueryBasisContextRequest, QueryContextAdmissionFailureClass, QueryContextBindingSource,
    QueryContextBudgetClass, QueryContextCostClass, QueryContextFamily,
    QueryContextPredictionDriftOutcome,
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
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use worth_runtime_bridge::facade::RelationalBridgeSnapshotIdentityParts;

#[test]
fn current_branch_head_context_binding_preserves_runtime_digests() {
    let preflight = execution_preflights::direct_runtime_preflight();
    let binding = bind_legacy_query_basis_context(
        QueryBasisContextRequest::current_branch_head(),
        QueryContextBindingSource::RuntimeCurrent(&preflight),
    )
    .expect("current runtime basis should bind");
    let admitted = admit_and_scope_legacy_query_basis_context_for_test(binding)
        .expect("current runtime basis should admit");

    assert_eq!(admitted.family(), &QueryContextFamily::CurrentBranchHead);
    assert_eq!(
        admitted.query_digest(),
        preflight.plan().query().validated_query_digest().as_str()
    );
    assert_eq!(
        admitted.basis_digest(),
        preflight.basis().proof().digest().as_str()
    );
    assert_eq!(
        admitted.cost_class(),
        &QueryContextCostClass::CurrentHeadNarrow
    );
    assert_eq!(
        admitted.budget_class(),
        &QueryContextBudgetClass::NarrowSingleBasis
    );
    assert_eq!(
        admitted.prediction_drift_outcome(),
        Some(&QueryContextPredictionDriftOutcome::PendingExecution)
    );
    assert_eq!(admitted.counters().query_basis_binding_count(), 1);
    assert_eq!(admitted.counters().historical_basis_lookup_count(), 0);
    assert_eq!(admitted.counters().basis_binding_width(), 1);
    assert_eq!(admitted.counters().denial_width(), 0);
    assert_eq!(admitted.counters().basis_rediscovery_count(), 0);
}

#[test]
fn legacy_query_basis_execution_cannot_fabricate_count_aggregate_results() {
    let preflight = execution_preflights::aggregate_rollup_collection_preflight();
    let binding = bind_legacy_query_basis_context(
        QueryBasisContextRequest::current_branch_head(),
        QueryContextBindingSource::RuntimeCurrent(&preflight),
    )
    .expect("count preflight should remain bindable for inspection");
    let admitted = admit_and_scope_legacy_query_basis_context_for_test(binding)
        .expect("count context should remain inspectable");

    let error = execute_query_basis_context(&admitted)
        .expect_err("legacy query-basis execution must not produce a count result");

    assert_eq!(
        error.failure_class(),
        &QueryContextAdmissionFailureClass::UnsupportedHistoricalBasis
    );
}

#[test]
fn alternate_branch_head_context_binding_is_explicitly_distinct() {
    let preflight = execution_preflights::alternate_basis_runtime_preflight();
    let binding = bind_legacy_query_basis_context(
        QueryBasisContextRequest::branch_head("branch:snapshot-2"),
        QueryContextBindingSource::RuntimeBranch(&preflight),
    )
    .expect("alternate runtime basis should bind");
    let admitted = admit_and_scope_legacy_query_basis_context_for_test(binding)
        .expect("alternate runtime basis should admit");

    assert_eq!(admitted.family(), &QueryContextFamily::BranchHead);
    assert_eq!(
        admitted.basis_digest(),
        preflight.basis().proof().digest().as_str()
    );
    assert_eq!(
        admitted.cost_class(),
        &QueryContextCostClass::BranchHeadNarrow
    );
    assert_eq!(admitted.counters().basis_binding_width(), 1);
    assert_eq!(admitted.counters().basis_rediscovery_count(), 0);
}

#[test]
fn retained_historical_context_binding_reuses_admitted_history_artifacts() {
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

    let binding = bind_legacy_query_basis_context(
        QueryBasisContextRequest::historical_snapshot("history:snapshot-1"),
        QueryContextBindingSource::Historical {
            query_preflight: &query_preflight,
            admission: &admission,
            metadata: &metadata,
        },
    )
    .expect("historical context should bind");
    let admitted = admit_and_scope_legacy_query_basis_context_for_test(binding)
        .expect("historical context should admit");
    let execution = execute_query_basis_context(&admitted)
        .expect("historical query-context execution should succeed");
    let shaped = super::super::attach_query_basis_metadata(&admitted, &execution)
        .expect("historical metadata should shape");

    assert_eq!(admitted.family(), &QueryContextFamily::HistoricalSnapshot);
    assert_eq!(
        admitted.cost_class(),
        &QueryContextCostClass::HistoricalRetainedBounded
    );
    assert_eq!(
        admitted.budget_class(),
        &QueryContextBudgetClass::HistoricalBounded
    );
    assert_eq!(
        admitted.historical_admission_class(),
        Some(&HistoricalAdmissionClass::RuntimeRetained)
    );
    assert_eq!(
        admitted.historical_materialization_cost_class(),
        Some(&HistoricalMaterializationCostClass::RetainedBounded)
    );
    assert!(admitted.materialization_path_identity_source().is_some());
    assert!(shaped.materialization_path_identity().is_some());
    assert_eq!(
        shaped.historical_materialization_cost_class(),
        Some(&HistoricalMaterializationCostClass::RetainedBounded)
    );
    assert_eq!(admitted.counters().historical_basis_lookup_count(), 1);
    assert_eq!(admitted.counters().historical_lookup_width(), 1);
    assert_eq!(admitted.counters().historical_path_rediscovery_count(), 0);
}

#[test]
fn retained_historical_admission_owns_runtime_snapshot_matching() {
    let snapshot = WorthQuerySnapshotIdentity::from_relational_snapshot(
        RelationalBridgeSnapshotIdentityParts::new(41, 7),
    );
    let admitted_projection = snapshot
        .bridge_admission_evidence()
        .terminal_projection_for_reporting()
        .to_owned();
    let query_preflight = execution_preflights::direct_runtime_preflight();
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        admitted_projection.clone(),
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let capability = HistoricalCapabilityDescriptor::retained_snapshot_for_test(
        admitted_projection.clone(),
        HistoricalPathReuseDescriptor::retained_reuse(),
    )
    .expect("fixture digest must fit");
    let admission = admit_historical_evaluation_path(request, capability)
        .expect("retained snapshot path should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::retained_snapshot_for_test(
            admitted_projection.clone(),
        )
        .expect("fixture digest must fit"),
    )
    .expect("retained snapshot path should resolve");
    let metadata = materialization_metadata_from_resolved(resolved);
    let historical = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::historical_snapshot(admitted_projection.clone()),
            QueryContextBindingSource::Historical {
                query_preflight: &query_preflight,
                admission: &admission,
                metadata: &metadata,
            },
        )
        .expect("historical snapshot context should bind"),
    )
    .expect("historical snapshot context should admit");

    assert!(historical.admits_runtime_snapshot(&snapshot));

    let wrong_snapshot = WorthQuerySnapshotIdentity::from_relational_snapshot(
        RelationalBridgeSnapshotIdentityParts::new(41, 8),
    );
    assert!(!historical.admits_runtime_snapshot(&wrong_snapshot));

    let branch_projection = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::branch_head(admitted_projection),
            QueryContextBindingSource::RuntimeBranch(&query_preflight),
        )
        .expect("runtime branch projection should bind"),
    )
    .expect("runtime branch projection should admit");
    assert!(!branch_projection.admits_runtime_snapshot(&snapshot));
}

#[test]
fn preview_derived_context_binding_preserves_preview_identity() {
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

    let binding = bind_legacy_query_basis_context(
        QueryBasisContextRequest::preview_derived_historical(
            preview_session_identity
                .terminal_projection_for_reporting()
                .to_string(),
        ),
        QueryContextBindingSource::PreviewDerivedHistorical(&foundation),
    )
    .expect("preview-derived context should bind");
    let admitted = admit_and_scope_legacy_query_basis_context_for_test(binding)
        .expect("preview-derived context should admit");

    assert_eq!(
        admitted.family(),
        &QueryContextFamily::PreviewDerivedHistorical
    );
    assert_eq!(
        admitted.query_digest(),
        foundation.validated_query_digest().as_str()
    );
    assert_eq!(
        admitted.cost_class(),
        &QueryContextCostClass::PreviewDerivedHistoricalBounded
    );
    assert_eq!(
        admitted.budget_class(),
        &QueryContextBudgetClass::PreviewDerivedBounded
    );
    assert_eq!(
        admitted.preview_provenance_identity_source(),
        Some(preview_session_identity.terminal_projection_for_reporting())
    );
}
