use crate::facade::{
    mark_dirty, BoundedSignalInputs, DeclaredSignalInput, EvaluationRequestMode, ExecutionReport,
    NodeContract, NodeId, SignalError, SignalGraph,
};
use crate::tests::leased_execution::support::{authority, request};
use crate::tests::support::{version_ab, ASPECT_A};

use super::executor_policy::aggressive_parallel_runtime_policy;

fn contract(inputs: &[NodeId]) -> NodeContract {
    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new(
        inputs
            .iter()
            .map(|&node| DeclaredSignalInput::new(node, ASPECT_A)),
    ))
}

fn bootstrap_graph() -> Result<(SignalGraph, NodeId, NodeId, NodeId, [NodeId; 2]), SignalError> {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(aggressive_parallel_runtime_policy());
    let selector = graph.node().with_contract(contract(&[])).build();
    let left = graph.node().with_contract(contract(&[])).build();
    let right = graph.node().with_contract(contract(&[])).build();
    let targets = std::array::from_fn(|_| {
        graph
            .node()
            .with_contract(contract(&[selector, left, right]))
            .build()
    });
    let plan = graph.build_evaluation_plan(
        &[selector, left, right, targets[0], targets[1]],
        EvaluationRequestMode::ForceOnDemand,
    )?;
    graph.execute_prepared_plan(&plan, &(), &|ctx| {
        let node = ctx.node();
        let value = if node == selector {
            0
        } else if node == left {
            10
        } else if node == right {
            20
        } else {
            let selected = ctx.read_aspect_version(selector, ASPECT_A)?.get(ASPECT_A);
            ctx.read_aspect_version(if selected == 0 { left } else { right }, ASPECT_A)?
                .get(ASPECT_A)
        };
        Ok(ctx.finish(version_ab(value, 0)))
    })?;
    Ok((graph, selector, left, right, targets))
}

fn rewire_targets(
    graph: &mut SignalGraph,
    selector: NodeId,
    right: NodeId,
    targets: &[NodeId; 2],
    workers: usize,
) -> Result<ExecutionReport, SignalError> {
    mark_dirty(&mut *graph, selector, ASPECT_A)?;
    let lease = authority()
        .request_lease(request(workers, 1_000_000))
        .map_err(|_| SignalError::invalid_input("test host lease denied"))?;
    graph.evaluate_checked(
        targets,
        EvaluationRequestMode::Default,
        &(),
        &|ctx| {
            let value = if ctx.node() == selector {
                1
            } else {
                assert_eq!(ctx.read(selector, ASPECT_A)?, 1);
                ctx.read(right, ASPECT_A)?
            };
            Ok(version_ab(value, 0))
        },
        worth_execution::ExecutionRequest::leased(&lease),
    )
}

#[test]
fn leased_rewires_preserve_outputs_dependencies_and_subscribers_at_each_worker_count() {
    let (base, selector, left, right, targets) = bootstrap_graph().unwrap();
    let mut reference = base.clone();
    let reference_report = rewire_targets(&mut reference, selector, right, &targets, 1).unwrap();
    for workers in [2, 4] {
        let mut graph = base.clone();
        let report = rewire_targets(&mut graph, selector, right, &targets, workers).unwrap();
        for target in targets {
            let dependencies = graph.dependencies_of(target).unwrap();
            assert_eq!(reference.dependencies_of(target).unwrap(), dependencies);
            assert!(dependencies.iter().any(|edge| edge.source() == right));
            assert!(!dependencies.iter().any(|edge| edge.source() == left));
            assert_eq!(
                reference.node_aspect_version(target).unwrap(),
                graph.node_aspect_version(target).unwrap()
            );
            assert_eq!(graph.node_aspect_version(target).unwrap().get(ASPECT_A), 20);
        }
        assert_eq!(
            reference.subscribers_of(left).unwrap(),
            graph.subscribers_of(left).unwrap()
        );
        assert_eq!(
            reference.subscribers_of(right).unwrap(),
            graph.subscribers_of(right).unwrap()
        );
        assert!(graph.subscribers_of(left).unwrap().is_empty());
        assert_eq!(graph.subscribers_of(right).unwrap().len(), targets.len());
        assert_eq!(reference_report.tasks_executed, report.tasks_executed);
        assert_eq!(
            reference_report.execution.last().unwrap().charged_work(),
            report.execution.last().unwrap().charged_work()
        );
    }
}
