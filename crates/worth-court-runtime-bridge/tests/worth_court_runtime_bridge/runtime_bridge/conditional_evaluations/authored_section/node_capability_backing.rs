//! The capability owner accounts for contract storage outside a Bridge section.
use super::*;
use worth_signal::facade::{PartitionSubscription, SignalGraph};

#[test]
fn installed_node_backing_report_includes_every_owned_partition_contract_allocation() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let mut graph = Box::new(SignalGraph::new());
    let baseline = graph.node().build();
    let scoped = graph
        .node()
        .with_partition_scope(PartitionSubscription::whole_partition(
            "authored-scope".repeat(8192),
        ))
        .build();
    let contract = graph.get_contract(scoped).unwrap();
    let mut expected = 0usize;
    for roster in [
        &contract.semantics.partition_scope,
        &contract.projection.consumes_partitions,
    ] {
        let roster = roster.as_ref().unwrap();
        expected += roster.capacity() * std::mem::size_of::<PartitionSubscription>();
        for subscription in roster {
            // ScopePath::one constructs this one-element backing from an array.
            assert_eq!(subscription.path().segments().len(), 1);
            expected += std::mem::size_of::<String>();
            expected += subscription.path().segments()[0].capacity();
        }
    }
    let base = graph
        .installed_node_capability_backing_bytes(baseline, &execution)
        .unwrap();
    let actual = graph
        .installed_node_capability_backing_bytes(scoped, &execution)
        .unwrap();
    assert_eq!(actual - base, expected as u64);
}

#[test]
fn installed_node_backing_report_preserves_the_original_work_ceiling_reason() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(1)),
    );
    let funding = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    let mut graph = Box::new(SignalGraph::new());
    let node = graph.node().build();
    assert!(matches!(
        graph.installed_node_capability_backing_bytes(node, &ExecutionRequest::serial(&funding)),
        Err(
            worth_signal::facade::SignalError::ExecutionCheckpointStopped(
                worth_signal::facade::SignalCheckpointDenial::WorkCeiling
            )
        )
    ));
}
