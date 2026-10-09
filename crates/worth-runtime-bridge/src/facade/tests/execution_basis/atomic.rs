use super::*;
use crate::facade::{BridgeExecutionPosture, BridgeExecutionSafePointFailureKind};

#[test]
fn atomic_basis_fulfills_exact_lifecycle_without_managed_capabilities() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let basis = runtime
        .admit_atomic_execution_basis(
            managed_intent("atomic-completion"),
            truth_basis("snapshot-a"),
            planned_truth_view(&runtime),
        )
        .expect("matching atomic intent and truth should admit");
    let handle = basis.request().request_handle();
    assert_eq!(basis.execution_posture(), BridgeExecutionPosture::Atomic);
    assert!(basis.step_contract().is_none());
    assert_eq!(basis.counters().signal_attempt_admission_count(), 1);
    assert_eq!(basis.counters().signal_queue_binding_count(), 0);
    assert_eq!(
        basis.managed_intent().resource_attempt_identity(),
        "atomic-completion"
    );
    let observer = basis.lifecycle_observer();
    let active = observer.observe().unwrap();
    assert!(active.reservation_active());
    assert!(active.managed_queue_pressure().is_none());
    assert_eq!(
        signal_status(&runtime, handle),
        ResourceInFlightStatus::Active
    );

    let receipt = basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Completed)
        .expect("atomic completion terminalizes the actual Signal attempt");
    assert!(receipt.signal_transition_performed());
    assert!(receipt.reservation_released());
    assert_eq!(
        receipt.signal_terminal(),
        BridgeExecutionBasisSignalTerminal::Fulfilled
    );
    assert_eq!(
        signal_status(&runtime, handle),
        ResourceInFlightStatus::Fulfilled
    );
    assert!(!observer.observe().unwrap().reservation_active());
}

#[test]
fn atomic_basis_refuses_managed_actions_and_drop_cancels_exact_attempt() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let intent = managed_intent("atomic-drop");
    let mut basis = runtime
        .admit_atomic_execution_basis(
            intent.clone(),
            truth_basis("snapshot-a"),
            planned_truth_view(&runtime),
        )
        .unwrap();
    let handle = basis.request().request_handle();
    let observer = basis.lifecycle_observer();
    let duplicate = runtime
        .admit_atomic_execution_basis(
            intent.clone(),
            truth_basis("snapshot-a"),
            planned_truth_view(&runtime),
        )
        .expect_err("a second live basis cannot reserve the same intent");
    assert_eq!(
        duplicate.kind(),
        BridgeExecutionBasisDenialKind::ManagedExecutionIntentAlreadyReserved
    );
    assert_eq!(duplicate.counters().signal_attempt_admission_count(), 0);
    assert_eq!(
        basis.observe_safe_point().unwrap_err().kind(),
        BridgeExecutionSafePointFailureKind::AtomicExecutionUnsupported
    );
    assert_eq!(
        basis.enqueue_managed_queue(1).unwrap_err().kind(),
        BridgeManagedQueueFailureKind::AtomicExecutionUnsupported
    );
    let failure = match basis.yield_execution_basis() {
        Err(failure) => failure,
        Ok(_) => panic!("atomic execution cannot mint yielded authority"),
    };
    assert_eq!(
        failure.kind(),
        BridgeExecutionBasisFinalizationFailureKind::AtomicExecutionUnsupported
    );
    let basis = failure.into_basis();
    let failure = basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Yielded)
        .unwrap_err();
    assert_eq!(
        failure.kind(),
        BridgeExecutionBasisFinalizationFailureKind::AtomicExecutionUnsupported
    );
    let basis = failure.into_basis();
    assert_eq!(
        signal_status(&runtime, handle),
        ResourceInFlightStatus::Active
    );
    assert!(observer.observe().unwrap().reservation_active());
    assert!(observer
        .observe()
        .unwrap()
        .managed_queue_pressure()
        .is_none());
    drop(basis);
    assert_eq!(
        signal_status(&runtime, handle),
        ResourceInFlightStatus::Cancelled
    );
    assert!(!observer.observe().unwrap().reservation_active());
    runtime
        .admit_atomic_execution_basis(
            intent,
            truth_basis("snapshot-a"),
            planned_truth_view(&runtime),
        )
        .unwrap()
        .finalize(BridgeExecutionBasisTerminalDisposition::Cancelled)
        .unwrap();
}
