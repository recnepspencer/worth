use serde_json::Value;

use super::SignalGraph;
use crate::data::aspect::Aspect;
use crate::data::dependency::DependencyEdge;
use crate::data::handle::NodeId;
use crate::data::node::NodeState;
use crate::data::output::PartitionSubscription;
use crate::diagnostics::model::summary::GraphSummary;
use crate::diagnostics::policy::{DetailLimit, OrdinaryAccessLane};
use crate::diagnostics::profile::DiagnosticsTier;

mod execution_facts;
mod node_definitions;

use execution_facts::{
    assert_forked_cold_facts, assert_populated_cold_facts, set_causality, set_execution_record,
};

#[derive(Clone, Copy, Debug)]
enum ArenaLane {
    Nodes,
    Hot,
    Warm,
    Cold,
}

impl ArenaLane {
    const ALL: [Self; 4] = [Self::Nodes, Self::Hot, Self::Warm, Self::Cold];

    const fn key(self) -> &'static str {
        match self {
            Self::Nodes => "nodes",
            Self::Hot => "hot",
            Self::Warm => "warm",
            Self::Cold => "cold",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum LengthMutation {
    Shorten,
    Lengthen,
}

struct PopulatedFixture {
    graph: SignalGraph,
    nodes: [NodeId; 7],
    recycled: NodeId,
}

struct SummaryFacts<'a> {
    arena: (u32, u32, u32),
    states: (u32, u32, u32),
    edges: (u32, u32),
    artifacts: (u32, u32),
    scope_causality_interner: (u32, u32, u32),
    dirty_samples: &'a [NodeId],
    execution_samples: &'a [NodeId],
}

fn set_state(graph: &mut SignalGraph, node: NodeId, state: NodeState) {
    graph.get_entry_mut(node).unwrap().set_state(state);
}

fn populated_fixture() -> PopulatedFixture {
    let mut graph = SignalGraph::new();
    let nodes: [NodeId; 7] = std::array::from_fn(|_| graph.create_node());
    set_state(&mut graph, nodes[0], NodeState::Clean);
    set_state(&mut graph, nodes[1], NodeState::MaybeStale);
    set_state(&mut graph, nodes[2], NodeState::Dirty);
    set_state(&mut graph, nodes[3], NodeState::Clean);
    set_state(&mut graph, nodes[4], NodeState::Dirty);
    graph
        .set_dependencies(nodes[2], [DependencyEdge::new(nodes[0], Aspect::new(0))])
        .unwrap();
    graph
        .set_dependencies(
            nodes[3],
            [
                DependencyEdge::whole_partition(nodes[0], Aspect::new(1), "serde-scope"),
                DependencyEdge::new(nodes[1], Aspect::new(0)),
            ],
        )
        .unwrap();
    set_state(&mut graph, nodes[3], NodeState::Clean);
    set_execution_record(&mut graph, nodes[3], Some(30));
    set_execution_record(&mut graph, nodes[4], None);
    set_causality(&mut graph, nodes[3], "serde-aligned");
    graph.unregister_node(nodes[5]).unwrap();
    let recycled = graph.create_node();
    assert_eq!(recycled.index(), nodes[5].index());
    assert_ne!(recycled.generation(), nodes[5].generation());
    set_state(&mut graph, recycled, NodeState::Clean);
    set_causality(&mut graph, recycled, "serde-recycled");
    graph.unregister_node(nodes[6]).unwrap();
    PopulatedFixture {
        graph,
        nodes,
        recycled,
    }
}

fn summary(graph: &SignalGraph, limit: usize) -> GraphSummary {
    GraphSummary::from_graph(
        graph,
        DiagnosticsTier::Development,
        DetailLimit::new(limit),
        OrdinaryAccessLane,
    )
}

fn assert_dependency_facts(
    graph: &SignalGraph,
    node: NodeId,
    expected: &[(NodeId, Aspect, Option<PartitionSubscription>)],
) {
    let actual = graph.dependencies_of(node).unwrap();
    assert_eq!(actual.len(), expected.len());
    for (edge, (source, aspect, scope)) in actual.iter().zip(expected) {
        assert_eq!(edge.source(), *source);
        assert_eq!(edge.aspect(), *aspect);
        assert_eq!(edge.scope_ref(), scope.as_ref());
    }
}

fn assert_summary(actual: &GraphSummary, expected: SummaryFacts<'_>) {
    assert_eq!(actual.profile, DiagnosticsTier::Development);
    let arena = (
        actual.active_node_count,
        actual.arena_capacity,
        actual.tombstone_count,
    );
    assert_eq!(arena, expected.arena);
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
    assert_eq!(
        (
            actual.nodes_with_trace_summary,
            actual.nodes_with_execution_record,
        ),
        expected.artifacts
    );
    let scope_causality_interner = (
        actual.nodes_with_partition_scopes,
        actual.nodes_with_causality,
        actual.partition_interner_size,
    );
    assert_eq!(scope_causality_interner, expected.scope_causality_interner);
    assert_eq!(actual.sample_dirty_nodes, expected.dirty_samples);
    assert_eq!(
        actual.sample_nodes_with_execution_record,
        expected.execution_samples
    );
}

fn assert_populated_facts(graph: &SignalGraph, fixture: &PopulatedFixture) {
    let nodes = fixture.nodes;
    assert_populated_cold_facts(graph, nodes[3], nodes[4], fixture.recycled);
    assert!(graph.is_alive(fixture.recycled));
    assert!(!graph.is_alive(nodes[5]));
    assert!(!graph.is_alive(nodes[6]));
    assert_eq!(graph.live_node_id_at(nodes[6].index() as usize), None);
    assert_eq!(graph.get_state(nodes[1]).unwrap(), NodeState::MaybeStale);
    assert_dependency_facts(
        graph,
        nodes[3],
        &[
            (
                nodes[0],
                Aspect::new(1),
                Some(PartitionSubscription::whole_partition("serde-scope")),
            ),
            (nodes[1], Aspect::new(0), None),
        ],
    );
    assert_eq!(
        graph.subscribers_of(nodes[0]).unwrap(),
        [nodes[2], nodes[3]]
    );
    assert_summary(
        &summary(graph, 3),
        SummaryFacts {
            arena: (6, 7, 2),
            states: (3, 1, 2),
            edges: (3, 3),
            artifacts: (0, 0),
            scope_causality_interner: (1, 2, 1),
            dirty_samples: &[nodes[2], nodes[4]],
            execution_samples: &[],
        },
    );
}

fn lane_sequence_mut<'a>(
    wire: &'a mut Value,
    graph_field: Option<&str>,
    lane: ArenaLane,
) -> &'a mut Vec<Value> {
    let graph = match graph_field {
        Some(field) => wire.get_mut(field).unwrap(),
        None => wire,
    };
    graph
        .get_mut("arena")
        .and_then(|arena| arena.get_mut(lane.key()))
        .and_then(Value::as_array_mut)
        .unwrap()
}

fn mutate_lane(
    wire: &mut Value,
    graph_field: Option<&str>,
    lane: ArenaLane,
    mutation: LengthMutation,
) {
    let sequence = lane_sequence_mut(wire, graph_field, lane);
    match mutation {
        LengthMutation::Shorten => {
            sequence.pop().expect("pop one populated lane value");
        }
        LengthMutation::Lengthen => {
            sequence.push(sequence.last().expect("populated lane").clone());
        }
    }
}

fn assert_alignment_error(error: impl std::fmt::Display) {
    let message = error.to_string();
    assert!(message.contains("signal graph arena lane lengths must match"));
    for label in ["nodes=", "hot=", "warm=", "cold="] {
        assert!(message.contains(label), "missing `{label}` in `{message}`");
    }
}

#[test]
fn aligned_graph_deserialization_preserves_literal_summary_and_identity_facts() {
    let fixture = populated_fixture();
    let wire = serde_json::to_vec(&fixture.graph).unwrap();
    let restored: SignalGraph = serde_json::from_slice(&wire).unwrap();
    assert_populated_facts(&restored, &fixture);
}

#[test]
fn graph_deserialization_rejects_every_arena_lane_length_mismatch() {
    let fixture = populated_fixture();
    let aligned = serde_json::to_value(&fixture.graph).unwrap();
    let mut completed = 0;
    for lane in ArenaLane::ALL {
        for mutation in [LengthMutation::Shorten, LengthMutation::Lengthen] {
            let mut malformed = aligned.clone();
            mutate_lane(&mut malformed, None, lane, mutation);
            let error = serde_json::from_value::<SignalGraph>(malformed)
                .expect_err("every independently misaligned arena lane must be rejected");
            assert_alignment_error(error);
            completed += 1;
        }
    }
    assert_eq!(completed, 8);
}

#[test]
fn forked_overlay_graph_round_trip_preserves_literal_current_truth() {
    let mut parent = SignalGraph::new();
    let nodes: [NodeId; 66] = std::array::from_fn(|_| {
        let node = parent.create_node();
        set_state(&mut parent, node, NodeState::Clean);
        node
    });
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
    set_execution_record(&mut parent, nodes[63], Some(63));
    set_execution_record(&mut parent, nodes[64], None);
    set_causality(&mut parent, nodes[63], "inherited");
    let (mut child, work) = parent.fork_persistent();
    assert_eq!(work.copied_mutable_graph_nodes(), 0);

    set_state(&mut parent, nodes[63], NodeState::Dirty);
    let parent_append = parent.create_node();
    parent
        .set_dependencies(
            parent_append,
            [DependencyEdge::new(nodes[3], Aspect::new(0))],
        )
        .unwrap();
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
    set_execution_record(&mut child, recycled, Some(10));
    set_causality(&mut child, recycled, "child-recycled");
    let child_append = child.create_node();
    child
        .set_dependencies(
            child_append,
            [DependencyEdge::new(nodes[6], Aspect::new(0))],
        )
        .unwrap();

    let restored: SignalGraph =
        serde_json::from_slice(&serde_json::to_vec(&child).unwrap()).unwrap();
    assert_forked_cold_facts(&restored, nodes[63], nodes[64], recycled);
    assert_eq!(parent.get_state(nodes[63]).unwrap(), NodeState::Dirty);
    assert_eq!(restored.get_state(nodes[63]).unwrap(), NodeState::Clean);
    assert!(!restored.is_alive(nodes[10]));
    assert!(restored.is_alive(recycled));
    assert_dependency_facts(
        &restored,
        nodes[65],
        &[
            (nodes[4], Aspect::new(0), None),
            (
                nodes[5],
                Aspect::new(1),
                Some(PartitionSubscription::whole_partition("child-scope")),
            ),
        ],
    );
    assert_dependency_facts(&restored, child_append, &[(nodes[6], Aspect::new(0), None)]);
    assert_summary(
        &summary(&restored, 3),
        SummaryFacts {
            arena: (67, 67, 1),
            states: (63, 1, 3),
            edges: (4, 4),
            artifacts: (0, 0),
            scope_causality_interner: (1, 2, 2),
            dirty_samples: &[recycled, nodes[65], child_append],
            execution_samples: &[],
        },
    );
}

#[test]
fn versioned_snapshot_deserialization_rejects_every_diagnostic_graph_lane_mismatch() {
    let mut fixture = populated_fixture();
    let snapshot = fixture.graph.capture_snapshot();
    let aligned = serde_json::to_value(&snapshot).unwrap();
    let restored: crate::state::SignalSnapshotV1 = serde_json::from_value(aligned.clone()).unwrap();
    assert_eq!(restored.snapshot_id(), snapshot.snapshot_id());
    assert_populated_facts(restored.diagnostic_graph(), &fixture);
    let mut completed = 1;
    for lane in ArenaLane::ALL {
        for mutation in [LengthMutation::Shorten, LengthMutation::Lengthen] {
            let mut malformed = aligned.clone();
            mutate_lane(&mut malformed, Some("diagnostic_graph"), lane, mutation);
            let error = serde_json::from_value::<crate::state::SignalSnapshotV1>(malformed)
                .expect_err("snapshot must reject a misaligned diagnostic graph");
            assert_alignment_error(error);
            completed += 1;
        }
    }
    assert_eq!(completed, 9);
}
