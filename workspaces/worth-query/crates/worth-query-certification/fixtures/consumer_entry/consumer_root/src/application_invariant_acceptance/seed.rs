use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};
use worth_query_topology_entry::{
    Body, BodyKey, Length, PlanarDiscoverySource, PlanarSuccessor, PositionX, PositionY,
    TopologySchemaBinding,
};

pub(super) fn seed_cycles<Schema: TopologySchemaBinding>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) {
    for (prefix, offset) in [("anchor", 0), ("sibling", 20), ("remote", 40)] {
        seed_cycle(graph, prefix, offset);
    }
    for target in ["b", "c"] {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarDiscoverySource::reference::<Schema>(),
                format!("anchor-a-discovers-{target}"),
                entity_key("anchor-a"),
                entity_key(&format!("sibling-{target}")),
            ))
            .expect("the authored anchor discovers its output sources");
    }
    graph
        .bind_relation(WorthQueryApplicationRelationSeed::new(
            PlanarDiscoverySource::reference::<Schema>(),
            "anchor-a-discovers-remote-b",
            entity_key("anchor-a"),
            entity_key("remote-b"),
        ))
        .expect("the authored anchor discovers an independent output source");
    graph
        .commit_seed_batch(phase)
        .expect("the complete bounded seed batch commits before installation finishes");
}

pub(super) fn seed_isolated_cycle<Schema: TopologySchemaBinding>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) {
    seed_cycle(graph, "isolated", 60);
    graph
        .commit_seed_batch(phase)
        .expect("the isolated real cycle commits during bootstrap");
}

fn seed_cycle<Schema: TopologySchemaBinding>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    prefix: &str,
    offset: u64,
) {
    for (name, x, y) in [("a", 1, 1), ("b", 10, 1), ("c", 1, 10)] {
        let key = format!("{prefix}-{name}");
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(Body::reference::<Schema>(), entity_key(&key))
                    .field(BodyKey::reference::<Schema>(), key)
                    .field(Length::reference::<Schema>(), length(1))
                    .field(PositionX::reference::<Schema>(), length(x + offset))
                    .field(PositionY::reference::<Schema>(), length(y + offset)),
            )
            .expect("the complete source vertex is valid");
    }
    for (from, to) in [("a", "b"), ("b", "c"), ("c", "a")] {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarSuccessor::reference::<Schema>(),
                format!("{prefix}-{from}-to-{to}"),
                entity_key(&format!("{prefix}-{from}")),
                entity_key(&format!("{prefix}-{to}")),
            ))
            .expect("the source ring has one successor per vertex");
    }
}

fn entity_key<Schema: TopologySchemaBinding>(
    key: &str,
) -> WorthQueryApplicationEntityKey<Schema, Body> {
    WorthQueryApplicationEntityKey::new(key.to_owned()).unwrap()
}

pub(super) fn length(value: u64) -> PositiveLength {
    PositiveLength::new(value).expect("the fixture coordinate is positive")
}
