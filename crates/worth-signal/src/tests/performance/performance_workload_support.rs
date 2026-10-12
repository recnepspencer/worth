use crate::facade::{EvaluationRequestMode, NodeId, SignalGraph};
use crate::tests::support::{version_ab, GraphDependencyBatchExt, ASPECT_B};

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

pub(super) fn evaluate_chain_bulk(graph: &mut SignalGraph, chain: &[NodeId]) {
    let plan = graph
        .build_evaluation_plan(chain, EvaluationRequestMode::Default)
        .unwrap();
    graph
        .execute_prepared_plan(&plan, &(), &|_ctx| Ok(version_ab(0, 1)))
        .unwrap();
}
