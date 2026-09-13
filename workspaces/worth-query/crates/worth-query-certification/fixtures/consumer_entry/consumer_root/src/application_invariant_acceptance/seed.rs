use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};
use worth_query_topology_entry::{Body, BodyKey, Length, PlanarSuccessor, PositionX, PositionY};

use crate::ConsumerSchema;

pub(super) fn seed_cycles(graph: &mut WorthQueryPrimaryGraphBootstrap<ConsumerSchema>) {
    for (prefix, offset) in [("anchor", 0), ("sibling", 20)] {
        for (name, x, y) in [("a", 1, 1), ("b", 10, 1), ("c", 1, 10)] {
            let key = format!("{prefix}-{name}");
            graph
                .bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Body::reference::<ConsumerSchema>(),
                        entity_key(&key),
                    )
                    .field(BodyKey::reference::<ConsumerSchema>(), key)
                    .field(Length::reference::<ConsumerSchema>(), length(1))
                    .field(PositionX::reference::<ConsumerSchema>(), length(x + offset))
                    .field(PositionY::reference::<ConsumerSchema>(), length(y + offset)),
                )
                .expect("the complete source vertex is valid");
        }
        for (from, to) in [("a", "b"), ("b", "c"), ("c", "a")] {
            graph
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    PlanarSuccessor::reference::<ConsumerSchema>(),
                    format!("{prefix}-{from}-to-{to}"),
                    entity_key(&format!("{prefix}-{from}")),
                    entity_key(&format!("{prefix}-{to}")),
                ))
                .expect("the source ring has one successor per vertex");
        }
    }
}

fn entity_key(key: &str) -> WorthQueryApplicationEntityKey<ConsumerSchema, Body> {
    WorthQueryApplicationEntityKey::new(key.to_owned()).unwrap()
}

pub(super) fn length(value: u64) -> PositiveLength {
    PositiveLength::new(value).expect("the fixture coordinate is positive")
}
