//! Actual feeding records and optional leaf identities from retained projection.
use super::super::{installation::World, schema::*};
use std::collections::BTreeMap;
use worth_query_host::facade::{
    application_invariants::EntityId, runtime::ExecutionAllocationPolicy,
};
#[derive(Debug, Eq, PartialEq)]
pub struct FeedingGraph {
    pub sources: BTreeMap<u64, EntityId>,
    pub leaves: BTreeMap<u64, EntityId>,
    pub edges: BTreeMap<(u64, u64), usize>,
}
pub fn committed<L: LawPolicy>(world: &World<L>) -> FeedingGraph {
    let completed = world
        .application
        .runtime()
        .with_host_advancement(|phase| {
            world
                .projection
                .project(
                    &phase,
                    |reader| {
                        let mut graph = FeedingGraph {
                            sources: BTreeMap::new(),
                            leaves: BTreeMap::new(),
                            edges: BTreeMap::new(),
                        };
                        for key in 0..world.nodes {
                            let Some(node) = reader
                                .resolve_optional_entity(NodeKey::reference(), key)
                                .unwrap()
                            else {
                                continue;
                            };
                            graph.sources.insert(key, node.entity_id());
                            for edge in reader.relations_from(Feeds::reference(), &node).unwrap() {
                                let leaf = reader.field(edge.to(), LeafKey::reference()).unwrap();
                                *graph.edges.entry((key, leaf)).or_default() += 1;
                            }
                        }
                        for key in 0..world.leaves {
                            if let Some(leaf) = reader
                                .resolve_optional_entity(LeafKey::reference(), key)
                                .unwrap()
                            {
                                graph.leaves.insert(key, leaf.entity_id());
                            }
                        }
                        graph
                    },
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let (graph, snapshot, _) = completed.into_parts();
    drop(snapshot);
    graph
}
pub fn identity<L: LawPolicy>(world: &World<L>, key: u64) -> Option<EntityId> {
    let completed = world
        .application
        .runtime()
        .with_host_advancement(|phase| {
            world
                .projection
                .project(
                    &phase,
                    |reader| {
                        reader
                            .resolve_optional_entity(LeafKey::reference(), key)
                            .unwrap()
                            .map(|leaf| leaf.entity_id())
                    },
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let (identity, snapshot, _) = completed.into_parts();
    drop(snapshot);
    identity
}
