use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
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
use crate::query_context::{
    admit_and_scope_legacy_query_basis_context_for_test, bind_diff_query_context,
    bind_legacy_query_basis_context, build_query_basis_result_bundle,
    build_query_diff_result_bundle, execute_query_basis_context, shape_query_diff_change_set,
    QueryBasisContextRequest, QueryContextBindingSource,
};

use super::super::lane::HistoricalDiffLane;

pub(super) fn current_lane() -> HistoricalDiffLane {
    let preflight = execution_preflights::direct_runtime_preflight();
    let context = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::current_branch_head(),
            QueryContextBindingSource::RuntimeCurrent(&preflight),
        )
        .expect("current context should bind"),
    )
    .expect("current context should admit");
    let result = execute_query_basis_context(&context)
        .expect("current query-context execution should succeed");
    let bundle = build_query_basis_result_bundle(&context, result)
        .expect("basis result bundle should shape");

    HistoricalDiffLane::from_basis_result_bundle(&bundle)
}

pub(super) fn branch_lane() -> HistoricalDiffLane {
    let preflight = execution_preflights::alternate_basis_runtime_preflight();
    let context = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::branch_head("branch:snapshot-2"),
            QueryContextBindingSource::RuntimeBranch(&preflight),
        )
        .expect("branch context should bind"),
    )
    .expect("branch context should admit");
    let result = execute_query_basis_context(&context)
        .expect("branch query-context execution should succeed");
    let bundle = build_query_basis_result_bundle(&context, result)
        .expect("basis result bundle should shape");

    HistoricalDiffLane::from_basis_result_bundle(&bundle)
}

pub(super) fn historical_lane() -> Result<HistoricalDiffLane, IdentityDerivationDenial> {
    let preflight = execution_preflights::direct_runtime_preflight();
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "history:snapshot-1",
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let capability = HistoricalCapabilityDescriptor::retained_snapshot_for_test(
        "history:snapshot-1",
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let admission =
        admit_historical_evaluation_path(request, capability).expect("history should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::retained_snapshot_for_test("history:snapshot-1")?,
    )
    .expect("history should resolve");
    let metadata = materialization_metadata_from_resolved(resolved);
    let context = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::historical_snapshot("history:snapshot-1"),
            QueryContextBindingSource::Historical {
                query_preflight: &preflight,
                admission: &admission,
                metadata: &metadata,
            },
        )
        .expect("historical context should bind"),
    )
    .expect("historical context should admit");
    let result = execute_query_basis_context(&context)
        .expect("historical query-context execution should succeed");
    let bundle = build_query_basis_result_bundle(&context, result)
        .expect("basis result bundle should shape");

    Ok(HistoricalDiffLane::from_basis_result_bundle(&bundle))
}

pub(super) fn store_historical_lane() -> Result<HistoricalDiffLane, IdentityDerivationDenial> {
    let preflight = execution_preflights::store_detail_preflight();
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "history:snapshot-1",
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let capability = HistoricalCapabilityDescriptor::retained_snapshot_for_test(
        "history:snapshot-1",
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let admission =
        admit_historical_evaluation_path(request, capability).expect("history should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::retained_snapshot_for_test("history:snapshot-1")?,
    )
    .expect("history should resolve");
    let metadata = materialization_metadata_from_resolved(resolved);
    let context = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::historical_snapshot("history:snapshot-1"),
            QueryContextBindingSource::Historical {
                query_preflight: &preflight,
                admission: &admission,
                metadata: &metadata,
            },
        )
        .expect("store historical context should bind"),
    )
    .expect("store historical context should admit");
    let result = execute_query_basis_context(&context)
        .expect("store historical query-context execution should succeed");
    let bundle = build_query_basis_result_bundle(&context, result)
        .expect("basis result bundle should shape");

    Ok(HistoricalDiffLane::from_basis_result_bundle(&bundle))
}

pub(super) fn preview_lane() -> HistoricalDiffLane {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) = active_preview_artifacts("historical-diff-preview");
    let preview_binding = bind_preflight_to_preview_session(
        preflight,
        PreviewSessionQueryContext::active(
            &active,
            &execution_record,
            PreviewEvaluationClass::promotion_eligible(),
        ),
    )
    .expect("preview binding should succeed");
    let foundation = admit_preview_workflow_foundation(&preview_binding)
        .expect("preview foundation should admit");
    let preview_session_identity = foundation
        .preview_session_identity()
        .bridge_admission_evidence();
    let context = admit_and_scope_legacy_query_basis_context_for_test(
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
    let result = execute_query_basis_context(&context)
        .expect("preview-derived query-context execution should succeed");
    let bundle = build_query_basis_result_bundle(&context, result)
        .expect("basis result bundle should shape");

    HistoricalDiffLane::from_basis_result_bundle(&bundle)
}

pub(super) fn branch_diff_lane() -> HistoricalDiffLane {
    let left_preflight = execution_preflights::direct_runtime_preflight();
    let right_preflight = execution_preflights::alternate_basis_runtime_preflight();
    let left = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::current_branch_head(),
            QueryContextBindingSource::RuntimeCurrent(&left_preflight),
        )
        .expect("left should bind"),
    )
    .expect("left should admit");
    let right = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::branch_head("branch:snapshot-2"),
            QueryContextBindingSource::RuntimeBranch(&right_preflight),
        )
        .expect("right should bind"),
    )
    .expect("right should admit");
    let diff = bind_diff_query_context(&left, &right).expect("diff should bind");
    let left_result =
        execute_query_basis_context(&left).expect("left query-context execution should succeed");
    let right_result =
        execute_query_basis_context(&right).expect("right query-context execution should succeed");
    let change_set = shape_query_diff_change_set(&diff, &left_result, &right_result)
        .expect("diff change-set should shape");
    let bundle = build_query_diff_result_bundle(&diff, change_set, &left_result, &right_result)
        .expect("diff result bundle should shape");

    HistoricalDiffLane::from_diff_result_bundle(
        &bundle,
        left_result.counters().context_execution_count(),
        right_result.counters().context_execution_count(),
        left_result.counters().executor_rediscovery_count()
            + right_result.counters().executor_rediscovery_count(),
    )
}

pub(super) fn current_historical_diff_lane() -> Result<HistoricalDiffLane, IdentityDerivationDenial>
{
    let current_preflight = execution_preflights::direct_runtime_preflight();
    let historical_preflight = execution_preflights::direct_runtime_preflight();
    let current = admit_and_scope_legacy_query_basis_context_for_test(
        bind_legacy_query_basis_context(
            QueryBasisContextRequest::current_branch_head(),
            QueryContextBindingSource::RuntimeCurrent(&current_preflight),
        )
        .expect("current should bind"),
    )
    .expect("current should admit");
    let request = HistoricalEvaluationRequest::retained_snapshot_for_test(
        "history:snapshot-1",
        1,
        1,
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let capability = HistoricalCapabilityDescriptor::retained_snapshot_for_test(
        "history:snapshot-1",
        HistoricalPathReuseDescriptor::retained_reuse(),
    )?;
    let admission =
        admit_historical_evaluation_path(request, capability).expect("history should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::retained_snapshot_for_test("history:snapshot-1")?,
    )
    .expect("history should resolve");
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
        .expect("historical should bind"),
    )
    .expect("historical should admit");
    let diff = bind_diff_query_context(&current, &historical).expect("diff should bind");
    let current_result = execute_query_basis_context(&current)
        .expect("current query-context execution should succeed");
    let historical_result = execute_query_basis_context(&historical)
        .expect("historical query-context execution should succeed");
    let change_set = shape_query_diff_change_set(&diff, &current_result, &historical_result)
        .expect("current-to-historical diff change-set should shape");
    let bundle =
        build_query_diff_result_bundle(&diff, change_set, &current_result, &historical_result)
            .expect("current-to-historical diff result bundle should shape");

    Ok(HistoricalDiffLane::from_diff_result_bundle(
        &bundle,
        current_result.counters().context_execution_count(),
        historical_result.counters().context_execution_count(),
        current_result.counters().executor_rediscovery_count()
            + historical_result.counters().executor_rediscovery_count(),
    ))
}
