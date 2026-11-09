//! Authored membership and path are separate real relation kinds.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::world::*;

pub fn graph(bootstrap: &mut WorthQueryPrimaryGraphBootstrap<WalkSchema>, graph: Graph) {
    for key in 0..graph.nodes as u64 {
        bootstrap
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(Node::reference(), node_key(key))
                    .field(NodeKey::at("NodeKey"), key)
                    .field(Active::at("Active"), key == 0 && graph.active)
                    .field(Blocked::at("Blocked"), graph.blocked.contains(&key)),
            )
            .unwrap();
    }
    for key in 0..graph.nodes as u64 {
        bootstrap
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                NodeOwner::reference(),
                format!("owner-{key}"),
                WorthQueryApplicationEntityKey::new("principal-0").unwrap(),
                node_key(key),
            ))
            .unwrap();
        bootstrap
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                GroupRoot::reference(),
                format!("group-{key}"),
                node_key(key),
                node_key(0),
            ))
            .unwrap();
    }
    for (index, (from, to)) in graph.edges.into_iter().enumerate() {
        bootstrap
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                Next::reference(),
                format!("next-{index}"),
                node_key(from),
                node_key(to),
            ))
            .unwrap();
    }
}
