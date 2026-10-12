use super::*;

#[test]
fn proof_stops_at_compatibility_when_no_bridge_request_is_supplied() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = proof_handle(request_execution, "main");
    let proof =
        handle.orchestrate_signal_compatibility_proof(orchestration_input(&handle, "face-a"));

    assert_eq!(
        proof.request().request_kind(),
        "signal_compatibility_orchestration"
    );
    assert_eq!(proof.witness_checks().len(), 1);
    assert!(proof.witness_checks()[0].did_pass());
    assert_eq!(proof.narrowing_decisions().len(), 1);
    assert!(proof.linked_artifacts().declaration_digest().is_some());
    assert!(proof.linked_artifacts().route_plan_digest().is_some());
    assert!(proof.linked_artifacts().receipt_digest().is_some());
    assert!(proof.linked_artifacts().envelope_digest().is_some());
}

#[test]
fn wrong_world_is_preserved_through_signal_orchestration() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = proof_handle(request_execution, "left");
    let right = proof_handle(request_execution, "right");
    let outcome = right.orchestrate_signal_compatibility(orchestration_input(&left, "face-a"));
    assert!(matches!(
        outcome,
        crate::signal_compatibility_orchestration::WorthQuerySignalCompatibilityOrchestrationOutcome::WrongWorld(_)
    ));
}

#[test]
fn ordinary_outcome_keeps_signal_checked_topology_visible() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = proof_handle(request_execution, "left");
    let right = proof_handle(request_execution, "right");
    match right.orchestrate_signal_compatibility_outcome(orchestration_input(&left, "face-a")) {
        WorthQueryOrdinaryOutcome::WrongWorld(posture) => {
            assert_eq!(
                posture
                    .checked_topology()
                    .signal_compatibility_orchestration_kind(),
                Some(
                    WorthQueryOrdinarySignalCompatibilityOrchestrationCheckedTopologyKind::WrongWorld
                )
            );
            assert!(posture
                .checked_topology()
                .signal_compatibility_orchestration_linked_artifacts()
                .is_some());
        }
        _ => panic!("expected wrong-world ordinary outcome"),
    }
}

#[test]
fn ordinary_outcome_keeps_deferred_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = proof_handle(request_execution, "main");
    match handle.orchestrate_signal_compatibility_outcome(local_input::<DeferredSignalInput>(
        &handle,
        "face-deferred",
    )) {
        WorthQueryOrdinaryOutcome::Deferred(_) => {}
        _ => panic!("expected deferred ordinary outcome"),
    }
}

#[test]
fn ordinary_outcome_keeps_unsupported_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = proof_handle(request_execution, "main");
    match handle.orchestrate_signal_compatibility_outcome(local_input::<UnsupportedSignalInput>(
        &handle,
        "face-unsupported",
    )) {
        WorthQueryOrdinaryOutcome::Unsupported(_) => {}
        _ => panic!("expected unsupported ordinary outcome"),
    }
}

#[test]
fn ordinary_outcome_keeps_basis_mismatch_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let handle = admitted_basis_mismatch_handle(request_execution);
    match handle
        .orchestrate_signal_compatibility_outcome(basis_mismatch_input(&handle, "face-basis"))
    {
        WorthQueryOrdinaryOutcome::BasisMismatch(_) => {}
        _ => panic!("expected basis-mismatch ordinary outcome"),
    }
}

#[test]
fn ordinary_outcome_keeps_wrong_handle_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left = proof_handle(request_execution, "shared");
    let right = admitted_same_world_different_handle(request_execution);
    match right.orchestrate_signal_compatibility_outcome(orchestration_input(&left, "face-handle"))
    {
        WorthQueryOrdinaryOutcome::WrongHandle(posture) => {
            assert_eq!(
                posture
                    .checked_topology()
                    .signal_compatibility_orchestration_kind(),
                Some(
                    WorthQueryOrdinarySignalCompatibilityOrchestrationCheckedTopologyKind::WrongHandle
                )
            );
        }
        _ => panic!("expected wrong-handle ordinary outcome"),
    }
}
