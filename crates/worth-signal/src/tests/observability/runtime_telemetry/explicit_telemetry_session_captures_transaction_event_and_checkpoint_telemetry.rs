//! Explicit telemetry session captures transaction event and checkpoint telemetry.

use super::*;

#[test]
fn explicit_telemetry_session_captures_transaction_event_and_checkpoint_telemetry() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    type TestRuntime = SignalRuntime<(), (), (), (), ()>;
    let mut runtime: TestRuntime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(SignalRuntimePolicy::operational());
    let session = runtime
        .begin_observation_session(SignalObservationRequest::telemetry())
        .unwrap();
    let mut ctx = ();
    let mut eval_ctx = ();
    let mut evaluator = EmptyCheckpointEvaluator;
    let mut transaction = runtime.begin(request_execution, &mut ctx);
    transaction.emit_event(());
    transaction
        .flush_events(CheckpointBarrier::PerOperation)
        .unwrap();
    transaction
        .flush_checkpoint(
            CheckpointBarrier::PerOperation,
            &mut evaluator,
            &mut eval_ctx,
        )
        .unwrap();
    transaction.commit().unwrap();

    assert_eq!(runtime.event_bus().telemetry().checkpoint.event_flushes, 1);
    assert_eq!(
        runtime
            .checkpoint()
            .telemetry()
            .checkpoint
            .checkpoint_flushes,
        1
    );
    assert!(
        runtime
            .checkpoint()
            .telemetry()
            .checkpoint
            .checkpoint_flush_nanos
            > 0
    );
    runtime.event_bus_mut().rollback(&mut ctx);
    assert_eq!(runtime.event_bus().telemetry().checkpoint.rollback_count, 1);
    runtime.cancel_observation_session(&session).unwrap();
}
