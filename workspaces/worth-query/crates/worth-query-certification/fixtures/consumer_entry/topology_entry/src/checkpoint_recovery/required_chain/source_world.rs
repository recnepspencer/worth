//! A real upstream source suffix disjoint from the three preserved outputs.

use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};

pub(super) fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>) {
    for (name, x, y) in [
        ("a", 1, 1),
        ("b", 10, 15),
        ("c", 1, 10),
        ("source-b", 10, 1),
        ("source-c", 14, 10),
    ] {
        let key = format!("anchor-{name}");
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Body::reference::<CheckpointSchema>(),
                    entity_key(&key),
                )
                .field(BodyKey::reference::<CheckpointSchema>(), key)
                .field(Length::reference::<CheckpointSchema>(), length(y + 1))
                .field(PositionX::reference::<CheckpointSchema>(), length(x))
                .field(PositionY::reference::<CheckpointSchema>(), length(y)),
            )
            .unwrap();
    }
    // A's ordinary source query reads these two successors, but its provider
    // derives input from A's own Y. B and C still consume actual A/B output
    // authority through the unchanged ChainHandler current_output calls.
    for (from, to) in [
        ("a", "source-b"),
        ("source-b", "source-c"),
        ("source-c", "b"),
        ("b", "c"),
        ("c", "a"),
    ] {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarSuccessor::reference::<CheckpointSchema>(),
                format!("anchor-{from}-to-{to}"),
                entity_key(&format!("anchor-{from}")),
                entity_key(&format!("anchor-{to}")),
            ))
            .unwrap();
    }
}

fn entity_key(key: &str) -> WorthQueryApplicationEntityKey<CheckpointSchema, Body> {
    WorthQueryApplicationEntityKey::new(key.to_owned()).unwrap()
}
