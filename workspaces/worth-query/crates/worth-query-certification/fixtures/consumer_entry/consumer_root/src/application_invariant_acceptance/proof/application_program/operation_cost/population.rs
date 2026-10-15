//! Real disjoint rings and discovery relations; population changes no policy or schema.
use super::*;
use worth_query_host::facade::{
    application_contribution::WorthQueryBootstrapAdvancementPhase,
    primary_graph::{
        WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
        WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
    },
};
pub(super) fn target(body: &str, index: usize) -> String {
    format!("{body}-target-{index}-a")
}
fn entity(key: &str) -> WorthQueryApplicationEntityKey<Schema, Body> {
    WorthQueryApplicationEntityKey::new(key.to_owned()).unwrap()
}
fn ring(graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>, anchor: &str) {
    for (suffix, x, y) in [("a", 1, 1), ("b", 10, 1), ("c", 1, 10)] {
        let key = if suffix == "a" {
            anchor.to_owned()
        } else {
            format!("{anchor}-{suffix}")
        };
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(Body::reference::<Schema>(), entity(&key))
                    .field(BodyKey::reference::<Schema>(), key)
                    .field(Length::reference::<Schema>(), length(1))
                    .field(PositionX::reference::<Schema>(), length(x))
                    .field(PositionY::reference::<Schema>(), length(y)),
            )
            .unwrap();
    }
    let keys = [
        anchor.to_owned(),
        format!("{anchor}-b"),
        format!("{anchor}-c"),
    ];
    for (from, to) in [(0, 1), (1, 2), (2, 0)] {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarSuccessor::reference::<Schema>(),
                format!("{}-to-{}", keys[from], keys[to]),
                entity(&keys[from]),
                entity(&keys[to]),
            ))
            .unwrap();
    }
}
pub(super) fn seed(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    fanout: usize,
    unrelated: usize,
) {
    assert!(fanout > 0);
    for (body, count) in std::iter::once(("focus-a".to_owned(), fanout))
        .chain((0..unrelated).map(|i| (format!("unrelated-{i}-a"), 1)))
    {
        ring(graph, &body);
        for i in 0..count {
            let target = target(&body, i);
            ring(graph, &target);
            graph
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    PlanarDiscoverySource::reference::<Schema>(),
                    format!("{body}-discovers-{i}"),
                    entity(&body),
                    entity(&target),
                ))
                .unwrap();
        }
    }
    graph.commit_seed_batch(phase).unwrap();
}
pub(super) fn retain_unrelated(
    runtime: &Runtime,
    request: &Request<'_>,
    count: usize,
) -> Vec<Settled> {
    (0..count)
        .map(|i| {
            let body = format!("unrelated-{i}-a");
            let owner = super::retain(runtime, request, &body, 220_000 + i as u64);
            let settled = super::settle(owner, request);
            assert_eq!(settled.settlement().root_outputs().count(), 1);
            settled
        })
        .collect()
}
