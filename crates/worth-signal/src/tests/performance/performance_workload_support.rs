use crate::facade::{EvaluationRequestMode, NodeId, SignalGraph};
use crate::tests::support::{evaluate, version_ab, GraphDependencyBatchExt, ASPECT_B};
use std::time::Instant;

pub(super) fn build_chain_graph(count: usize) -> (SignalGraph, Vec<NodeId>) {
    let mut graph = SignalGraph::new();
    let mut chain: Vec<NodeId> = Vec::with_capacity(count);

    let first = graph.create_node();
    chain.push(first);

    for i in 1..count {
        let node = graph.create_node();
        graph
            .append_dependency(node, chain[i - 1], ASPECT_B)
            .unwrap();
        chain.push(node);
    }

    (graph, chain)
}

pub(super) fn with_perf_topology_asserts_disabled<T>(run: impl FnOnce() -> T) -> T {
    SignalGraph::with_topology_debug_asserts_skipped(run)
}

pub(super) fn evaluate_chain(graph: &mut SignalGraph, chain: &[NodeId]) {
    let mut compute = |_id, _g: &SignalGraph| Ok(version_ab(0, 1));
    for node in chain {
        evaluate(&mut *graph, *node, &mut compute).unwrap();
    }
}

pub(super) fn evaluate_chain_bulk(graph: &mut SignalGraph, chain: &[NodeId]) {
    let plan = graph
        .build_evaluation_plan(chain, EvaluationRequestMode::Default)
        .unwrap();
    graph
        .execute_prepared_plan(&plan, &(), &|_ctx| Ok(version_ab(0, 1)))
        .unwrap();
}

pub(super) fn evaluate_chain_bulk_profile(
    graph: &mut SignalGraph,
    chain: &[NodeId],
) -> (u128, u128, u64, u64, u128) {
    graph.reset_telemetry();
    let plan_start = Instant::now();
    let plan = graph
        .build_evaluation_plan(chain, EvaluationRequestMode::Default)
        .unwrap();
    let plan_elapsed_ms = plan_start.elapsed().as_millis();
    let execute_start = Instant::now();
    graph
        .execute_prepared_plan(&plan, &(), &|_ctx| Ok(version_ab(0, 1)))
        .unwrap();
    let execute_elapsed_ms = execute_start.elapsed().as_millis();
    let telemetry = *graph.telemetry();
    (
        plan_elapsed_ms,
        execute_elapsed_ms,
        telemetry.planner.plans_built,
        telemetry.planner.tasks_scheduled,
        telemetry.execution.stage_execution_nanos / 1_000_000,
    )
}

pub(super) fn warm_up_bulk_evaluation_path() {
    let (mut graph, chain) = with_perf_topology_asserts_disabled(|| build_chain_graph(8));
    let _ = evaluate_chain_bulk_profile(&mut graph, &chain);
}

pub(super) fn median_u128(values: &mut [u128]) -> u128 {
    values.sort_unstable();
    values[values.len() / 2]
}
