//! Event bus begin failures record failure and rollback diagnostics.

use super::*;

#[test]
fn event_bus_begin_failures_record_failure_and_rollback_diagnostics() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .with_events::<DiagnosticsEvent>()
        .with_domains::<DiagnosticsDomain>()
        .build();
    let node = runtime.graph_mut().node().build();
    runtime
        .event_bus_mut()
        .subscribe(Box::new(NeedsMissingProviderSubscriber))
        .unwrap();

    let mut runtime_ctx = ();
    let err = runtime
        .transaction(request_execution, &mut runtime_ctx, |tx| {
            tx.mark_dirty(node, ASPECT_A)?;
            Ok(())
        })
        .unwrap_err();
    assert!(format!("{err}").contains("event bus begin failed"));

    let failure = runtime
        .observe()
        .latest_failure_diagnostics()
        .expect("begin failure diagnostics should be retained");
    assert_eq!(failure.phase, ExecutionFailurePhase::CommitPromotion);
    let rollback = runtime
        .observe()
        .latest_rollback_diagnostics()
        .expect("begin rollback diagnostics should be retained");
    assert!(rollback.rolled_back);
    assert!(rollback
        .reason
        .as_deref()
        .unwrap_or_default()
        .contains("event bus begin failed"));
}
