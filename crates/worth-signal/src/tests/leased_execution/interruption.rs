//! Interrupted checked kernels cannot publish an epoch or poison later requests.
use std::sync::atomic::{AtomicUsize, Ordering};

use super::support::{authority, cancellable, request};
use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, EvaluationRequestMode, NodeContract,
    ParallelAdmissionPolicy, SignalError, SignalExecutionFailure, SignalExecutionStopReason,
    SignalGraph, SignalRuntimePolicy,
};

const VALUE: Aspect = Aspect::new(0);

#[test]
fn kernel_panic_preserves_the_epoch_and_the_authority_accepts_a_later_request() {
    assert_interrupted_epoch(true);
}

#[test]
fn cancellation_inside_a_kernel_preserves_the_epoch_despite_an_ignored_checkpoint_error() {
    assert_interrupted_epoch(false);
}

fn assert_interrupted_epoch(panic_kernel: bool) {
    let execution = authority();
    for workers in [1, 2, 4] {
        let mut graph = SignalGraph::new();
        graph.set_runtime_policy(SignalRuntimePolicy::forensic().with_parallel_admission(
            ParallelAdmissionPolicy {
                throughput_min_parallel_tasks: 1,
                balanced_min_parallel_tasks: 1,
                latency_bounded_min_parallel_tasks: 1,
                full_parallel_min_tasks: 1,
            },
        ));
        let targets = std::array::from_fn::<_, 2, _>(|_| {
            graph
                .node()
                .with_contract(
                    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
                )
                .build()
        });
        let before = targets.map(|node| {
            (
                graph.get_state(node).unwrap(),
                graph.node_aspect_version(node).unwrap(),
                graph.get_dep_snapshot(node).unwrap().clone(),
                graph.explanation_fact(node).cloned(),
                graph.provenance_fact(node).cloned(),
            )
        });
        let allocators = graph.diagnostics_state().lineage_allocator_state();
        let lineage = graph
            .diagnostics_state()
            .lineage_records()
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let (admission, cancellation) = cancellable(workers, 2_000_000);
        let lease = execution.request_lease(admission).unwrap();
        let calls = AtomicUsize::new(0);
        let error = graph
            .evaluate_checked(
                &targets,
                EvaluationRequestMode::Default,
                &(),
                &|ctx| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    if panic_kernel && ctx.node() == targets[1] {
                        panic!("checked callback failure");
                    }
                    if !panic_kernel {
                        cancellation.cancel();
                        let _ignored = ctx.work().checkpoint(1);
                    }
                    Ok(AspectVersion::zero().with(VALUE, 37))
                },
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap_err();
        let SignalError::ExecutionStopped(stop) = error else {
            panic!("missing typed interruption");
        };
        let expected = if panic_kernel {
            SignalExecutionFailure::Panic
        } else {
            SignalExecutionFailure::Cancelled
        };
        // The enclosing request preserves the nested kernel's typed failure.
        let mut kernel_reason = stop.reason();
        while let SignalExecutionStopReason::Failure {
            cause: SignalExecutionFailure::Domain(error),
            ..
        } = kernel_reason
        {
            let SignalError::ExecutionStopped(nested) = error.as_ref() else {
                break;
            };
            kernel_reason = nested.reason();
        }
        assert!(
            matches!(kernel_reason, SignalExecutionStopReason::Failure { cause, .. } if *cause == expected),
            "{stop:?}"
        );
        assert!(calls.load(Ordering::SeqCst) > 0);
        assert_eq!(stop.publication_progress().completed_epochs(), 0);
        assert_eq!(stop.publication_progress().completed_tasks(), 0);
        assert_eq!(
            graph.diagnostics_state().lineage_allocator_state(),
            allocators
        );
        assert_eq!(
            graph
                .diagnostics_state()
                .lineage_records()
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            lineage
        );
        for (node, expected) in targets.into_iter().zip(before) {
            assert_eq!(graph.get_state(node).unwrap(), expected.0);
            assert_eq!(graph.node_aspect_version(node).unwrap(), expected.1);
            assert_eq!(graph.get_dep_snapshot(node).unwrap(), &expected.2);
            assert_eq!(graph.explanation_fact(node), expected.3.as_ref());
            assert_eq!(graph.provenance_fact(node), expected.4.as_ref());
            assert!(graph.dependencies_of(node).unwrap().is_empty());
            assert!(graph.subscribers_of(node).unwrap().is_empty());
        }
        drop(lease);
        let recovery = execution
            .request_lease(request(workers, 2_000_000))
            .unwrap();
        let report = graph
            .evaluate_checked(
                &targets,
                EvaluationRequestMode::Default,
                &(),
                &|_| Ok(AspectVersion::zero().with(VALUE, 41)),
                worth_execution::ExecutionRequest::leased(&recovery),
            )
            .unwrap();
        assert_eq!(report.tasks_executed, 2);
        for node in targets {
            assert_eq!(graph.node_aspect_version(node).unwrap().get(VALUE), 41);
        }
    }
}
