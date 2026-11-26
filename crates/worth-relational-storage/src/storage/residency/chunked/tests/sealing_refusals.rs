//! The single sealing owner derives summaries before hashing and refuses an
//! omitted kind or edge (`M3` at the writer), out-of-chunk or overlapping
//! slots, foreign ranges and oversize versions.
use super::super::canonical_read::ChunkResidency;
use super::super::chunk_summary::ClaimedChunkSummaries;
use super::super::consolidation::verify_bound_chunk;
use super::super::layout::ChunkLayout;
use super::super::sealing::{seal_chunk_image, ChunkSealRefusal, ChunkSealRequest};
use super::super::slot_projection::{AdjacencyEdge, SlotVersion};
use super::super::slot_range::{SlotDomain, SlotRange};
use super::member_store::{accountant, edge, seal, version, MemberStore, SlotFixture, LAYOUT};
use crate::identity::data::{KindId, PartitionId, RelationId};
use crate::storage::residency::working_set::ResidencyPoolKind;

fn slots() -> Vec<SlotFixture> {
    vec![
        SlotFixture {
            slot: 2,
            head: Some(version(1, 2)),
            // Kind 9 appears only in this slot's spilled history prefix.
            history: (0..200)
                .map(|v| version(if v == 0 { 9 } else { 4 }, v))
                .collect(),
            adjacency: vec![edge(30, 1), edge(31, 2)],
        },
        SlotFixture {
            slot: 5,
            head: Some(version(2, 5)),
            history: Vec::new(),
            adjacency: vec![edge(31, 3)],
        },
    ]
}

fn request<'a>(
    store_slots: &'a [SlotFixture],
    range: SlotRange,
    claimed: Option<ClaimedChunkSummaries>,
) -> ChunkSealRequest<'a> {
    ChunkSealRequest {
        layout: LAYOUT,
        domain: SlotDomain::Entity,
        domain_slots: 100,
        slot_range: range,
        slots: Box::new(store_slots.iter().map(SlotFixture::projection)),
        claimed,
    }
}

#[test]
fn summaries_are_derived_from_every_record_and_an_omitted_edge_is_refused() {
    let store = MemberStore::default();
    let range = SlotRange::new(0, 64).unwrap();
    let fixtures = slots();
    let sealed = seal(&store, SlotDomain::Entity, 100, range, &fixtures);
    assert!(sealed.continuation_pages > 0, "slot 2's history spilled");
    let kinds = |values: &[u32]| values.iter().copied().map(KindId::new).collect::<Vec<_>>();
    assert_eq!(
        sealed.semantic.head_summary().record_kinds(),
        kinds(&[1, 2])
    );
    assert_eq!(
        sealed.semantic.head_summary().adjacency_kinds(),
        kinds(&[30, 31])
    );
    assert_eq!(
        sealed.semantic.history_summary().record_kinds(),
        kinds(&[4, 9])
    );

    let honest =
        ClaimedChunkSummaries::from_kinds(kinds(&[1, 2]), kinds(&[30, 31]), kinds(&[4, 9]));
    let claimed = seal_chunk_image(request(&fixtures, range, Some(honest)), &mut &store).unwrap();
    assert_eq!(claimed, sealed, "a correct claim seals the identical entry");

    for omitted in [
        ClaimedChunkSummaries::from_kinds(kinds(&[1, 2]), kinds(&[31]), kinds(&[4, 9])),
        ClaimedChunkSummaries::from_kinds(kinds(&[1, 2]), kinds(&[30, 31]), kinds(&[4])),
    ] {
        assert!(matches!(
            seal_chunk_image(request(&fixtures, range, Some(omitted)), &mut &store),
            Err(ChunkSealRefusal::UnverifiedSummary { .. })
        ));
    }
}

#[test]
fn sealing_refuses_foreign_ranges_out_of_chunk_or_overlapping_slots_and_oversize_versions() {
    let store = MemberStore::default();
    let range = SlotRange::new(0, 64).unwrap();
    let refusal = |fixtures: Vec<SlotFixture>, range: SlotRange| {
        seal_chunk_image(request(&fixtures, range, None), &mut &store).unwrap_err()
    };
    let at = |slot: u64, payload: usize| SlotFixture {
        slot,
        head: Some(super::super::slot_projection::SlotVersion {
            kind: KindId::new(1),
            payload: vec![7; payload],
        }),
        history: Vec::new(),
        adjacency: Vec::new(),
    };
    assert_eq!(
        refusal(vec![at(1, 8)], SlotRange::new(10, 74).unwrap()),
        ChunkSealRefusal::NotFixedChunk {
            slot_range: SlotRange::new(10, 74).unwrap(),
            domain_slots: 100
        }
    );
    assert_eq!(
        refusal(vec![at(64, 8)], range),
        ChunkSealRefusal::SlotOutsideChunk { slot: 64 }
    );
    assert_eq!(
        refusal(vec![at(3, 8), at(3, 8)], range),
        ChunkSealRefusal::SlotOutOfOrder { slot: 3 }
    );
    assert_eq!(
        refusal(vec![at(4, 8), at(2, 8)], range),
        ChunkSealRefusal::SlotOutOfOrder { slot: 2 }
    );
    assert_eq!(
        refusal(vec![at(1, LAYOUT.slot_inline_budget + 1)], range),
        ChunkSealRefusal::OversizeVersion {
            slot: 1,
            bytes: LAYOUT.slot_inline_budget + 1,
            limit: LAYOUT.slot_inline_budget
        }
    );
}

#[test]
fn reg8_seals_high_byte_payloads_large_edge_ids_and_multi_level_continuation_directories() {
    let store = MemberStore::default();
    let range = SlotRange::new(0, ChunkLayout::REG8.chunk_slots).unwrap();
    let fixtures = vec![
        SlotFixture {
            slot: 1,
            head: Some(SlotVersion {
                kind: KindId::new(u32::MAX),
                payload: vec![0xff; 1000],
            }),
            // 300 * 1000B of 0xff produces >= 75 continuation pages (> 64 directory entries).
            history: (0..300)
                .map(|index| {
                    let mut payload = vec![0xff; 1000];
                    payload[..4].copy_from_slice(&(index as u32).to_be_bytes());
                    SlotVersion {
                        kind: KindId::new(u32::MAX - (index % 3) as u32),
                        payload,
                    }
                })
                .collect(),
            adjacency: Vec::new(),
        },
        SlotFixture {
            slot: 2,
            head: Some(SlotVersion {
                kind: KindId::new(1),
                payload: vec![0xfe; 16],
            }),
            history: Vec::new(),
            adjacency: (0..300_u64)
                .map(|index| AdjacencyEdge {
                    relation_kind: KindId::new(u32::MAX),
                    relation: RelationId::new(
                        PartitionId::new(u32::MAX),
                        u64::MAX - index,
                        u32::MAX,
                    ),
                })
                .collect(),
        },
    ];
    let sealed = seal_chunk_image(
        ChunkSealRequest {
            layout: ChunkLayout::REG8,
            domain: SlotDomain::Entity,
            domain_slots: ChunkLayout::REG8.chunk_slots,
            slot_range: range,
            slots: Box::new(fixtures.iter().map(SlotFixture::projection)),
            claimed: None,
        },
        &mut &store,
    )
    .expect("REG8 must seal high-byte payloads, max-width edges, and >=64 continuation pages");
    assert!(
        sealed.continuation_pages >= 75,
        "expected >= 75 continuation pages (multi-level REG8 directory), got {}",
        sealed.continuation_pages
    );
    let residency = ChunkResidency::new(
        &accountant(1 << 20, 1 << 20, 4096),
        ResidencyPoolKind::Historical,
    );
    verify_bound_chunk(&residency, &store, &sealed.semantic, &sealed.binding)
        .expect("cold verification agrees with REG8 sealed commitment");
}
