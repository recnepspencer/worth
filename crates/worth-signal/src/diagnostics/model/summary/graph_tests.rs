use super::GraphSummary;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::trace::{CausalityMetadata, ExecutionTraceStamp, RuntimeArtifactState};
use crate::diagnostics::policy::{DetailLimit, OrdinaryAccessLane};
use crate::diagnostics::profile::DiagnosticsTier;

struct SummaryFacts<'a> {
    active: u32,
    capacity: u32,
    tombstones: u32,
    states: (u32, u32, u32),
    edges: (u32, u32),
    partition_nodes: u32,
    artifacts: (u32, u32),
    causality: u32,
    interner_size: u32,
    dirty_samples: &'a [NodeId],
    execution_samples: &'a [NodeId],
}

fn summary(graph: &SignalGraph, limit: usize) -> GraphSummary {
    GraphSummary::from_graph(
        graph,
        DiagnosticsTier::Development,
        DetailLimit::new(limit),
        OrdinaryAccessLane,
    )
}

fn assert_summary(graph: &SignalGraph, actual: &GraphSummary, expected: SummaryFacts<'_>) {
    assert_eq!(actual.profile, DiagnosticsTier::Development);
    assert_eq!(actual.active_node_count, expected.active);
    assert_eq!(actual.arena_capacity, expected.capacity);
    assert_eq!(actual.tombstone_count, expected.tombstones);
    assert_eq!(
        (
            actual.clean_node_count,
            actual.maybe_stale_node_count,
            actual.dirty_node_count,
        ),
        expected.states
    );
    assert_eq!(
        (actual.dependency_edge_count, actual.subscriber_edge_count),
        expected.edges
    );
    assert_eq!(actual.nodes_with_partition_scopes, expected.partition_nodes);
    assert_eq!(
        (
            actual.nodes_with_trace_summary,
            actual.nodes_with_execution_record,
        ),
        expected.artifacts
    );
    assert_eq!(actual.nodes_with_causality, expected.causality);
    assert_eq!(actual.partition_interner_size, expected.interner_size);
    assert_eq!(actual.sample_dirty_nodes.as_slice(), expected.dirty_samples);
    assert_eq!(
        actual.sample_nodes_with_execution_record.as_slice(),
        expected.execution_samples
    );
    assert_eq!(actual.metrics, graph.observe().metrics());
}

fn set_state(graph: &mut SignalGraph, node: NodeId, state: NodeState) {
    graph
        .get_entry_mut(node)
        .expect("fixture node remains live")
        .set_state(state);
}

fn set_runtime_artifact(graph: &mut SignalGraph, node: NodeId, execution_record_id: Option<u64>) {
    let mut entry = graph
        .get_entry_mut(node)
        .expect("fixture node remains live");
    entry.set_runtime_artifact_state(Some(RuntimeArtifactState::default()));
    entry.set_execution_trace_stamp(Some(ExecutionTraceStamp {
        execution_record_id,
        semantic_segment_id: execution_record_id,
    }));
}

fn set_causality(graph: &mut SignalGraph, node: NodeId, kind: &str) {
    graph
        .set_causality(
            node,
            Some(CausalityMetadata {
                kind: kind.to_owned(),
                fields: Default::default(),
            }),
        )
        .expect("fixture causality write succeeds");
}

#[test]
fn exclusive_summary_preserves_all_fields_order_and_live_mutations() {
    let mut graph = SignalGraph::new();
    let nodes = (0..6).map(|_| graph.create_node()).collect::<Vec<_>>();
    set_state(&mut graph, nodes[0], NodeState::Clean);
    set_state(&mut graph, nodes[1], NodeState::MaybeStale);
    set_state(&mut graph, nodes[4], NodeState::Clean);
    graph
        .set_dependencies(nodes[2], [DependencyEdge::new(nodes[0], Aspect::new(0))])
        .unwrap();
    graph
        .set_dependencies(
            nodes[3],
            [
                DependencyEdge::whole_partition(nodes[0], Aspect::new(1), "exclusive-scope"),
                DependencyEdge::new(nodes[1], Aspect::new(0)),
            ],
        )
        .unwrap();
    set_runtime_artifact(&mut graph, nodes[3], Some(30));
    set_runtime_artifact(&mut graph, nodes[4], None);
    set_causality(&mut graph, nodes[3], "exclusive");
    graph.unregister_node(nodes[5]).unwrap();

    assert_summary(
        &graph,
        &summary(&graph, 1),
        SummaryFacts {
            active: 5,
            capacity: 6,
            tombstones: 1,
            states: (2, 1, 2),
            edges: (3, 3),
            partition_nodes: 1,
            artifacts: (2, 1),
            causality: 1,
            interner_size: 1,
            dirty_samples: &[nodes[2]],
            execution_samples: &[nodes[3]],
        },
    );

    set_state(&mut graph, nodes[1], NodeState::Clean);
    set_state(&mut graph, nodes[4], NodeState::Dirty);
    set_runtime_artifact(&mut graph, nodes[2], None);
    graph
        .get_entry_mut(nodes[3])
        .unwrap()
        .set_runtime_artifact_state(None);

    assert_summary(
        &graph,
        &summary(&graph, 2),
        SummaryFacts {
            active: 5,
            capacity: 6,
            tombstones: 1,
            states: (2, 0, 3),
            edges: (3, 3),
            partition_nodes: 1,
            artifacts: (2, 0),
            causality: 1,
            interner_size: 1,
            dirty_samples: &[nodes[2], nodes[3]],
            execution_samples: &[],
        },
    );
}

#[test]
fn forked_summary_reads_divergent_pages_segments_appends_and_recycled_slots() {
    let mut parent = SignalGraph::new();
    let nodes = (0..66)
        .map(|_| {
            let node = parent.create_node();
            set_state(&mut parent, node, NodeState::Clean);
            node
        })
        .collect::<Vec<_>>();
    parent
        .set_dependencies(nodes[64], [DependencyEdge::new(nodes[0], Aspect::new(0))])
        .unwrap();
    parent
        .set_dependencies(
            nodes[65],
            [DependencyEdge::whole_partition(
                nodes[1],
                Aspect::new(1),
                "inherited-scope",
            )],
        )
        .unwrap();
    set_state(&mut parent, nodes[64], NodeState::Clean);
    set_state(&mut parent, nodes[65], NodeState::Clean);
    set_runtime_artifact(&mut parent, nodes[63], Some(63));
    set_runtime_artifact(&mut parent, nodes[64], None);
    set_causality(&mut parent, nodes[63], "inherited");

    let (mut child, work) = parent.fork_persistent();
    assert_eq!(work.copied_mutable_graph_nodes(), 0);

    set_state(&mut parent, nodes[63], NodeState::Dirty);
    parent
        .set_dependencies(nodes[63], [DependencyEdge::new(nodes[2], Aspect::new(0))])
        .unwrap();
    let parent_append = parent.create_node();
    parent
        .set_dependencies(
            parent_append,
            [DependencyEdge::new(nodes[3], Aspect::new(0))],
        )
        .unwrap();
    set_runtime_artifact(&mut parent, nodes[65], Some(65));

    set_state(&mut child, nodes[64], NodeState::MaybeStale);
    set_state(&mut child, nodes[65], NodeState::Dirty);
    child
        .set_dependencies(
            nodes[65],
            [
                DependencyEdge::new(nodes[4], Aspect::new(0)),
                DependencyEdge::whole_partition(nodes[5], Aspect::new(1), "child-scope"),
            ],
        )
        .unwrap();
    child.unregister_node(nodes[10]).unwrap();
    let recycled = child.create_node();
    assert_eq!(recycled.index(), nodes[10].index());
    assert_ne!(recycled.generation(), nodes[10].generation());
    set_runtime_artifact(&mut child, recycled, Some(10));
    set_causality(&mut child, recycled, "child-recycled");
    let child_append = child.create_node();
    child
        .set_dependencies(
            child_append,
            [DependencyEdge::new(nodes[6], Aspect::new(0))],
        )
        .unwrap();

    assert_summary(
        &parent,
        &summary(&parent, 2),
        SummaryFacts {
            active: 67,
            capacity: 67,
            tombstones: 0,
            states: (65, 0, 2),
            edges: (4, 4),
            partition_nodes: 1,
            artifacts: (3, 2),
            causality: 1,
            interner_size: 1,
            dirty_samples: &[nodes[63], parent_append],
            execution_samples: &[nodes[63], nodes[65]],
        },
    );
    assert_summary(
        &child,
        &summary(&child, 2),
        SummaryFacts {
            active: 67,
            capacity: 67,
            tombstones: 1,
            states: (63, 1, 3),
            edges: (4, 4),
            partition_nodes: 1,
            artifacts: (3, 2),
            causality: 2,
            interner_size: 2,
            dirty_samples: &[recycled, nodes[65]],
            execution_samples: &[recycled, nodes[63]],
        },
    );
}
