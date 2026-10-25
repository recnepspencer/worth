//! Independent committed truth obtained through a genuine retained projection.
use super::{installation::World, schema::*};
use std::collections::{BTreeMap, BTreeSet};
use worth_query_host::facade::{
    application_invariants::EntityId, runtime::ExecutionAllocationPolicy,
};

pub struct Graph {
    pub edges: BTreeSet<(u64, u64)>,
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
                            identities: BTreeMap::new(),
                        };
                        for source in 0..world.nodes {
                            let node = reader.resolve_entity(NodeKey::reference(), source).unwrap();
                            graph.identities.insert(source, node.entity_id());
                            for edge in reader.relations_from(Precedes::reference(), &node).unwrap()
                            {
                                let target = reader.field(edge.to(), NodeKey::reference()).unwrap();
                                graph.edges.insert((source, target));
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
