//! A real request must account for private preparation before its atomic commit.
use std::sync::atomic::{AtomicUsize, Ordering};

use super::support::{authority, request};
use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, EvaluationRequestMode, NodeContract, NodeId,
    SignalError, SignalGraph, SignalRuntimePolicy,
};

const VALUE: Aspect = Aspect::new(0);

fn fixture() -> (SignalGraph, [NodeId; 2]) {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic().with_parallel_admission(
        crate::runtime_policy::ParallelAdmissionPolicy {
            throughput_min_parallel_tasks: 1,
            balanced_min_parallel_tasks: 1,
            latency_bounded_min_parallel_tasks: 1,
            full_parallel_min_tasks: 1,
        },
    ));
    let targets = std::array::from_fn(|_| {
        graph
            .node()
            .with_contract(
                NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
            )
            .build()
    });
    (graph, targets)
}

#[test]
fn final_preparation_work_exhaustion_after_evaluation_preserves_the_epoch() {
    let (mut baseline, targets) = fixture();
    let lease = authority().request_lease(request(1, 2_000_000)).unwrap();
    let complete = baseline
        .evaluate_checked(
            &targets,
            EvaluationRequestMode::Default,
            &(),
            &|_| Ok(AspectVersion::zero().with(VALUE, 37)),
            &lease,
        )
        .unwrap();
    let complete_work = complete.execution.last().unwrap().charged_work();
    assert!(complete_work > 1);
    drop(lease);

    for workers in [1, 2, 4] {
        let (mut graph, targets) = fixture();
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
        let calls = AtomicUsize::new(0);
        // Derive the last admissible boundary from actual complete execution,
        // rather than mirroring the implementation's operation-unit formulas.
        let lease = authority()
            .request_lease(request(workers, complete_work - 1))
            .unwrap();
        let error = graph
            .evaluate_checked(
                &targets,
                EvaluationRequestMode::Default,
                &(),
                &|_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(AspectVersion::zero().with(VALUE, 37))
                },
                &lease,
            )
            .unwrap_err();
        let SignalError::ExecutionStopped(stop) = error else {
            panic!("missing typed stop")
        };
        assert!(
            matches!(
                stop.reason(),
                crate::data::error::SignalExecutionStopReason::WorkExhausted { .. }
            ),
            "{stop:?}"
        );
        assert_eq!(calls.load(Ordering::SeqCst), targets.len());
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
    }
}
