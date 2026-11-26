//! Summary-pruned kind scans and adjacency traversals over an admitted
//! directory return exactly the canonical full scan while reading only the
//! chunks whose producer-derived summary may contain the selection.
use super::super::canonical_read::ChunkResidency;
use super::super::chunk_directory::RegionChunkDirectory;
use super::super::slot_range::{fixed_partition, SlotDomain};
use super::super::summary_scan::{scan_chunks, ChunkScan, ScanPruning};
use super::member_store::{accountant, edge, seal, version, MemberStore, SlotFixture, LAYOUT};
use crate::identity::data::KindId;
use crate::storage::residency::working_set::ResidencyPoolKind;

const SLOTS: u64 = 640;

/// Kind 7 heads live only in chunk 3, history kind 9 only in a spilled
/// history of chunk 5, adjacency kind 11 only on a vertex of chunk 8.
fn chunk_slots(chunk: u64) -> Vec<SlotFixture> {
    let base = chunk * LAYOUT.chunk_slots;
    (0..4)
        .map(|offset| {
            let slot = base + offset * 13;
            let head_kind = if chunk == 3 && offset == 2 { 7 } else { 1 };
            let rare_history = chunk == 5 && offset == 1;
            SlotFixture {
                slot,
                head: Some(version(head_kind, slot)),
                history: (0..if rare_history { 300 } else { 2 })
                    .map(|v| version(if rare_history && v == 0 { 9 } else { 1 }, v))
                    .collect(),
                adjacency: (0..3)
                    .map(|e| {
                        edge(
                            if chunk == 8 && offset == 3 && e == 1 {
                                11
                            } else {
                                10
                            },
                            slot + e,
                        )
                    })
                    .collect(),
            }
        })
        .collect()
}

#[test]
fn pruned_scans_agree_with_canonical_full_scans_and_read_only_candidate_chunks() {
    let store = MemberStore::default();
    let sealed = fixed_partition(SLOTS, LAYOUT.chunk_slots)
        .enumerate()
        .map(|(chunk, range)| {
            let sealed = seal(
                &store,
                SlotDomain::Entity,
                SLOTS,
                range,
                &chunk_slots(chunk as u64),
            );
            (sealed.semantic, sealed.binding)
        })
        .collect::<Vec<_>>();
    let directory = RegionChunkDirectory::admit(LAYOUT, SLOTS, 0, sealed).unwrap();
    let residency =
        ChunkResidency::new(&accountant(1 << 20, 1 << 20, 4096), ResidencyPoolKind::Head);
    let scan = |selection, pruning| {
        scan_chunks(
            &directory,
            &residency,
            &store,
            SlotDomain::Entity,
            selection,
            pruning,
        )
        .unwrap()
    };
    for selection in [
        ChunkScan::HeadKind(KindId::new(7)),
        ChunkScan::HistoryKind(KindId::new(9)),
        ChunkScan::Adjacency(KindId::new(11)),
        ChunkScan::HeadKind(KindId::new(99)),
    ] {
        let canonical = scan(selection, ScanPruning::Canonical);
        let pruned = scan(selection, ScanPruning::Summaries);
        assert_eq!(pruned.hits, canonical.hits, "{selection:?}");
        assert_eq!(canonical.chunks_read, 10);
        let expected_reads = usize::from(selection != ChunkScan::HeadKind(KindId::new(99)));
        assert_eq!(pruned.chunks_read, expected_reads, "{selection:?}");
        assert_eq!(pruned.hits.len(), expected_reads, "{selection:?}");
    }
    let common = scan(ChunkScan::HeadKind(KindId::new(1)), ScanPruning::Summaries);
    assert_eq!(common.chunks_read, 10);
    assert_eq!(
        common,
        scan(ChunkScan::HeadKind(KindId::new(1)), ScanPruning::Canonical)
    );
}

#[test]
fn bounded_multi_chunk_scan_evicts_across_chunk_residency_caches() {
    let store = MemberStore::default();
    let sealed = fixed_partition(SLOTS, LAYOUT.chunk_slots)
        .enumerate()
        .map(|(chunk, range)| {
            let base = chunk as u64 * LAYOUT.chunk_slots;
            let slots = vec![SlotFixture {
                slot: base + 1,
                head: Some(version(1, base + 1)),
                // Every chunk spills continuation pages so all four caches
                // compete for the same tight pool budget.
                history: (0..250)
                    .map(|v| version(if chunk == 9 && v == 0 { 9 } else { 1 }, v))
                    .collect(),
                adjacency: vec![edge(10, base + 1)],
            }];
            let sealed = seal(&store, SlotDomain::Entity, SLOTS, range, &slots);
            (sealed.semantic, sealed.binding)
        })
        .collect::<Vec<_>>();
    let directory = RegionChunkDirectory::admit(LAYOUT, SLOTS, 0, sealed).unwrap();

    // Tight enough that unborrowed continuation pages from earlier chunks MUST
    // be evicted for later chunks' head/history/directory pages to fit.
    let tight_budget = 16 * 1024;
    let tight_accountant = accountant(tight_budget, 1 << 20, 4096);
    let tight_residency = ChunkResidency::new(&tight_accountant, ResidencyPoolKind::Head);
    let outcome = scan_chunks(
        &directory,
        &tight_residency,
        &store,
        SlotDomain::Entity,
        ChunkScan::HistoryKind(KindId::new(9)),
        ScanPruning::Canonical,
    )
    .expect("cross-cache eviction prevents starvation across chunks");
    assert_eq!(outcome.chunks_read, 10);
    assert_eq!(outcome.hits.len(), 1);
    assert!(tight_accountant.charged_bytes(ResidencyPoolKind::Head) <= tight_budget);
    assert!(tight_residency.continuation_observation().evictions > 0);
}
