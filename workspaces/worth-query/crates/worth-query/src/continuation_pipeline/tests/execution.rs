use crate::application::WorthQueryDomainOperatingContext;
use crate::continuation_pipeline::WorthQueryContinuationExecutionOutcome;
use crate::ordinary_outcome::{
    WorthQueryOrdinaryContinuationCheckedTopologyKind, WorthQueryOrdinaryOutcome,
};

use super::support::{
    admitted_handle, admitted_workspace, continuation_handle_in, drifted_readmission_handle_in,
    runtime_route_request, target_request, ReadmissionDrift, RuntimeFamily,
};

#[test]
fn execution_stays_separate_from_preparation_and_produces_runtime_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    )) {
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Prepared(prepared) => {
            prepared
        }
        _ => panic!("expected prepared continuation"),
    };
    let prepared_digest = prepared.prepared_digest().to_string();

    let executed = match handle.execute_prepared_continuation(prepared) {
        WorthQueryContinuationExecutionOutcome::Executed(executed) => executed,
        _ => panic!("expected executed continuation"),
    };

    assert_eq!(
        executed.family(),
        crate::continuation_pipeline::WorthQueryPreparedContinuationFamily::BridgeRuntimeRoute
    );
    assert_ne!(executed.execution_digest(), prepared_digest);
    assert_eq!(executed.prepared().prepared_digest(), prepared_digest);
}

#[test]
fn preparation_and_execution_both_preserve_wrong_world_when_world_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, left) = admitted_workspace(request_execution, "left");
    let right = continuation_handle_in(&workspace, "right");

    let wrong_world = right.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &left,
        "face-a",
        runtime_route_request(),
    ));
    assert!(matches!(
        wrong_world,
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::WrongWorld(_)
    ));

    let prepared = match left.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &left,
        "face-a",
        runtime_route_request(),
    )) {
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Prepared(prepared) => {
            prepared
        }
        _ => panic!("expected prepared continuation"),
    };
    let execution = right.execute_prepared_continuation(prepared);
    assert!(matches!(
        execution,
        WorthQueryContinuationExecutionOutcome::WrongWorld(_)
    ));
}

#[test]
fn continuation_ordinary_outcome_keeps_execution_topology_honest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared =
        match handle.prepare_continuation_from_target_outcome(target_request::<RuntimeFamily>(
            &handle,
            "face-a",
            runtime_route_request(),
        )) {
            WorthQueryOrdinaryOutcome::Bound(prepared) => prepared,
            _ => panic!("expected bound continuation outcome"),
        };

    assert!(matches!(
        handle.execute_prepared_continuation_outcome(prepared),
        WorthQueryOrdinaryOutcome::Bound(_)
    ));

    match continuation_handle_in(&workspace, "right").execute_prepared_continuation_outcome(
        match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
            &handle,
            "face-a",
            runtime_route_request(),
        )) {
            crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Prepared(
                prepared,
            ) => prepared,
            _ => panic!("expected prepared continuation"),
        },
    ) {
        WorthQueryOrdinaryOutcome::WrongWorld(posture) => {
            assert_eq!(
                posture.checked_topology().continuation_kind(),
                Some(WorthQueryOrdinaryContinuationCheckedTopologyKind::WrongWorld)
            );
        }
        _ => panic!("expected wrong-world execution outcome"),
    }
}

#[test]
fn execution_readmission_preserves_basis_identity_for_equivalent_runtime_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_handle(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    )) {
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Prepared(prepared) => {
            prepared
        }
        _ => panic!("expected prepared continuation"),
    };
    assert_eq!(
        prepared
            .execution_readmission()
            .basis_witness()
            .basis_identity_digest(),
        handle
            .operating_context()
            .continuation_execution_readmission_observation(
                prepared.execution_readmission(),
                handle.support_snapshot()
            )
            .basis_identity_digest()
    );
}

#[test]
fn continuation_ordinary_outcome_keeps_new_execution_denials_visible() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    )) {
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Prepared(prepared) => {
            prepared
        }
        _ => panic!("expected prepared runtime continuation"),
    };

    match crate::continuation_pipeline::ordinary_outcome_from_execution_checked(
        drifted_readmission_handle_in(&workspace, "main", ReadmissionDrift::AuthorityMismatch)
            .execute_prepared_continuation_checked(prepared),
    ) {
        WorthQueryOrdinaryOutcome::AuthorityMismatch(posture) => {
            assert_eq!(
                posture.checked_topology().continuation_kind(),
                Some(WorthQueryOrdinaryContinuationCheckedTopologyKind::AuthorityMismatch)
            );
        }
        _ => panic!("expected authority mismatch ordinary outcome"),
    }
}

#[test]
fn continuation_ordinary_outcome_keeps_execution_drift_topology_visible() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, handle) = admitted_workspace(request_execution, "main");
    let prepared = match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        &handle,
        "face-a",
        runtime_route_request(),
    )) {
        crate::continuation_pipeline::WorthQueryPreparedContinuationOutcome::Prepared(prepared) => {
            prepared
        }
        _ => panic!("expected prepared runtime continuation"),
    };

    match crate::continuation_pipeline::ordinary_outcome_from_execution_checked(
        drifted_readmission_handle_in(&workspace, "main", ReadmissionDrift::AsyncRequest)
            .execute_prepared_continuation_checked(prepared),
    ) {
        WorthQueryOrdinaryOutcome::RebindRequired(posture) => {
            assert_eq!(
                posture.checked_topology().continuation_kind(),
                Some(WorthQueryOrdinaryContinuationCheckedTopologyKind::AsyncRequestDrift)
            );
        }
        _ => panic!("expected async-request-drift ordinary outcome"),
    }
}
