use crate::facade::{DependencyEdge, EvaluationRequestMode};
use crate::tests::performance_support::{build_chain_graph, with_perf_topology_asserts_disabled};
use crate::tests::support::ASPECT_B;

#[test]
fn batch_bootstrap_admits_each_declared_chain_edge() {
    let (mut graph, chain) = with_perf_topology_asserts_disabled(|| build_chain_graph(10_000));
    assert_eq!(chain.len(), 10_000);
    assert_eq!(graph.active_node_count(), 10_000);
    assert!(graph.dependencies_of(chain[0]).unwrap().is_empty());
    for pair in chain.windows(2) {
        assert_eq!(
            graph.dependencies_of(pair[1]).unwrap(),
            &[DependencyEdge::new(pair[0], ASPECT_B)]
        );
    }
    let plan = graph
        .build_evaluation_plan(&chain, EvaluationRequestMode::Default)
        .unwrap();
    assert_eq!(plan.summary.task_count, 10_000);
}
