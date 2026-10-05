use std::sync::atomic::{AtomicUsize, Ordering};

use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, DeclaredSignalInput, EvaluationRequestMode,
    NodeContract, SignalError, SignalGraph,
};
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};

use super::support::{authority, request};

const VALUE: Aspect = Aspect::new(0);

#[test]
fn framework_preparation_denial_precedes_evaluator_and_publication() {
    let mut graph = SignalGraph::new();
    let targets = (0..128)
        .map(|_| {
            graph
                .node()
                .with_contract(
                    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
                )
                .build()
        })
        .collect::<Vec<_>>();
    let before = targets
        .iter()
        .map(|&node| graph.node_aspect_version(node).unwrap())
        .collect::<Vec<_>>();
    let mut admission = request(4, 1_000_000);
    admission.policy = ExecutionRequestPolicy::new(
        admission.policy.posture(),
        admission.policy.determinism(),
        ExecutionBudget::new(
            admission.policy.budget().max_workers(),
            16 * 1024,
            1_000_000,
        ),
    );
    let lease = authority().request_lease(admission).unwrap();
    let calls = AtomicUsize::new(0);
    let error = graph
        .evaluate_checked(
            &targets,
            EvaluationRequestMode::Default,
            &(),
            &|_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(AspectVersion::zero().with(VALUE, 99))
            },
            &lease,
        )
        .unwrap_err();
    let SignalError::ExecutionStopped(stop) = error else {
        panic!("missing typed stop")
    };
    assert!(
        matches!(stop.reason(), crate::data::error::SignalExecutionStopReason::PreparationMemoryExhausted {
        required: Some(required), reserved, ..
    } if required > reserved)
    );
    assert_eq!(
        stop.disposition(),
        crate::data::error::SignalPublicationDisposition::NoWork
    );
    assert_eq!(stop.publication_progress().completed_tasks(), 0);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        targets
            .iter()
            .map(|&node| graph.node_aspect_version(node).unwrap())
            .collect::<Vec<_>>(),
        before
    );
}

#[test]
fn later_epoch_stop_preserves_committed_graph_prefix_in_the_outcome() {
    let mut graph = SignalGraph::new();
    let producer = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let consumer = graph
        .node()
        .with_contract(
            NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new([
                DeclaredSignalInput::new(producer, VALUE),
            ])),
        )
        .build();
    let before_consumer = graph.node_aspect_version(consumer).unwrap();
    let lease = authority().request_lease(request(4, 1_000_000)).unwrap();
    let error = graph
        .evaluate_checked(
            &[consumer],
            EvaluationRequestMode::Default,
            &(),
            &|ctx| {
                if ctx.node() == producer {
                    return Ok(AspectVersion::zero().with(VALUE, 901));
                }
                let _ignored = ctx.read(producer, Aspect::new(1));
                Ok(AspectVersion::zero().with(VALUE, 902))
            },
            &lease,
        )
        .unwrap_err();
    let SignalError::ExecutionStopped(stop) = error else {
        panic!("missing typed stop")
    };
    let progress = stop.publication_progress();
    assert_eq!(
        progress.disposition(),
        crate::data::error::SignalPublicationDisposition::WorkerLocal
    );
    assert_eq!(progress.completed_epochs(), 1);
    assert_eq!(progress.completed_tasks(), 1);
    assert_eq!(graph.node_aspect_version(producer).unwrap().get(VALUE), 901);
    assert_eq!(
        graph.node_aspect_version(consumer).unwrap(),
        before_consumer
    );
    assert!(graph.dependencies_of(consumer).unwrap().is_empty());
}
