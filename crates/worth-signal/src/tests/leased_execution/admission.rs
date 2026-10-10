use super::support::{authority, cancellable, request};
use crate::facade::{
    mark_dirty, Aspect, AspectVersion, BoundedSignalInputs, DeclaredSignalInput,
    EvaluationRequestMode, NodeContract, SignalError, SignalGraph,
};
use std::sync::atomic::{AtomicUsize, Ordering};

const ASPECT: Aspect = Aspect::new(0);

#[test]
fn a_missing_footprint_is_refused_before_shared_epoch_evaluation() {
    let mut graph = SignalGraph::new();
    let declared = graph
        .node()
        .with_contract(NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default()))
        .build();
    let open = graph.node().build();
    let open_calls = AtomicUsize::new(0);
    let lease = authority().request_lease(request(4, 1_000_000)).unwrap();
    let result = graph.evaluate_checked(
        &[declared, open],
        EvaluationRequestMode::Default,
        &(),
        &|ctx| {
            if ctx.node() == open {
                open_calls.fetch_add(1, Ordering::SeqCst);
            }
            Ok(AspectVersion::zero().with(ASPECT, 9))
        },
        worth_execution::ExecutionRequest::leased(&lease),
    );
    let error = result.expect_err("a missing footprint must refuse checked epoch evaluation");
    assert!(
        matches!(&error, SignalError::InvalidInput { message, .. } if message == "missing bounded inputs"),
        "a missing footprint must be refused by epoch resource admission: {error:?}"
    );
    assert_eq!(
        open_calls.load(Ordering::SeqCst),
        0,
        "a missing footprint must never reach the evaluator"
    );
    assert_eq!(graph.node_aspect_version(open).unwrap().get(ASPECT), 0);
}

#[test]
fn public_stage_splits_a_declared_reader_from_its_unsettled_producer() {
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
    let mut plan = graph
        .build_evaluation_plan(&[consumer], EvaluationRequestMode::Default)
        .unwrap();
    assert_eq!(plan.stages.len(), 2);
    // Public stages are caller-editable; depth staging is not the admission proof.
    let later = plan.stages.remove(1).tasks;
    plan.stages[0].tasks.extend(later);
    plan.summary.stage_count = 1;
    plan.summary.max_stage_width = 2;
    let lease = authority().request_lease(request(4, 1_000_000)).unwrap();
    let result = graph.execute_prepared_plan_checked(
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
        worth_execution::ExecutionRequest::leased(&lease),
    );
    assert!(
        result.is_ok(),
        "an epoch must split declared conflicts before producer readiness: {result:?}"
    );
    assert_eq!(result.unwrap().stages.len(), 2);
    assert_eq!(graph.node_aspect_version(consumer).unwrap().get(ASPECT), 8);
}

#[test]
fn public_stage_splits_a_rewiring_reader_from_its_declared_producer() {
    let mut graph = SignalGraph::new();
    let empty = NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::default());
    let producer = graph.node().with_contract(empty.clone()).build();
    let alternate = graph.node().with_contract(empty).build();
    let lease = authority().request_lease(request(4, 1_000_000)).unwrap();
    graph
        .evaluate_checked(
            &[producer, alternate],
            EvaluationRequestMode::Default,
            &(),
            &|_| Ok(AspectVersion::zero().with(ASPECT, 3)),
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .unwrap();
    let consumer = graph
        .node()
        .with_contract(
            NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new([
                DeclaredSignalInput::new(producer, ASPECT),
                DeclaredSignalInput::new(alternate, ASPECT),
            ])),
        )
        .build();
    graph
        .evaluate_checked(
            &[consumer],
            EvaluationRequestMode::Default,
            &(),
            &|ctx| Ok(AspectVersion::zero().with(ASPECT, ctx.read(alternate, ASPECT)? + 1)),
            worth_execution::ExecutionRequest::leased(&lease),
        )
        .unwrap();
    mark_dirty(&mut graph, producer, ASPECT).unwrap();
    mark_dirty(&mut graph, consumer, ASPECT).unwrap();
    let mut plan = graph
        .build_evaluation_plan(&[consumer], EvaluationRequestMode::Default)
        .unwrap();
    assert_eq!(plan.stages.len(), 2);
    let later = plan.stages.remove(1).tasks;
    plan.stages[0].tasks.extend(later);
    plan.summary.stage_count = 1;
    plan.summary.max_stage_width = 2;
    let result = graph.execute_prepared_plan_checked(
        &plan,
        &(),
        &|ctx| {
            let value = if ctx.node() == producer {
                9
            } else {
                ctx.read(producer, ASPECT)? + 1
            };
            Ok(AspectVersion::zero().with(ASPECT, value))
        },
        worth_execution::ExecutionRequest::leased(&lease),
    );
    assert!(
        result.is_ok(),
        "an epoch must split a rewire's declared producer conflict: {result:?}"
    );
    assert_eq!(result.unwrap().stages.len(), 2);
    assert_eq!(graph.node_aspect_version(consumer).unwrap().get(ASPECT), 10);
    assert_eq!(
        graph
            .dependencies_of(consumer)
            .unwrap()
            .iter()
            .map(|edge| edge.source())
            .collect::<Vec<_>>(),
        vec![producer]
    );
}

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
            worth_execution::ExecutionRequest::leased(&lease),
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
            worth_execution::ExecutionRequest::leased(&lease),
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
                worth_execution::ExecutionRequest::leased(&lease),
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
