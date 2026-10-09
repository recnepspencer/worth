//! The two rings of the equal-output fixture.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};
pub(super) fn seed_cycle(graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>) {
    for (name, x, y) in [
        ("a", 1, 1),
        ("b", 10, 1),
        ("c", 1, 10),
        ("isolated", 50, 50),
        ("island", 60, 50),
        ("atoll", 50, 60),
    ] {
        let key = format!("anchor-{name}");
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(Body::reference::<Schema>(), entity_key(&key))
                    .field(BodyKey::reference::<Schema>(), key)
                    // The initial performed publication writes an equal derived
                    // value; native revisions still decide whether it can reuse.
                    .field(Length::reference::<Schema>(), length(y + 1))
                    .field(PositionX::reference::<Schema>(), length(x))
                    .field(PositionY::reference::<Schema>(), length(y)),
            )
            .unwrap();
    }
    for (from, to) in [
        ("a", "b"),
        ("b", "c"),
        ("c", "island"),
        ("isolated", "a"),
        ("island", "atoll"),
        ("atoll", "isolated"),
    ] {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarSuccessor::reference::<Schema>(),
                format!("anchor-{from}-to-{to}"),
                entity_key(&format!("anchor-{from}")),
                entity_key(&format!("anchor-{to}")),
            ))
            .unwrap();
    }
}

fn entity_key(key: &str) -> WorthQueryApplicationEntityKey<Schema, Body> {
    WorthQueryApplicationEntityKey::new(key.to_owned()).unwrap()
}
