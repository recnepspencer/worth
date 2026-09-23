use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};
use worth_query_topology_entry::{
    Body, BodyKey, Length, PlanarDiscoverySource, PlanarSuccessor, PositionX, PositionY,
};

use crate::ConsumerSchema;

pub(super) fn seed_cycles(graph: &mut WorthQueryPrimaryGraphBootstrap<ConsumerSchema>) {
    for (prefix, offset) in [("anchor", 0), ("sibling", 20), ("remote", 40)] {
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
    for target in ["b", "c"] {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarDiscoverySource::reference::<ConsumerSchema>(),
                format!("anchor-a-discovers-{target}"),
                entity_key("anchor-a"),
                entity_key(&format!("sibling-{target}")),
            ))
            .expect("the authored anchor discovers its output sources");
    }
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            PlanarDiscoverySource::reference::<ConsumerSchema>(),
            "anchor-a-discovers-remote-b",
            entity_key("anchor-a"),
            entity_key("remote-b"),
        ))
        .expect("the authored anchor discovers an independent output source");
    graph
        .commit_seed_batch()
        .expect("the complete bounded seed batch commits before installation finishes");
}

fn entity_key(key: &str) -> WorthQueryApplicationEntityKey<ConsumerSchema, Body> {
    WorthQueryApplicationEntityKey::new(key.to_owned()).unwrap()
}

pub(super) fn length(value: u64) -> PositiveLength {
    PositiveLength::new(value).expect("the fixture coordinate is positive")
}
