//! Independent copies of one neutral model. Each ring is the five-body cycle
//! a → source-b → source-c → b → c → a, and shares nothing with another ring.
//!
//! The root output of a body publishes its Y plus one. `b` decides over the
//! root output of `a` and `c` over the output of `b`; each republishes its own
//! Length.

use super::*;
use worth_query_consumer_values::PlanarVertex;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};

/// Rings sit this far apart on X, so no two bodies coincide.
pub(super) const RING_SPACING: u64 = 20;

/// The body key of `role` in `ring`.
pub(super) fn key(ring: usize, role: &str) -> String {
    format!("ring-{ring}-{role}")
}

/// The ring and role a body key names.
pub(super) fn ring_and_role(body_key: &str) -> Option<(usize, &str)> {
    let (ring, role) = body_key.strip_prefix("ring-")?.split_once('-')?;
    Some((ring.parse().ok()?, role))
}

/// The one output a ring's chain node consumes; other bodies consume none.
pub(super) fn upstream_of(body_key: &str) -> Option<ChainUpstream> {
    let (ring, role) = ring_and_role(body_key)?;
    match role {
        "b" => Some(ChainUpstream {
            key: key(ring, "a"),
            root: true,
        }),
        "c" => Some(ChainUpstream {
            key: key(ring, "b"),
            root: false,
        }),
        _ => None,
    }
}

/// One ring's bodies in cycle order: role, X before the ring's offset, and Y.
const BODIES: [(&str, u64, u64); 5] = [
    ("a", 1, 1),
    ("source-b", 10, 1),
    ("source-c", 14, 10),
    ("b", 10, 15),
    ("c", 1, 10),
];

/// The vertices of ring `ring`, for a commit that creates it.
pub(super) fn vertices(ring: usize) -> Vec<PlanarVertex> {
    BODIES
        .into_iter()
        .map(|(role, x, y)| PlanarVertex {
            body_key: key(ring, role),
            x: length(x + RING_SPACING * ring as u64),
            y: length(y),
        })
        .collect()
}

/// Seeds `RINGS` independent rings.
pub(super) fn seed<const RINGS: usize>(
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
                    .field(BodyKey::reference::<CheckpointSchema>(), body_key)
                    .field(Length::reference::<CheckpointSchema>(), length(y + 1))
                    .field(
                        PositionX::reference::<CheckpointSchema>(),
                        length(x + offset),
                    )
                    .field(PositionY::reference::<CheckpointSchema>(), length(y)),
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
