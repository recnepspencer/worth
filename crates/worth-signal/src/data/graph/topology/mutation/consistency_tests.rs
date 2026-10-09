use crate::data::dependency::{CanonicalDependencies, DependencyEdge};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::tests::support::ASPECT_A;

#[test]
fn seeded_local_mutations_agree_with_the_whole_graph_oracle() -> Result<(), SignalError> {
    for seed in [1, 7, 41, 73, 101, 509, 1_024, 4_096] {
        let mut random = MutationSequence(seed);
        let mut graph = SignalGraph::new();
        let nodes = (0..12).map(|_| graph.create_node()).collect::<Vec<_>>();
        for consumer in 1..nodes.len() {
            let desired = random.dependencies(&nodes[..consumer]);
            graph.reconcile_dependencies(nodes[consumer], desired.as_slice())?;
        }
        graph.assert_bidirectional_consistency()?;
        for _ in 0..64 {
            match random.next() % 3 {
                0 => reconcile_random_node(&mut graph, &nodes, &mut random)?,
                1 => reconcile_random_batch(&mut graph, &nodes, &mut random)?,
                _ => repair_random_membership(&mut graph, &nodes, &mut random)?,
            }
            graph.assert_bidirectional_consistency()?;
        }
    }
    Ok(())
}

fn reconcile_random_node(
    graph: &mut SignalGraph,
    nodes: &[NodeId],
    random: &mut MutationSequence,
) -> Result<(), SignalError> {
    let index = 1 + random.next() as usize % (nodes.len() - 1);
    let node = nodes[index];
    let former = sources(graph, node)?;
    let desired = random.dependencies(&nodes[..index]);
    let desired_sources = desired
        .as_slice()
        .iter()
        .map(|edge| edge.source())
        .collect::<Vec<_>>();
    graph.reconcile_dependencies(node, desired.as_slice())?;
    graph.assert_bidirectional_consistency_at(node, &former, &desired_sources)
}

fn reconcile_random_batch(
    graph: &mut SignalGraph,
    nodes: &[NodeId],
    random: &mut MutationSequence,
) -> Result<(), SignalError> {
    let batch = [8, 9].map(|index| (nodes[index], random.dependencies(&nodes[..index])));
    let former = batch
        .iter()
        .map(|(node, _)| sources(graph, *node))
        .collect::<Result<Vec<_>, _>>()?;
    graph.reconcile_dependencies_batch(&batch)?;
    for ((node, desired), former) in batch.iter().zip(former) {
        let desired_sources = desired
            .as_slice()
            .iter()
            .map(|edge| edge.source())
            .collect::<Vec<_>>();
        graph.assert_bidirectional_consistency_at(*node, &former, &desired_sources)?;
    }
    Ok(())
}

fn repair_random_membership(
    graph: &mut SignalGraph,
    nodes: &[NodeId],
    random: &mut MutationSequence,
) -> Result<(), SignalError> {
    let source = nodes[random.next() as usize % nodes.len()];
    let candidate = nodes[random.next() as usize % nodes.len()];
    let removed = if sources(graph, candidate)?.contains(&source) {
        source
    } else {
        candidate
    };
    let mut subscribers = graph.raw_subscribers_of(source)?.to_vec();
    subscribers.push(removed);
    subscribers.sort_unstable();
    subscribers.dedup();
    graph.set_subscribers_sorted(source, &subscribers)?;
    graph.reconcile_subscriber_membership_for_sources(&[source])?;
    for node in [source, removed] {
        let desired = sources(graph, node)?;
        graph.assert_bidirectional_consistency_at(node, &[], &desired)?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Corruption {
    MissingDesiredSubscriber,
    RetainedFormerSubscriber,
    MissingSubscriberDependency,
    RetainedFormerDependency,
    MissingDesiredDependency,
    TombstonedFormerSubscriber,
}

#[test]
fn local_corruption_checks_name_the_whole_graph_oracle_edge() -> Result<(), SignalError> {
    for corruption in [
        Corruption::MissingDesiredSubscriber,
        Corruption::RetainedFormerSubscriber,
        Corruption::MissingSubscriberDependency,
        Corruption::RetainedFormerDependency,
        Corruption::MissingDesiredDependency,
        Corruption::TombstonedFormerSubscriber,
    ] {
        let (mut graph, [former, desired, node, subscriber]) = reconciled_fixture()?;
        let edge = match corruption {
            Corruption::MissingDesiredSubscriber => {
                graph.set_subscribers_sorted(desired, &[])?;
                format!("missing subscriber edge {desired} -> {node}")
            }
            Corruption::RetainedFormerSubscriber => {
                graph.set_subscribers_sorted(former, &[node])?;
                format!("missing dependency edge {former} -> {node}")
            }
            Corruption::MissingSubscriberDependency => {
                corrupt_dependency_list(&mut graph, subscriber, &[])?;
                format!("missing dependency edge {node} -> {subscriber}")
            }
            Corruption::RetainedFormerDependency => {
                corrupt_dependency_list(
                    &mut graph,
                    node,
                    &[
                        DependencyEdge::new(former, ASPECT_A),
                        DependencyEdge::new(desired, ASPECT_A),
                    ],
                )?;
                format!("missing subscriber edge {former} -> {node}")
            }
            Corruption::TombstonedFormerSubscriber => {
                graph.get_entry_mut(node)?.set_tombstoned(true);
                graph.set_subscribers_sorted(former, &[node])?;
                format!("missing dependency edge {former} -> {node}")
            }
            Corruption::MissingDesiredDependency => {
                corrupt_dependency_list(&mut graph, node, &[])?;
                format!("missing dependency edge {desired} -> {node}")
            }
        };
        let local = graph
            .assert_bidirectional_consistency_at(node, &[former], &[desired])
            .unwrap_err();
        let whole = graph.assert_bidirectional_consistency().unwrap_err();
        assert_eq!(local, whole);
        assert!(local.to_string().contains(&edge), "{local}");
    }
    Ok(())
}

#[test]
fn whole_graph_oracle_alone_catches_an_untouched_corruption() -> Result<(), SignalError> {
    let (mut graph, [former, desired, node, _]) = reconciled_fixture()?;
    let unrelated_source = graph.create_node();
    let unrelated_consumer = graph.create_node();
    graph.set_dependencies(
        unrelated_consumer,
        [DependencyEdge::new(unrelated_source, ASPECT_A)],
    )?;
    graph.set_subscribers_sorted(unrelated_source, &[])?;
    // A local mutation checks its touched edges; the explicit oracle also checks unrelated edges.
    graph.assert_bidirectional_consistency_at(node, &[former], &[desired])?;
    assert!(graph
        .assert_bidirectional_consistency()
        .unwrap_err()
        .to_string()
        .contains(&format!(
            "missing subscriber edge {unrelated_source} -> {unrelated_consumer}"
        )));
    Ok(())
}

#[test]
fn single_node_check_edge_visits_are_independent_of_unrelated_graph_size() -> Result<(), SignalError>
{
    let visits = [16, 4_096].map(reconciled_hub_visits);
    let smaller = visits[0].as_ref().map_err(Clone::clone)?;
    let larger = visits[1].as_ref().map_err(Clone::clone)?;
    assert_eq!(smaller, larger);
    // Actual dependency/subscriber probes (2), 63 former-hub probes, and replacement membership/dependency probes (2).
    assert_eq!(*smaller, 67);
    Ok(())
}

fn reconciled_hub_visits(unrelated: usize) -> Result<usize, SignalError> {
    let mut graph = SignalGraph::new();
    let hub = graph.create_node();
    let replacement = graph.create_node();
    let subscribers = (0..64).map(|_| graph.create_node()).collect::<Vec<_>>();
    for &node in &subscribers {
        graph.set_dependencies(node, [DependencyEdge::new(hub, ASPECT_A)])?;
    }
    for _ in 0..unrelated {
        graph.create_node();
    }
    let node = *subscribers.last().unwrap();
    graph.reconcile_dependencies(node, &[DependencyEdge::new(replacement, ASPECT_A)])?;
    graph.assert_bidirectional_consistency()?;
    graph.bidirectional_consistency_visits_at_for_test(node, &[hub], &[replacement])
}

fn reconciled_fixture() -> Result<(SignalGraph, [NodeId; 4]), SignalError> {
    let mut graph = SignalGraph::new();
    let nodes = std::array::from_fn(|_| graph.create_node());
    let [former, desired, node, subscriber] = nodes;
    graph.set_dependencies(node, [DependencyEdge::new(former, ASPECT_A)])?;
    graph.set_dependencies(subscriber, [DependencyEdge::new(node, ASPECT_A)])?;
    graph.reconcile_dependencies(node, &[DependencyEdge::new(desired, ASPECT_A)])?;
    Ok((graph, nodes))
}

fn corrupt_dependency_list(
    graph: &mut SignalGraph,
    node: NodeId,
    dependencies: &[DependencyEdge],
) -> Result<(), SignalError> {
    let id = graph
        .topology
        .dependency_edges
        .insert_from_slice(dependencies);
    graph.set_dependencies_id_direct(node, id)
}

fn sources(graph: &SignalGraph, node: NodeId) -> Result<Vec<NodeId>, SignalError> {
    Ok(graph
        .raw_dependencies_of(node)?
        .iter()
        .map(|edge| edge.source())
        .collect())
}

struct MutationSequence(u64);

impl MutationSequence {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn dependencies(&mut self, candidates: &[NodeId]) -> CanonicalDependencies {
        CanonicalDependencies::new(
            candidates
                .iter()
                .filter(|_| self.next().is_multiple_of(3))
                .map(|&source| DependencyEdge::new(source, ASPECT_A)),
        )
    }
}
