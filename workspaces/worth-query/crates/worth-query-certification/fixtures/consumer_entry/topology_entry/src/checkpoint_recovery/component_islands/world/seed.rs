//! Bootstrap authored triangle rings and actual optional coupling edges.
use super::*;
pub(super) fn rings(
    graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>,
    count: usize,
    coupling: &[(String, String)],
) {
    for ring in 0..count {
        for (vertex, x, y) in [("a", 1, 1), ("b", 10, 1), ("c", 1, 10)] {
            let key = key(ring, vertex);
            graph
                .bind_entity(
                    WorthQueryApplicationEntitySeed::new(Body::reference(), entity_key(&key))
                        .field(BodyKey::reference(), key)
                        .field(Length::reference(), length(y + 1))
                        .field(PositionX::reference(), length(x + ring as u64 * 20))
                        .field(PositionY::reference(), length(y)),
                )
                .unwrap();
        }
        for (from, to) in [("a", "b"), ("b", "c"), ("c", "a")] {
            graph
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    PlanarSuccessor::reference(),
                    format!("successor:{ring}:{from}:{to}"),
                    entity_key(&key(ring, from)),
                    entity_key(&key(ring, to)),
                ))
                .unwrap();
        }
    }
    for (from, to) in coupling {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarCoupling::reference(),
                format!("island:{from}:{to}"),
                entity_key(from),
                entity_key(to),
            ))
            .unwrap();
    }
}
fn entity_key(key: &str) -> WorthQueryApplicationEntityKey<CheckpointSchema, Body> {
    WorthQueryApplicationEntityKey::new(key).unwrap()
}
