use super::*;
use crate::facade::{AspectVersion, NodeContract, NodeEvaluationResult, SignalCheckpointDenial};
use crate::tests::leased_execution::support::{authority, cancellable};
use worth_execution::{ExecutionScan, MapKernelFailure};
use worth_foundational::PartitionIdentity;

#[test]
fn checked_result_storage_checkpoint_preserves_cancellation() {
    let mut graph = SignalGraph::new();
    let node = graph
        .node()
        .with_contract(
            NodeContract::wildcard()
                .with_bounded_inputs(BoundedSignalInputs::default())
                .with_max_checked_result_heap_bytes(8192),
        )
        .build();
    let (request, cancellation) = cancellable(1, 100_000);
    let lease = authority().request_lease(request).unwrap();
    let key = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![key], vec![(key, ())]).unwrap();
    let mut observed = None;
    let _outcome = scan.run(Some(&lease), (), 0, 0, 0, 0, |_, _, work| {
        let context = CheckedEvaluationContext::new(&graph, node, &(), work, 0, 8192)
            .map_err(MapKernelFailure::Domain)?;
        cancellation.cancel();
        observed = Some(
            context
                .into_prepared(
                    NodeEvaluationResult::from_version(AspectVersion::zero())
                        .with_output_identity("measured-output"),
                )
                .unwrap_err(),
        );
        Ok::<_, MapKernelFailure<SignalError>>(((), ()))
    });
    assert_eq!(
        observed,
        Some(SignalError::ExecutionCheckpointStopped(
            SignalCheckpointDenial::Cancelled
        ))
    );
}

#[test]
fn checked_read_checkpoint_preserves_cancellation() {
    let mut graph = SignalGraph::new();
    let source = graph.node().build();
    let node = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let (request, cancellation) = cancellable(1, 100_000);
    let lease = authority().request_lease(request).unwrap();
    let key = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![key], vec![(key, ())]).unwrap();
    let mut observed = None;
    let _outcome = scan.run(Some(&lease), (), 0, 0, 0, 0, |_, _, work| {
        let mut context = CheckedEvaluationContext::new(&graph, node, &(), work, 0, 8192)
            .map_err(MapKernelFailure::Domain)?;
        cancellation.cancel();
        observed = Some(context.read(source, Aspect::new(0)).unwrap_err());
        Ok::<_, MapKernelFailure<SignalError>>(((), ()))
    });
    assert_eq!(
        observed,
        Some(SignalError::ExecutionCheckpointStopped(
            SignalCheckpointDenial::Cancelled
        ))
    );
}
