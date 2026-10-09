use crate::facade::{DependencyBatchEdit, DependencyEdge, NodeId, SignalGraph};

pub(crate) fn with_perf_topology_asserts_disabled<T>(run: impl FnOnce() -> T) -> T {
    SignalGraph::with_topology_debug_asserts_skipped(run)
}

pub(crate) fn build_chain_graph(count: usize) -> (SignalGraph, Vec<NodeId>) {
    let mut graph = SignalGraph::new();
    graph.reserve_node_capacity(count);
    let mut chain = Vec::with_capacity(count);
    let mut dependency_edits = Vec::with_capacity(count.saturating_sub(1));

    let first = graph.create_node();
    chain.push(first);

    for index in 1..count {
        let node = graph.create_node();
        dependency_edits.push((
            node,
            vec![DependencyEdge::new(
                chain[index - 1],
                crate::tests::support::ASPECT_B,
            )],
        ));
        chain.push(node);
    }

    graph
        .apply_dependency_batch_edit(&DependencyBatchEdit::from_pairs(dependency_edits))
        .unwrap();

    (graph, chain)
}
