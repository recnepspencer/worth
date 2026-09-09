use super::*;
use crate::facade::{Aspect, SignalRuntimePolicy};

fn diamond(population: usize, maximum: usize) -> (SignalGraph, Vec<NodeId>) {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(
        SignalRuntimePolicy::development().with_maximum_upstream_dependency_visits(maximum),
    );
    let nodes = (0..population)
        .map(|_| graph.node().build())
        .collect::<Vec<_>>();
    for target in [1, 2] {
        graph
            .set_dependencies(
                nodes[target],
                [DependencyEdge::new(nodes[0], Aspect::new(0))],
            )
            .unwrap();
    }
    graph
        .set_dependencies(
            nodes[3],
            [
                DependencyEdge::new(nodes[1], Aspect::new(0)),
                DependencyEdge::new(nodes[2], Aspect::new(0)),
            ],
        )
        .unwrap();
    // Canonical topology/state fixture; no provider execution is claimed here.
    for node in &nodes[..4] {
        graph.transition_node_clean(*node).unwrap();
    }
    (graph, nodes)
}

#[test]
fn upstream_exact_edge_limit_counts_duplicate_diamond_edges() {
    let (graph, nodes) = diamond(4, 4);
    assert_eq!(graph.has_current_unsettled_upstream(nodes[3]), Ok(false));
    let (short, nodes) = diamond(4, 3);
    assert_eq!(
        short.has_current_unsettled_upstream(nodes[3]),
        Err(SignalError::UpstreamDependencyWorkExhausted { maximum_visits: 3 })
    );
}

#[test]
fn upstream_scratch_and_work_ignore_unrelated_graph_population() {
    let mut baseline = None;
    for population in [4, 64, 4096] {
        let (graph, nodes) = diamond(population, 4);
        let mut traversal =
            UpstreamTraversal::new(graph.current_runtime_dependencies_of(nodes[3]).unwrap(), 4);
        assert_eq!(traversal.has_unsettled(&graph, &mut |_| Ok(())), Ok(false));
        let facts = (
            traversal.visits,
            traversal.visited.len(),
            traversal.stack.capacity(),
        );
        assert_eq!((facts.0, facts.1), (4, 3));
        assert!(graph.arena_capacity() >= population);
        match baseline {
            None => baseline = Some(facts),
            Some(expected) => assert_eq!(facts, expected),
        }
    }
}

#[test]
fn upstream_preserves_reverse_edge_order_and_early_unsettled_return() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(
        SignalRuntimePolicy::development().with_maximum_upstream_dependency_visits(1),
    );
    let clean = graph.node().build();
    let dirty = graph.node().build();
    let target = graph.node().build();
    graph.transition_node_clean(clean).unwrap();
    graph
        .set_dependencies(
            target,
            [
                DependencyEdge::new(clean, Aspect::new(0)),
                DependencyEdge::new(dirty, Aspect::new(0)),
            ],
        )
        .unwrap();
    assert_eq!(graph.has_current_unsettled_upstream(target), Ok(true));
    assert_eq!(graph.has_current_unsettled_upstream(clean), Ok(false));
}

#[test]
fn upstream_deep_chain_is_iterative_and_stale_handles_are_validated() {
    let mut graph = SignalGraph::new();
    graph.set_runtime_policy(
        SignalRuntimePolicy::development().with_maximum_upstream_dependency_visits(127),
    );
    let nodes = (0..128).map(|_| graph.node().build()).collect::<Vec<_>>();
    for pair in nodes.windows(2) {
        graph
            .set_dependencies(pair[1], [DependencyEdge::new(pair[0], Aspect::new(0))])
            .unwrap();
    }
    for node in &nodes {
        graph.transition_node_clean(*node).unwrap();
    }
    assert_eq!(graph.has_current_unsettled_upstream(nodes[127]), Ok(false));
    graph.unregister_node(nodes[0]).unwrap();
    // Exercise an explicitly stale retained edge rather than fabricating an ID.
    graph
        .inject_retired_dependency_for_test(nodes[1], nodes[0], Aspect::new(0))
        .unwrap();
    let expected = graph.get_state(nodes[0]).unwrap_err();
    assert!(matches!(expected, SignalError::StaleHandle { .. }));
    assert_eq!(
        graph.has_current_unsettled_upstream(nodes[1]),
        Err(expected)
    );
}

#[test]
fn conditional_upstream_spends_the_existing_attempt_before_tree_or_stack_growth() {
    use crate::data::retained_storage::RetainedStoragePreparation;
    let (graph, nodes) = diamond(64, 4);
    let mut full = RetainedStoragePreparation::new(1_000);
    full.reserve_visits(7).unwrap();
    assert_eq!(
        graph.conditional_has_current_unsettled_upstream(nodes[3], &mut full),
        Ok(false)
    );
    let spent = full.visits();
    for limit in [spent - 1, spent] {
        let mut work = RetainedStoragePreparation::new(limit);
        work.reserve_visits(7).unwrap();
        let result = graph.conditional_has_current_unsettled_upstream(nodes[3], &mut work);
        if limit == spent {
            assert_eq!(result, Ok(false));
            assert_eq!(work.visits(), spent);
        } else {
            assert_eq!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted {
                    maximum_visits: limit
                })
            );
            assert!(work.visits() >= 7);
        }
    }
    let mut empty = RetainedStoragePreparation::new(0);
    assert_eq!(
        graph.conditional_has_current_unsettled_upstream(nodes[3], &mut empty),
        Err(SignalError::ConditionalEvaluationWorkExhausted { maximum_visits: 0 })
    );
    assert_eq!(empty.visits(), 0);
}
