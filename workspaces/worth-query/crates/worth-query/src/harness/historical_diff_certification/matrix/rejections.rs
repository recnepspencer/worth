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
    bind_legacy_query_basis_context, execute_query_basis_context, shape_query_diff_change_set,
    QueryBasisContextRequest, QueryContextAdmissionError, QueryContextBindingSource,
};

pub(super) fn historical_broadening_error(
) -> Result<QueryContextAdmissionError, IdentityDerivationDenial> {
    let preflight = execution_preflights::ordered_collection_preflight();
    let request = HistoricalEvaluationRequest::full_reconstruction_for_test(
        "history:reconstruction",
        4,
        8,
        HistoricalPathReuseDescriptor::no_reuse(),
    )?;
    let capability = HistoricalCapabilityDescriptor::full_reconstruction_for_test(
        "history:reconstruction",
        HistoricalPathReuseDescriptor::no_reuse(),
    )?;
    let admission =
        admit_historical_evaluation_path(request, capability).expect("reconstruction should admit");
    let resolved = resolve_historical_materialization_path(
        admission.clone(),
        HistoricalMaterializationDescriptor::full_reconstruction_for_test(
            "history:reconstruction",
        )?,
    )
    .expect("reconstruction should resolve");
    let metadata = materialization_metadata_from_resolved(resolved);
    let context = admit_and_scope_legacy_query_basis_context_for_test(
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

    Ok(execute_query_basis_context(&context)
        .expect_err("reconstruction lane should deny broadening before rich execution"))
}

pub(super) fn preview_lane_context() -> crate::query_context::ScopedQueryBasisContext {
    let preflight = execution_preflights::direct_runtime_preflight();
    let (_runtime, active, execution_record) =
        active_preview_artifacts("historical-diff-preview-rejection");
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
    admit_and_scope_legacy_query_basis_context_for_test(
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
    .expect("preview-derived context should admit")
}

pub(super) fn diff_scope_mismatch_error() -> QueryContextAdmissionError {
    let left_preflight = execution_preflights::direct_runtime_preflight();
    let right_preflight = execution_preflights::ordered_collection_preflight();
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
            QueryBasisContextRequest::branch_head("branch:ordered"),
            QueryContextBindingSource::RuntimeBranch(&right_preflight),
        )
        .expect("right should bind"),
    )
    .expect("right should admit");
    bind_diff_query_context(&left, &right).expect_err("mismatched query scopes should deny")
}

pub(super) fn basis_substitution_error(
) -> Result<QueryContextAdmissionError, IdentityDerivationDenial> {
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

    Ok(bind_legacy_query_basis_context(
        QueryBasisContextRequest::historical_snapshot("history:other"),
        QueryContextBindingSource::Historical {
            query_preflight: &preflight,
            admission: &admission,
            metadata: &metadata,
        },
    )
    .expect_err("basis substitution must deny"))
}

pub(super) fn comparison_broadening_error() -> QueryContextAdmissionError {
    let left_preflight = execution_preflights::ordered_collection_without_traversal_preflight();
    let right_preflight = execution_preflights::alternate_basis_ordered_collection_preflight();
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
            QueryBasisContextRequest::branch_head("branch:ordered-collection"),
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

    shape_query_diff_change_set(&diff, &left_result, &right_result)
        .expect_err("ordered collection diff should deny hidden broadening")
}

pub(super) fn comparison_shape_mismatch_error() -> QueryContextAdmissionError {
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
    let wrong_right_result =
        execute_query_basis_context(&left).expect("left query-context execution should succeed");

    shape_query_diff_change_set(&diff, &left_result, &wrong_right_result)
        .expect_err("basis-mismatched execution artifact should deny comparison shaping")
}
