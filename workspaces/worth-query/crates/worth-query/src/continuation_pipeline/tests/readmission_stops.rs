use crate::continuation_pipeline::{
    WorthQueryContinuationExecutionOutcome, WorthQueryContinuationExecutionReadmissionNextAction,
    WorthQueryContinuationExecutionReadmissionStop,
    WorthQueryContinuationExecutionReadmissionStopKind, WorthQueryPreparedContinuationOutcome,
};

use super::support::{
    admitted_workspace, drifted_readmission_handle_in, historical_truth_view_request,
    preview_session_request, runtime_route_request, target_request, HistoricalFamily,
    PreviewFamily, ReadmissionDrift, RuntimeFamily,
};

#[test]
fn stale_basis_stops_before_later_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    )) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected prepared continuation"),
    };
    let stop = match drifted_readmission_handle_in(&workspace, "main", ReadmissionDrift::Stale)
        .execute_prepared_continuation(prepared)
    {
        WorthQueryContinuationExecutionOutcome::Stale(stop) => stop,
        _ => panic!("stale basis must stop at continuation readmission"),
    };

    assert_readmission_stop(
        &stop,
        WorthQueryContinuationExecutionReadmissionStopKind::StaleBasis,
        WorthQueryContinuationExecutionReadmissionNextAction::RefreshBasis,
    );
}

#[test]
fn basis_mismatch_prescribes_basis_refresh_before_later_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(
        target_request::<HistoricalFamily>(&handle, "face-a", historical_truth_view_request()),
    ) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected prepared historical continuation"),
    };
    let stop =
        match drifted_readmission_handle_in(&workspace, "main", ReadmissionDrift::BasisMismatch)
            .execute_prepared_continuation(prepared)
        {
            WorthQueryContinuationExecutionOutcome::BasisMismatch(stop) => stop,
            _ => panic!("basis mismatch must stop at continuation readmission"),
        };

    assert_readmission_stop(
        &stop,
        WorthQueryContinuationExecutionReadmissionStopKind::BasisMismatch,
        WorthQueryContinuationExecutionReadmissionNextAction::RefreshBasis,
    );
}

#[test]
fn lower_binding_mismatch_prescribes_proof_inspection_before_later_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(
        target_request::<HistoricalFamily>(&handle, "face-a", historical_truth_view_request()),
    ) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected prepared historical continuation"),
    };
    let installed_authority = prepared.installed_authority().witness_identity().clone();
    assert_eq!(
        &installed_authority,
        handle.installed_authority().witness_identity()
    );
    let drifted_handle =
        drifted_readmission_handle_in(&workspace, "main", ReadmissionDrift::LowerBindingMismatch);
    assert_eq!(
        drifted_handle.installed_authority().witness_identity(),
        &installed_authority,
        "lower-binding drift must occur beneath the retained installed authority"
    );
    let stop = match drifted_handle.execute_prepared_continuation(prepared) {
        WorthQueryContinuationExecutionOutcome::LowerBindingMismatch(stop) => stop,
        _ => panic!("lower binding mismatch must stop at its owning boundary"),
    };

    assert_readmission_stop(
        &stop,
        WorthQueryContinuationExecutionReadmissionStopKind::LowerBindingMismatch,
        WorthQueryContinuationExecutionReadmissionNextAction::InspectProofLane,
    );
}

#[test]
fn authority_mismatch_prescribes_proof_inspection_before_later_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    )) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected prepared runtime continuation"),
    };
    let stop = match drifted_readmission_handle_in(
        &workspace,
        "main",
        ReadmissionDrift::AuthorityMismatch,
    )
    .execute_prepared_continuation(prepared)
    {
        WorthQueryContinuationExecutionOutcome::AuthorityMismatch(stop) => stop,
        _ => panic!("authority mismatch must stop at continuation readmission"),
    };

    assert_readmission_stop(
        &stop,
        WorthQueryContinuationExecutionReadmissionStopKind::AuthorityMismatch,
        WorthQueryContinuationExecutionReadmissionNextAction::InspectProofLane,
    );
}

#[test]
fn async_request_drift_prescribes_context_rebind_before_later_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    )) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected prepared runtime continuation"),
    };
    let stop =
        match drifted_readmission_handle_in(&workspace, "main", ReadmissionDrift::AsyncRequest)
            .execute_prepared_continuation(prepared)
        {
            WorthQueryContinuationExecutionOutcome::AsyncRequestDrift(stop) => stop,
            _ => panic!("async request drift must stop at continuation readmission"),
        };

    assert_readmission_stop(
        &stop,
        WorthQueryContinuationExecutionReadmissionStopKind::AsyncRequestDrift,
        WorthQueryContinuationExecutionReadmissionNextAction::RebindContext,
    );
}

#[test]
fn replay_drift_prescribes_basis_refresh_before_later_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(
        target_request::<HistoricalFamily>(&handle, "face-a", historical_truth_view_request()),
    ) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected prepared historical continuation"),
    };
    let stop = match drifted_readmission_handle_in(&workspace, "main", ReadmissionDrift::Replay)
        .execute_prepared_continuation(prepared)
    {
        WorthQueryContinuationExecutionOutcome::ReplayDrift(stop) => stop,
        _ => panic!("replay drift must stop at continuation readmission"),
    };

    assert_readmission_stop(
        &stop,
        WorthQueryContinuationExecutionReadmissionStopKind::ReplayDrift,
        WorthQueryContinuationExecutionReadmissionNextAction::RefreshBasis,
    );
}

#[test]
fn policy_remask_drift_prescribes_support_check_before_later_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    )) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected prepared runtime continuation"),
    };
    let stop = match drifted_readmission_handle_in(&workspace, "main", ReadmissionDrift::Remask)
        .execute_prepared_continuation(prepared)
    {
        WorthQueryContinuationExecutionOutcome::RemaskDrift(stop) => stop,
        _ => panic!("policy remask drift must stop at continuation readmission"),
    };

    assert_readmission_stop(
        &stop,
        WorthQueryContinuationExecutionReadmissionStopKind::PolicyRemaskDrift,
        WorthQueryContinuationExecutionReadmissionNextAction::CheckPolicySupport,
    );
}

#[test]
fn preview_residue_prescribes_explicit_handoff_before_later_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(target_request::<PreviewFamily>(
        &handle,
        "face-a",
        preview_session_request(),
    )) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected prepared preview continuation"),
    };
    let stop = match drifted_readmission_handle_in(
        &workspace,
        "main",
        ReadmissionDrift::PreviewCrossedResidue,
    )
    .execute_prepared_continuation(prepared)
    {
        WorthQueryContinuationExecutionOutcome::PreviewCrossedResidue(stop) => stop,
        _ => panic!("preview residue must stop at continuation readmission"),
    };

    assert_readmission_stop(
        &stop,
        WorthQueryContinuationExecutionReadmissionStopKind::PreviewCrossedResidue,
        WorthQueryContinuationExecutionReadmissionNextAction::UseExplicitHandoff,
    );
}

#[test]
fn stale_completion_prescribes_basis_refresh_before_later_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    )) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("expected prepared runtime continuation"),
    };
    let stop =
        match drifted_readmission_handle_in(&workspace, "main", ReadmissionDrift::StaleCompletion)
            .execute_prepared_continuation(prepared)
        {
            WorthQueryContinuationExecutionOutcome::StaleCompletion(stop) => stop,
            _ => panic!("stale completion must stop at continuation readmission"),
        };

    assert_readmission_stop(
        &stop,
        WorthQueryContinuationExecutionReadmissionStopKind::StaleCompletion,
        WorthQueryContinuationExecutionReadmissionNextAction::RefreshBasis,
    );
}

fn assert_readmission_stop(
    stop: &WorthQueryContinuationExecutionReadmissionStop,
    expected_kind: WorthQueryContinuationExecutionReadmissionStopKind,
    expected_action: WorthQueryContinuationExecutionReadmissionNextAction,
) {
    assert_eq!(stop.kind(), expected_kind);
    assert_eq!(stop.next_action(), expected_action);
    assert_eq!(stop.counters().planning_attempts(), 0);
    assert_eq!(stop.counters().lower_runtime_attempts(), 0);
    assert_eq!(stop.counters().execution_attempts(), 0);
}
