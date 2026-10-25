//! Independent committed truth obtained through a genuine retained projection.
use super::{installation::World, schema::*};
use std::collections::{BTreeMap, BTreeSet};
use worth_query_host::facade::{
    application_invariants::EntityId, runtime::ExecutionAllocationPolicy,
};

pub struct Graph {
    pub edges: BTreeSet<(u64, u64)>,
    pub multiplicity: BTreeMap<(u64, u64), usize>,
    pub identities: BTreeMap<u64, EntityId>,
}
pub fn committed<L: LawPolicy>(world: &World<L>) -> Graph {
    let completed = world
        .application
        .runtime()
        .with_host_advancement(|phase| {
            world
                .projection
                .project(
                    &phase,
                    |reader| {
                        let mut graph = Graph {
                            edges: BTreeSet::new(),
                            multiplicity: BTreeMap::new(),
                            identities: BTreeMap::new(),
                        };
                        for source in 0..world.nodes {
                            let Some(node) = reader
                                .resolve_optional_entity(NodeKey::reference(), source)
                                .unwrap()
                            else {
                                continue;
                            };
                            graph.identities.insert(source, node.entity_id());
                            for edge in reader.relations_from(Precedes::reference(), &node).unwrap()
                            {
                                let target = reader.field(edge.to(), NodeKey::reference()).unwrap();
                                graph.edges.insert((source, target));
                                *graph.multiplicity.entry((source, target)).or_default() += 1;
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

/// Resolve the requested key directly, including keys outside the expected node set.
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
                            .resolve_optional_entity(NodeKey::reference(), key)
                            .unwrap()
                            .map(|entity| entity.entity_id())
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

#[path = "read/feeds.rs"]
pub mod feeds;
