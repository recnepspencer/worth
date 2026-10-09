//! The default apply threshold still uses the checked, atomic packet owner.
use std::sync::atomic::{AtomicUsize, Ordering};

use super::support::{authority, cancellable, request};
use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, EvaluationRequestMode, NodeContract, SignalError,
    SignalExecutionFailure, SignalExecutionStopReason, SignalGraph,
};

const VALUE: Aspect = Aspect::new(0);

fn fixture() -> (SignalGraph, crate::facade::NodeId) {
    let mut graph = SignalGraph::new();
    let target = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    (graph, target)
}

fn terminal_reason(mut reason: &SignalExecutionStopReason) -> &SignalExecutionStopReason {
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
    reason
}

#[test]
fn default_apply_threshold_preserves_singleton_on_cancel_and_last_work_unit() {
    for workers in [1, 2, 4] {
        let (mut funded, target) = fixture();
        let lease = authority()
            .request_lease(request(workers, 20_000_000))
            .unwrap();
        let complete = funded
            .evaluate_checked(
                &[target],
                EvaluationRequestMode::Default,
                &(),
                &|_| Ok(AspectVersion::zero().with(VALUE, 29)),
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap();
        let completed_work = complete.execution.last().unwrap().charged_work();
        assert!(completed_work > 1);
        drop(lease);

        let (mut graph, target) = fixture();
        let calls = AtomicUsize::new(0);
        let lease = authority()
            .request_lease(request(workers, completed_work - 1))
            .unwrap();
        let error = graph
            .evaluate_checked(
                &[target],
                EvaluationRequestMode::Default,
                &(),
                &|_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(AspectVersion::zero().with(VALUE, 29))
                },
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap_err();
        let SignalError::ExecutionStopped(stop) = error else {
            panic!("expected typed work stop");
        };
        assert!(
            matches!(
                terminal_reason(stop.reason()),
                SignalExecutionStopReason::WorkExhausted { .. }
            ),
            "{stop:?}"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(stop.publication_progress().completed_epochs(), 0);
        assert_eq!(graph.node_aspect_version(target).unwrap().get(VALUE), 0);
        drop(lease);

        let (mut graph, target) = fixture();
        let calls = AtomicUsize::new(0);
        let (admission, cancellation) = cancellable(workers, 20_000_000);
        let lease = authority().request_lease(admission).unwrap();
        let error = graph
            .evaluate_checked(
                &[target],
                EvaluationRequestMode::Default,
                &(),
                &|ctx| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    cancellation.cancel();
                    let _ignored = ctx.work().checkpoint(1);
                    Ok(AspectVersion::zero().with(VALUE, 31))
                },
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap_err();
        let SignalError::ExecutionStopped(stop) = error else {
            panic!("expected typed cancellation stop");
        };
        assert!(
            matches!(
                terminal_reason(stop.reason()),
                SignalExecutionStopReason::Failure {
                    cause: SignalExecutionFailure::Cancelled,
                    ..
                }
            ),
            "{stop:?}"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(stop.publication_progress().completed_epochs(), 0);
        assert_eq!(graph.node_aspect_version(target).unwrap().get(VALUE), 0);
    }
}
