use std::sync::atomic::{AtomicUsize, Ordering};

use super::support::{authority, request};
use crate::data::retained_storage::{RetainedStorageMeasurement, RetainedStoragePreparation};
use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, EvaluationRequestMode, NodeContract,
    NodeEvaluationResult, SignalError, SignalExecutionFailure, SignalExecutionStopReason,
    SignalGraph,
};

const VALUE: Aspect = Aspect::new(0);

fn result() -> NodeEvaluationResult {
    NodeEvaluationResult::from_version(AspectVersion::zero().with(VALUE, 7))
        .with_output_identity("declared-result")
}

fn contains_checked_result_violation(error: &SignalError) -> bool {
    match error {
        SignalError::CheckedResultCapacityExceeded { required, declared } => {
            *required > 0 && *declared == 0
        }
        SignalError::ExecutionStopped(stop) => match stop.reason() {
            SignalExecutionStopReason::Failure {
                cause: SignalExecutionFailure::Domain(nested),
                ..
            } => contains_checked_result_violation(nested),
            _ => false,
        },
        _ => false,
    }
}

#[test]
fn declared_checked_result_heap_is_enforced_before_publication() {
    let heap = result()
        .retained_heap_charge(&mut RetainedStoragePreparation::new(usize::MAX))
        .unwrap()
        .bytes();
    assert!(heap > 0);
    for maximum in [0, heap] {
        let mut graph = SignalGraph::new();
        let node = graph
            .node()
            .with_contract(
                NodeContract::wildcard()
                    .with_produces(VALUE)
                    .with_bounded_inputs(BoundedSignalInputs::default())
                    .with_max_checked_result_heap_bytes(maximum),
            )
            .build();
        let calls = AtomicUsize::new(0);
        let lease = authority().request_lease(request(2, 2_000_000)).unwrap();
        let outcome = graph.evaluate_checked(
            &[node],
            EvaluationRequestMode::Default,
            &(),
            &|_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(result())
            },
            &lease,
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        if maximum == 0 {
            let error = outcome.unwrap_err();
            assert!(contains_checked_result_violation(&error), "{error:?}");
            assert_eq!(graph.node_aspect_version(node).unwrap().get(VALUE), 0);
        } else {
            let report = outcome.unwrap();
            assert_eq!(report.tasks_executed, 1);
            assert_eq!(graph.node_aspect_version(node).unwrap().get(VALUE), 7);
        }
    }
}
