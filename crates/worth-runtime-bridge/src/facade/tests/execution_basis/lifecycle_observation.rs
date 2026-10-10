use super::*;
use crate::facade::BridgeExecutionBasisLifecycleSignalStatus;

#[test]
fn owner_observer_reports_reservation_signal_and_exact_queue_lifecycle() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let mut basis = runtime
        .admit_managed_execution_basis(
            managed_intent("observed-attempt"),
            step_contract(),
            truth_basis("snapshot-a"),
            planned_truth_view(&runtime, execution),
            execution,
        )
        .expect("managed execution should admit");
    let observer = basis.lifecycle_observer();

    let opened = observer
        .observe()
        .expect("owner observation should succeed");
    assert!(opened.reservation_active());
    assert_eq!(
        opened.signal_status(),
        Some(BridgeExecutionBasisLifecycleSignalStatus::Active)
    );
    let opened_queue = opened
        .managed_queue_pressure()
        .expect("managed basis binds one Signal queue");
    assert_eq!(opened_queue.queue_depth(), 0);
    assert_eq!(opened_queue.queue_capacity(), 4);

    let (_, occupancy) = basis
        .enqueue_managed_queue(2)
        .expect("bounded queue work should admit")
        .into_parts();
    let occupied = observer
        .observe()
        .expect("occupied basis should remain observable");
    assert!(occupied.reservation_active());
    assert_eq!(
        occupied
            .managed_queue_pressure()
            .expect("queue remains bound")
            .queue_depth(),
        2
    );

    basis
        .release_managed_queue_occupancy(occupancy)
        .expect("exact move-only occupancy should release");
    basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Completed)
        .expect("empty managed basis should finalize");

    let closed = observer
        .observe()
        .expect("terminal Signal state should remain observable");
    assert!(!closed.reservation_active());
    assert_eq!(
        closed.signal_status(),
        Some(BridgeExecutionBasisLifecycleSignalStatus::Fulfilled)
    );
    assert_eq!(
        closed
            .managed_queue_pressure()
            .expect("terminal request retains its empty queue observation")
            .queue_depth(),
        0
    );
}

#[test]
fn owner_observer_sees_drop_cancel_and_release_the_reservation() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let basis = runtime
        .admit_managed_execution_basis(
            managed_intent("dropped-attempt"),
            step_contract(),
            truth_basis("snapshot-a"),
            planned_truth_view(&runtime, execution),
            execution,
        )
        .expect("managed execution should admit");
    let observer = basis.lifecycle_observer();
    drop(basis);

    let dropped = observer
        .observe()
        .expect("dropped basis should retain terminal observation");
    assert!(!dropped.reservation_active());
    assert_eq!(
        dropped.signal_status(),
        Some(BridgeExecutionBasisLifecycleSignalStatus::Cancelled)
    );
    assert_eq!(
        dropped
            .managed_queue_pressure()
            .expect("managed queue remains observable")
            .queue_depth(),
        0
    );
}
