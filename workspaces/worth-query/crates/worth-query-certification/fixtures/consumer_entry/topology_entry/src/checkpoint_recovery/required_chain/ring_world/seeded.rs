//! The seeded rings.

use super::{super::*, key};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};

/// Rings sit this far apart on X, so no two bodies coincide.
pub(in super::super) const RING_SPACING: u64 = 20;

/// One ring's bodies in cycle order: role, X before the ring's offset, and Y.
const BODIES: [(&str, u64, u64); 5] = [
    ("a", 1, 1),
    ("source-b", 10, 1),
    ("source-c", 14, 10),
    ("b", 10, 15),
    ("c", 1, 10),
];

/// Seeds `RINGS` independent rings.
pub(in super::super) fn seed<const RINGS: usize>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>,
) {
    for ring in 0..RINGS {
        let offset = RING_SPACING * ring as u64;
        for (role, x, y) in BODIES {
            let body_key = key(ring, role);
            graph
                .bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Body::reference::<CheckpointSchema>(),
                        entity_key(&body_key),
                    )
                    .field(BodyKey::at("BodyKey"), body_key)
                    .field(Length::at("Length"), length(y + 1))
                    .field(PositionX::at("PositionX"), length(x + offset))
                    .field(PositionY::at("PositionY"), length(y)),
                )
                .unwrap();
        }
        for (position, (from, ..)) in BODIES.into_iter().enumerate() {
            let (to, ..) = BODIES[(position + 1) % BODIES.len()];
            graph
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    PlanarSuccessor::reference::<CheckpointSchema>(),
                    format!("ring-{ring}-{from}-to-{to}"),
                    entity_key(&key(ring, from)),
                    entity_key(&key(ring, to)),
                ))
                .unwrap();
        }
    }
}

fn entity_key(key: &str) -> WorthQueryApplicationEntityKey<CheckpointSchema, Body> {
    WorthQueryApplicationEntityKey::new(key.to_owned()).unwrap()
}
