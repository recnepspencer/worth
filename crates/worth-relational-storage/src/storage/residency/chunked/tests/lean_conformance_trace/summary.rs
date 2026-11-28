//! Summary derivation and pruned-vs-canonical scan conformance trace builders
//! for `StoreM4.ChunkedCoverage`.
use std::collections::BTreeSet;

use super::super::super::canonical_read::ChunkResidency;
use super::super::super::chunk_directory::RegionChunkDirectory;
use super::super::super::chunk_summary::ClaimedChunkSummaries;
use super::super::super::locator_binding::ChunkLocatorBinding;
use super::super::super::sealing::{seal_chunk_image, ChunkSealRequest};
use super::super::super::semantic_entry::ChunkSemanticEntry;
use super::super::super::slot_range::{fixed_partition, SlotDomain, SlotRange};
use super::super::super::summary_scan::{scan_chunks, ChunkScan, ScanPruning};
use super::super::member_store::{
    accountant, edge, seal, version, MemberStore, SlotFixture, LAYOUT,
};
use super::{rows_json, u64_list_json, Trace};
use crate::identity::data::KindId;
use crate::storage::residency::working_set::ResidencyPoolKind;

fn scan_fixtures(chunk: u64) -> Vec<SlotFixture> {
    let base = chunk * LAYOUT.chunk_slots;
    (0..3)
        .map(|i| {
            let slot = base + i * 10;
            let head_kind = if chunk == 1 && i == 1 { 7 } else { 1 };
            let rare_hist = chunk == 2 && i == 0;
            SlotFixture {
                slot,
                head: Some(version(head_kind, slot)),
                history: (0..if rare_hist { 120 } else { 2 })
                    .map(|v| version(if rare_hist && v == 0 { 9 } else { 4 }, v))
                    .collect(),
                adjacency: vec![edge(if chunk == 2 && i == 2 { 11 } else { 10 }, slot)],
            }
        })
        .collect()
}

pub(super) fn record_scan_step(
    trace: &mut Trace,
    directory: &RegionChunkDirectory,
    residency: &ChunkResidency,
    store: &MemberStore,
    chunk_fixtures: &[Vec<SlotFixture>],
    selection: ChunkScan,
) {
    let can = scan_chunks(
        directory,
        residency,
        store,
        SlotDomain::Entity,
        selection,
        ScanPruning::Canonical,
    )
    .unwrap();
    let pru = scan_chunks(
        directory,
        residency,
        store,
        SlotDomain::Entity,
        selection,
        ScanPruning::Summaries,
    )
    .unwrap();
    let (qkey, can_hits, pru_hits): (u64, Vec<u64>, Vec<u64>) = match selection {
        ChunkScan::HeadKind(k) | ChunkScan::HistoryKind(k) => (
            u64::from(k.as_u32()),
            can.hits.iter().map(|h| h.slot).collect(),
            pru.hits.iter().map(|h| h.slot).collect(),
        ),
        ChunkScan::Adjacency(k) => (
            u64::from(k.as_u32()),
            can.hits
                .iter()
                .map(|h| h.slot * 1_000_000 + h.relation.unwrap().local_slot_value())
                .collect(),
            pru.hits
                .iter()
                .map(|h| h.slot * 1_000_000 + h.relation.unwrap().local_slot_value())
                .collect(),
        ),
    };
    let chunks_json: Vec<String> = directory
        .domain_chunks(SlotDomain::Entity)
        .zip(chunk_fixtures)
        .map(|(ch, fx)| {
            let recs: Vec<(u64, Vec<u64>)> = match selection {
                ChunkScan::HeadKind(_) => fx
                    .iter()
                    .filter_map(|f| {
                        f.head
                            .as_ref()
                            .map(|h| (f.slot, vec![u64::from(h.kind.as_u32())]))
                    })
                    .collect(),
                ChunkScan::HistoryKind(_) => fx
                    .iter()
                    .map(|f| {
                        let ks: BTreeSet<u64> = f
                            .history
                            .iter()
                            .map(|v| u64::from(v.kind.as_u32()))
                            .collect();
                        (f.slot, ks.into_iter().collect())
                    })
                    .collect(),
                ChunkScan::Adjacency(_) => fx
                    .iter()
                    .flat_map(|f| {
                        f.adjacency.iter().map(move |e| {
                            (
                                f.slot * 1_000_000 + e.relation.local_slot_value(),
                                vec![u64::from(e.relation_kind.as_u32())],
                            )
                        })
                    })
                    .collect(),
            };
            let summary: Vec<u64> = match selection {
                ChunkScan::HeadKind(_) => ch.semantic().head_summary().record_kinds(),
                ChunkScan::HistoryKind(_) => ch.semantic().history_summary().record_kinds(),
                ChunkScan::Adjacency(_) => ch.semantic().head_summary().adjacency_kinds(),
            }
            .iter()
            .map(|k| u64::from(k.as_u32()))
            .collect();
            format!(
                r#"{{"records":{},"summary":{}}}"#,
                rows_json(&recs),
                u64_list_json(&summary)
            )
        })
        .collect();
    trace.push(
        "relational_summary_scan",
        format!(
            r#""query_key":{qkey},"chunks":[{}],"canonical_hits":{},"pruned_hits":{},"canonical_chunks_read":{},"pruned_chunks_read":{}"#,
            chunks_json.join(","),
            u64_list_json(&can_hits),
            u64_list_json(&pru_hits),
            can.chunks_read,
            pru.chunks_read,
        ),
    );
}

pub(super) fn record_summary_cases(
    trace: &mut Trace,
) -> (
    MemberStore,
    Vec<Vec<SlotFixture>>,
    Vec<(ChunkSemanticEntry, ChunkLocatorBinding)>,
) {
    let store = MemberStore::default();
    let slots = 192;
    let fixtures: Vec<Vec<SlotFixture>> = (0..3).map(scan_fixtures).collect();
    let sealed: Vec<(ChunkSemanticEntry, ChunkLocatorBinding)> =
        fixed_partition(slots, LAYOUT.chunk_slots)
            .enumerate()
            .map(|(i, range)| {
                let s = seal(&store, SlotDomain::Entity, slots, range, &fixtures[i]);
                (s.semantic, s.binding)
            })
            .collect();
    let dir = RegionChunkDirectory::admit(LAYOUT, slots, 0, sealed.clone()).unwrap();
    let res = ChunkResidency::new(&accountant(1 << 20, 1 << 20, 4096), ResidencyPoolKind::Head);
    for sel in [
        ChunkScan::HeadKind(KindId::new(7)),
        ChunkScan::HistoryKind(KindId::new(9)),
        ChunkScan::Adjacency(KindId::new(11)),
        ChunkScan::HeadKind(KindId::new(99)),
        ChunkScan::HeadKind(KindId::new(1)),
    ] {
        record_scan_step(trace, &dir, &res, &store, &fixtures, sel);
    }

    let claim_range = SlotRange::new(64, 128).unwrap();
    let claim_recs: Vec<(u64, Vec<u64>)> = fixtures[1]
        .iter()
        .filter_map(|f| {
            f.head
                .as_ref()
                .map(|h| (f.slot, vec![u64::from(h.kind.as_u32())]))
        })
        .collect();
    for (claimed_head, expected_ok) in [(vec![1, 7], true), (vec![1], false)] {
        let claimed = ClaimedChunkSummaries::from_kinds(
            claimed_head.iter().copied().map(KindId::new),
            [KindId::new(10)],
            [KindId::new(4)],
        );
        let admitted = seal_chunk_image(
            ChunkSealRequest {
                layout: LAYOUT,
                domain: SlotDomain::Entity,
                domain_slots: slots,
                slot_range: claim_range,
                slots: Box::new(fixtures[1].iter().map(SlotFixture::projection)),
                claimed: Some(claimed),
            },
            &mut &store,
        )
        .is_ok();
        assert_eq!(admitted, expected_ok);
        let claimed_u64: Vec<u64> = claimed_head.into_iter().map(u64::from).collect();
        trace.push(
            "relational_summary_claim",
            format!(
                r#""records":{},"claimed_summary":{},"admitted":{admitted}"#,
                rows_json(&claim_recs),
                u64_list_json(&claimed_u64),
            ),
        );
    }
    (store, fixtures, sealed)
}
