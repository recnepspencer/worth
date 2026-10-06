use super::support::{authority, cancellable, request};
use crate::facade::{
    Aspect, AspectVersion, BoundedSignalInputs, DeclaredSignalInput, EvaluationRequestMode,
    NodeContract, SignalError, SignalGraph,
};
use std::sync::atomic::{AtomicUsize, Ordering};

const ASPECT: Aspect = Aspect::new(0);

#[test]
fn declared_maximum_schedules_an_unsettled_producer_absent_from_current_edges() {
    let mut graph = SignalGraph::new();
    let producer = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let consumer = graph
        .node()
        .with_contract(
            NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new([
                DeclaredSignalInput::new(producer, ASPECT),
            ])),
        )
        .build();
    assert!(graph.dependencies_of(consumer).unwrap().is_empty());
    let plan = graph
        .build_evaluation_plan(&[consumer], EvaluationRequestMode::Default)
        .unwrap();
    assert_eq!(plan.summary.task_count, 2);
    let lease = authority().request_lease(request(4, 1_000_000)).unwrap();
    let report = graph
        .execute_prepared_plan_checked(
            &plan,
            &(),
            &|ctx| {
                let value = if ctx.node() == producer {
                    7
                } else {
                    ctx.read(producer, ASPECT)? + 1
                };
                Ok(AspectVersion::zero().with(ASPECT, value))
            },
            &lease,
        )
        .unwrap();
    assert_eq!(report.tasks_executed, 2);
    assert_eq!(graph.node_aspect_version(consumer).unwrap().get(ASPECT), 8);
}

#[test]
fn ignored_undeclared_read_error_cannot_publish_a_successful_output() {
    let mut graph = SignalGraph::new();
    let producer = graph.node().build();
    let consumer = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let plan = graph
        .build_evaluation_plan(&[consumer], EvaluationRequestMode::Default)
        .unwrap();
    let before = graph.node_aspect_version(consumer).unwrap();
    let lease = authority().request_lease(request(4, 1_000_000)).unwrap();
    let error = graph
        .execute_prepared_plan_checked(
            &plan,
            &(),
            &|ctx| {
                let _ignored = ctx.read(producer, ASPECT);
                Ok(AspectVersion::zero().with(ASPECT, 9))
            },
            &lease,
        )
        .unwrap_err();
    let SignalError::ExecutionStopped(stop) = error else {
        panic!("missing checked read stop")
    };
    assert_eq!(
        stop.disposition(),
        crate::data::error::SignalPublicationDisposition::WorkerLocal
    );
    assert_eq!(graph.node_aspect_version(consumer).unwrap(), before);
    assert!(graph.dependencies_of(consumer).unwrap().is_empty());
}

#[test]
fn cancelled_or_exhausted_request_does_not_invoke_the_evaluator_or_publish() {
    for cancelled in [false, true] {
        let mut graph = SignalGraph::new();
        let node = graph
            .node()
            .with_contract(
                NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()),
            )
            .build();
        let plan = graph
            .build_evaluation_plan(&[node], EvaluationRequestMode::Default)
            .unwrap();
        let before = graph.node_aspect_version(node).unwrap();
        let (request, source) = cancellable(4, if cancelled { 1_000_000 } else { 0 });
        if cancelled {
            source.cancel();
        }
        let lease = authority().request_lease(request).unwrap();
        let calls = AtomicUsize::new(0);
        let error = graph
            .execute_prepared_plan_checked(
                &plan,
                &(),
                &|_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(AspectVersion::zero().with(ASPECT, 13))
                },
                &lease,
            )
            .unwrap_err();
        let SignalError::ExecutionStopped(stop) = error else {
            panic!("missing typed request stop")
        };
        assert_eq!(
            stop.disposition(),
            crate::data::error::SignalPublicationDisposition::NoWork
        );
        assert_eq!(stop.execution().charged_work(), 0);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(graph.node_aspect_version(node).unwrap(), before);
    }
}
