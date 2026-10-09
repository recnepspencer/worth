//! Demanded recompute that does not change is not a meaningful change.

use super::*;

#[test]
fn demanded_recompute_that_does_not_change_is_not_a_meaningful_change() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (mut runtime, chain) = build_chain();
    let calls: CallLog = Arc::default();
    let constant = constant_computeds_evaluator(calls.clone(), chain.upstreams.clone());
    settle_on_demand(&mut runtime, &[chain.second], &constant);

    let notices = Arc::new(Mutex::new(Vec::new()));
    runtime.observe_nodes(
        ObservationPolicy::meaningful_change(),
        [chain.second],
        Box::new(RecordingListener {
            notices: Arc::clone(&notices),
        }),
    );
    let mut summary = None;
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty(chain.source, ASPECT_A)?;
            tx.evaluate_dirty(&constant)?;
            summary = Some(tx.evaluate_observed_demand(&constant)?);
            Ok(())
        })
        .unwrap();

    // `first` is recomputed because the source changed; its output is
    // identical, so `second` is never invalidated and never runs.
    assert_eq!(
        summary.unwrap(),
        ObservedDemandSummary {
            reach_visits: 3,
            targets: 1,
            passes: 2,
            tasks_executed: 1,
        }
    );
    assert_eq!(calls_for(&calls, chain.first), 2);
    assert_eq!(calls_for(&calls, chain.second), 1);
    assert!(
        notices
            .lock()
            .expect("observed demand notices mutex poisoned")
            .is_empty(),
        "meaningful-change watchers stay quiet when the demanded value is unchanged"
    );
}
