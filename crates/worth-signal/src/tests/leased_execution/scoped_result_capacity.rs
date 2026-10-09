use std::sync::atomic::{AtomicUsize, Ordering};

use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};

use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, ChangedRegion, EvaluationRequestMode, NodeContract,
    NodeEvaluationResult, PartitionSubscription, ScopePath, SignalError, SignalGraph,
};

use super::support::{authority, request};

const VALUE: Aspect = Aspect::new(0);
const REGIONS: usize = 12;

fn region(index: usize, depth: usize) -> ScopePath {
    ScopePath::new((0..depth).map(|level| format!("branch-{index}-{level}"))).unwrap()
}

fn request_with_memory(workers: usize, memory_bytes: u64) -> worth_execution::LeaseRequest {
    let mut admission = request(workers, 10_000_000);
    admission.policy = ExecutionRequestPolicy::new(
        admission.policy.posture(),
        admission.policy.determinism(),
        ExecutionBudget::new(
            admission.policy.budget().max_workers(),
            memory_bytes,
            10_000_000,
        ),
    );
    admission
}

fn output(depth: usize) -> NodeEvaluationResult {
    (0..REGIONS).fold(
        NodeEvaluationResult::from_version(AspectVersion::zero().with(VALUE, 7)),
        |result, index| {
            result.with_changed_aspect_region(VALUE, ChangedRegion::exact(region(index, depth)))
        },
    )
}

#[derive(Debug)]
enum Attempt {
    EarlyStop,
    ResultCapacityStop,
    Published(Vec<(u64, u64)>),
}

fn attempt(depth: usize, workers: usize, memory_bytes: u64) -> Attempt {
    use crate::data::error::{
        SignalExecutionFailure, SignalExecutionStopReason, SignalPublicationDisposition,
    };

    let mut graph = SignalGraph::new();
    let node = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .partitioned_output()
        .build();
    let calls = AtomicUsize::new(0);
    let lease = authority()
        .request_lease(request_with_memory(workers, memory_bytes))
        .unwrap();
    let outcome = graph.evaluate_checked(
        &[node],
        EvaluationRequestMode::Default,
        &(),
        &|_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(output(depth))
        },
        worth_execution::ExecutionRequest::leased(&lease),
    );
    match outcome {
        Err(SignalError::ExecutionStopped(stop)) => {
            assert_eq!(stop.publication_progress().completed_tasks(), 0);
            assert_eq!(graph.node_aspect_version(node).unwrap().get(VALUE), 0);
            let mut reason = stop.reason();
            while let SignalExecutionStopReason::Failure {
                cause: SignalExecutionFailure::Domain(error),
                ..
            } = reason
            {
                let SignalError::ExecutionStopped(nested) = error.as_ref() else {
                    break;
                };
                reason = nested.reason();
            }
            match reason {
                SignalExecutionStopReason::PreparationMemoryExhausted { .. }
                | SignalExecutionStopReason::Admission(_) => {
                    assert_eq!(calls.load(Ordering::SeqCst), 0);
                    Attempt::EarlyStop
                }
                SignalExecutionStopReason::Failure {
                    cause: SignalExecutionFailure::ResultCapacityExceeded,
                    ..
                } => {
                    assert_eq!(calls.load(Ordering::SeqCst), 1);
                    assert_eq!(
                        stop.disposition(),
                        SignalPublicationDisposition::WorkerLocal
                    );
                    Attempt::ResultCapacityStop
                }
                other => panic!("unexpected depth-{depth} scoped output stop at {memory_bytes} bytes: {other:?}"),
            }
        }
        Err(error) => panic!("scoped output lost typed resource result: {error}"),
        Ok(report) => {
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(report.tasks_executed, 1);
            assert!(
                report
                    .execution
                    .last()
                    .unwrap()
                    .physical()
                    .peak_charged_memory_bytes()
                    <= memory_bytes
            );
            let versions = (0..REGIONS)
                .map(|index| {
                    let path = region(index, depth);
                    let exact = PartitionSubscription::exact(path.clone());
                    let subtree = PartitionSubscription::subtree(path.prefix(1).unwrap());
                    (
                        graph
                            .node_version_for_scope(node, VALUE, Some(&exact))
                            .unwrap(),
                        graph
                            .node_version_for_scope(node, VALUE, Some(&subtree))
                            .unwrap(),
                    )
                })
                .collect::<Vec<_>>();
            assert!(versions.iter().all(|versions| *versions == (7, 7)));
            Attempt::Published(versions)
        }
    }
}

#[test]
fn full_depth_returned_regions_are_resource_bounded_and_publish_atomically() {
    const LOW: u64 = 16 * 1024;
    const FUNDED: u64 = 32 * 1024 * 1024;
    let mut minimum_by_worker = Vec::new();
    for workers in [1, 2, 4] {
        let mut minimum_by_depth = Vec::new();
        for depth in [1, 2, 4, ScopePath::MAX_DEPTH] {
            assert!(matches!(attempt(depth, workers, LOW), Attempt::EarlyStop));
            let reference = match attempt(depth, workers, FUNDED) {
                Attempt::Published(reference) => reference,
                other => panic!("funded depth-{depth} result must publish; observed {other:?}"),
            };
            let mut denied = LOW;
            let mut completed = FUNDED;
            let mut saw_result_capacity = false;
            // Search the public boundary without copying owner accounting.
            // Returned results beyond their grant stop locally; admitted
            // results publish every exact and subtree version atomically.
            while denied + 1 < completed {
                let probe = denied + (completed - denied) / 2;
                match attempt(depth, workers, probe) {
                    Attempt::EarlyStop => denied = probe,
                    Attempt::ResultCapacityStop => {
                        saw_result_capacity = true;
                        denied = probe;
                    }
                    Attempt::Published(versions) => {
                        assert_eq!(versions, reference);
                        completed = probe;
                    }
                }
            }
            assert!(
                saw_result_capacity,
                "depth-{depth} sweep must reach the result grant"
            );
            assert!(
                completed < FUNDED,
                "a constrained depth-{depth} result must publish"
            );
            assert!(matches!(
                attempt(depth, workers, completed),
                Attempt::Published(_)
            ));
            minimum_by_depth.push(completed);
        }
        assert!(
            minimum_by_depth[0] < minimum_by_depth[3],
            "shallow output must need less capacity than a depth-eight output"
        );
        minimum_by_worker.push(minimum_by_depth);
    }
    assert!(
        minimum_by_worker.windows(2).all(|pair| pair[0] == pair[1]),
        "canonical capacity and scoped results are independent of worker count"
    );
}
