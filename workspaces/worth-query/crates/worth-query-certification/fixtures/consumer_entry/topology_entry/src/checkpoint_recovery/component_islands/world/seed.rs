//! Bootstrap authored triangle rings and actual optional coupling edges.
//!
//! Each seed batch is one canonical commit whose artifact encodes exactly the
//! rows it installs, about 31 KiB per ring (measured linear from 10 to 3,000
//! rings). One commit of 10,000 rings would exceed Relational's 256 MiB
//! canonical frame, so the population is installed in bounded batches.
use super::*;
use worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase;

/// Rings per seed batch: about 31 MiB of canonical artifact per commit.
const RINGS_PER_SEED_BATCH: usize = 1_000;

pub(super) fn rings(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>,
    count: usize,
    coupling: &[(String, String)],
) {
    let first_batch = keys(count.min(RINGS_PER_SEED_BATCH));
    assert!(
        coupling
            .iter()
            .all(|(from, to)| first_batch.contains(from) && first_batch.contains(to)),
        "coupling edges join rings seeded in the first batch"
    );
    for batch in (0..count).step_by(RINGS_PER_SEED_BATCH) {
        for ring in batch..count.min(batch + RINGS_PER_SEED_BATCH) {
            ring_rows(graph, ring);
        }
        if batch == 0 {
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
        graph.commit_seed_batch(phase).unwrap();
    }
}

fn ring_rows(graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>, ring: usize) {
    for (vertex, x, y) in [("a", 1, 1), ("b", 10, 1), ("c", 1, 10)] {
        let key = key(ring, vertex);
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(Body::reference(), entity_key(&key))
                    .field(BodyKey::at("BodyKey"), key)
                    .field(Length::at("Length"), length(y + 1))
                    .field(PositionX::at("PositionX"), length(x + ring as u64 * 20))
                    .field(PositionY::at("PositionY"), length(y)),
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

fn entity_key(key: &str) -> WorthQueryApplicationEntityKey<CheckpointSchema, Body> {
    WorthQueryApplicationEntityKey::new(key).unwrap()
}
