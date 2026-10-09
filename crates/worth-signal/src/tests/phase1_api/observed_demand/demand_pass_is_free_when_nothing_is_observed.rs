//! Demand pass is free when nothing is observed.

use super::*;

#[test]
fn demand_pass_is_free_when_nothing_is_observed() {
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
    let evaluator = changing_evaluator(calls.clone(), chain.upstreams.clone());
    let mut summary = None;
    runtime
        .transaction(request_execution, &mut (), |tx| {
            tx.mark_dirty(chain.source, ASPECT_A)?;
            tx.evaluate_dirty(&evaluator)?;
            summary = Some(tx.evaluate_observed_demand(&evaluator)?);
            Ok(())
        })
        .unwrap();
    assert_eq!(summary.unwrap(), ObservedDemandSummary::default());
    assert_eq!(calls_for(&calls, chain.first), 0);
}
