//! Live region placement courts (Store M4.3c: `S2`, `M3`): a captured
//! partition is sealed into bounded `REG8` chunks, admitted, scanned cold
//! under a tight working set with summary pruning, consolidated without
//! moving its semantic root, and every placement defect is refused.
use std::collections::BTreeSet;
use std::sync::Mutex;

use super::super::canonical_read::ChunkResidency;
use super::super::chunk_directory::ChunkCoverageRefusal;
use super::super::chunk_images::{ChunkHeadImage, ChunkHistoryImage};
use super::super::consolidation::{
    consolidate_history_chunk, verify_bound_chunk, ChunkVerificationRefusal,
};
use super::super::member_io::ChunkMemberSource;
use super::super::page_cache::ResidentPage;
use super::super::region_placement::{
    ChunkPlacementRefusal, RegionChunkPlacement, REGION_CHUNK_LAYOUT,
};
use super::super::region_projection::{seal_region_chunks, SealedRegionChunks};
use super::super::semantic_entry::{root_semantic_merkle, ChunkSemanticEntry};
use super::super::slot_range::SlotDomain;
use super::super::summary_scan::{scan_chunks, ChunkScan, ScanHit, ScanPruning};
use super::member_store::{accountant, MemberStore};
use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::durability::medium::RelationalDurableMemberLocator as Locator;
use crate::identity::data::{KindId, PartitionId, RelationId, VersionId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::adjacency::AdjacencySetFixturePort as _;
use crate::storage::residency::working_set::ResidencyPoolKind;
use crate::storage::substrate::record_arena::arena::slot_lifecycle::metadata::RecordArenaFixturePort as _;
use crate::storage::substrate::{EntityArena, EntityExtra, RelationArena, SlotInit};

const COMMON: u32 = 1;
const RARE: u32 = 7;
const LINK: u32 = 40;
const RARE_SLOTS: std::ops::Range<u64> = 8192..8200;

fn partition(entities: u64, hub_edges: u64) -> PartitionState {
    let id = PartitionId(1);
    let policy = AdjacencyPolicy {
        backend: AdjacencyBackend::CompressedFanoutAdjacency,
        small_degree_inline_capacity: 4,
    };
    let mut partition = PartitionState {
        partition_id: id,
        adjacency_policy: policy.clone(),
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(entities as usize),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    };
    for slot in 0..entities {
        let kind = if RARE_SLOTS.contains(&slot) {
            RARE
        } else {
            COMMON
        };
        let init = SlotInit {
            partition_id: id,
            kind_id: KindId::new(kind),
            version_id: VersionId(1 + slot),
            extra: EntityExtra::default(),
        };
        partition
            .entity_arena
            .write_reserved_slot_for_test(init, slot as usize, 1)
            .unwrap();
    }
    let hub = partition.adjacency.ensure(0, &policy);
    for relation in 0..hub_edges {
        let id = RelationId::new(id, relation, 1);
        hub.insert_adjacent_id_at_for_test(KindId::new(LINK), id, VersionId(1));
    }
    partition
}

fn sealed(partition: &PartitionState) -> (SealedRegionChunks, MemberStore) {
    let sealed = seal_region_chunks(partition, &mut |_, _| Ok::<(), ()>(())).unwrap();
    let store = MemberStore::default();
    for (locator, bytes) in &sealed.members {
        assert!(bytes.len() <= REGION_CHUNK_LAYOUT.max_chunk_image_bytes);
        store.replace(*locator, bytes.clone());
    }
    (sealed, store)
}

/// Records which members a read faulted.
struct Counting<'s> {
    store: &'s MemberStore,
    read: Mutex<BTreeSet<Locator>>,
}

impl ChunkMemberSource for Counting<'_> {
    fn read_range(&self, member: Locator, offset: u64, target: &mut [u8]) -> Result<usize, Cause> {
        self.read.lock().unwrap().insert(member);
        self.store.read_range(member, offset, target)
    }
}

fn decoded<T: ResidentPage>(store: &MemberStore, locator: Locator) -> u64 {
    T::decode(&store.bytes(locator), locator)
        .unwrap()
        .resident_bytes()
}

#[test]
fn a_partition_larger_than_the_working_set_scans_cold_chunk_by_chunk() {
    let entities = 3 * REGION_CHUNK_LAYOUT.chunk_slots + 100;
    let (sealed, store) = sealed(&partition(entities, 0));
    let placement = &sealed.placement;
    assert_eq!(placement.entity_domain, entities);
    let directory = placement.admit().unwrap();
    assert_eq!(directory.domain_chunks(SlotDomain::Entity).count(), 4);
    assert_eq!(
        directory.root_semantic_merkle(),
        placement.root_semantic_merkle
    );
    let heads = directory
        .chunks()
        .iter()
        .map(|chunk| decoded::<ChunkHeadImage>(&store, chunk.binding().head_locator()))
        .collect::<Vec<_>>();
    let history = directory
        .chunks()
        .iter()
        .map(|chunk| decoded::<ChunkHistoryImage>(&store, chunk.binding().history_locator()))
        .max()
        .unwrap();
    let pool = heads.iter().max().unwrap() + history + 4096;
    assert!(
        pool < heads.iter().sum::<u64>(),
        "the region exceeds the pool"
    );
    let fill = store
        .bytes(directory.chunks()[0].binding().head_locator())
        .len() as u64;
    let accountant = accountant(pool, fill * 2, 4096);
    let residency = ChunkResidency::new(&accountant, ResidencyPoolKind::Head);
    let scan = |kind, pruning, source: &dyn ChunkMemberSource| {
        let scan = ChunkScan::HeadKind(KindId::new(kind));
        scan_chunks(
            &directory,
            &residency,
            source,
            SlotDomain::Entity,
            scan,
            pruning,
        )
        .unwrap()
    };
    let common = scan(COMMON, ScanPruning::Canonical, &store);
    assert_eq!(common.chunks_read, 4);
    assert_eq!(
        common.hits.len() as u64,
        entities - RARE_SLOTS.count() as u64
    );
    let counting = Counting {
        store: &store,
        read: Mutex::new(BTreeSet::new()),
    };
    let fresh = ChunkResidency::new(&accountant, ResidencyPoolKind::Head);
    let pruned = scan_chunks(
        &directory,
        &fresh,
        &counting,
        SlotDomain::Entity,
        ChunkScan::HeadKind(KindId::new(RARE)),
        ScanPruning::Summaries,
    )
    .unwrap();
    assert_eq!(pruned.chunks_read, 1);
    let expected = RARE_SLOTS
        .map(|slot| ScanHit {
            slot,
            relation: None,
        })
        .collect::<Vec<_>>();
    assert_eq!(pruned.hits, expected);
    assert_eq!(scan(RARE, ScanPruning::Canonical, &store).hits, expected);
    let read = counting.read.into_inner().unwrap();
    for chunk in directory
        .chunks()
        .iter()
        .filter(|chunk| !chunk.semantic().slot_range().contains(RARE_SLOTS.start))
    {
        let binding = chunk.binding();
        assert!(
            !read.contains(&binding.head_locator()),
            "a pruned chunk faulted"
        );
    }
}

#[test]
fn a_hub_vertex_spills_adjacency_into_continuation_pages_and_streams_bounded() {
    let edges = 20_000;
    let (sealed, store) = sealed(&partition(10, edges));
    let placement = &sealed.placement;
    let directory = placement.admit().unwrap();
    let continuation = directory.chunks()[0]
        .binding()
        .continuation_directory_locator()
        .expect("the hub's adjacency spilled");
    assert!(continuation.entry_count > 10);
    assert!(placement.members.len() as u64 > continuation.entry_count + 2);
    let pool = 128 * 1024;
    let decoded_edges =
        edges * std::mem::size_of::<super::super::slot_projection::AdjacencyEdge>() as u64;
    assert!(pool < decoded_edges, "the hub's adjacency exceeds the pool");
    let accountant = accountant(pool, 64 * 1024, 4096);
    let residency = ChunkResidency::new(&accountant, ResidencyPoolKind::Head);
    let scan = ChunkScan::Adjacency(KindId::new(LINK));
    let outcome = scan_chunks(
        &directory,
        &residency,
        &store,
        SlotDomain::Entity,
        scan,
        ScanPruning::Summaries,
    )
    .unwrap();
    let relations = outcome
        .hits
        .iter()
        .map(|hit| (hit.slot, hit.relation.unwrap().local_slot_value()))
        .collect::<Vec<_>>();
    assert_eq!(
        relations,
        (0..edges).map(|edge| (0, edge)).collect::<Vec<_>>()
    );
    assert!(residency.continuation_observation().evictions > 0);
    let none = ChunkScan::Adjacency(KindId::new(LINK + 1));
    let skipped = scan_chunks(
        &directory,
        &residency,
        &store,
        SlotDomain::Entity,
        none,
        ScanPruning::Summaries,
    );
    assert_eq!(skipped.unwrap().chunks_read, 0);
}

#[test]
fn consolidation_rebinds_history_without_moving_the_semantic_root() {
    let (sealed, store) = sealed(&partition(REGION_CHUNK_LAYOUT.chunk_slots + 5, 3));
    let placement = &sealed.placement;
    let directory = placement.admit().unwrap();
    let accountant = accountant(16 << 20, 16 << 20, 4096);
    let residency = ChunkResidency::new(&accountant, ResidencyPoolKind::Historical);
    let consolidated = placement
        .consolidate_history(&directory, &residency, &store, &mut &store)
        .unwrap();
    assert_eq!(
        consolidated.root_semantic_merkle,
        placement.root_semantic_merkle
    );
    consolidated.admit().unwrap();
    let chunk = &directory.chunks()[0];
    // A candidate carrying another chunk's meaning never rebinds history.
    let foreign = directory.chunks()[1].binding().history_locator();
    let refused = consolidate_history_chunk(
        &residency,
        &store,
        chunk.semantic(),
        chunk.binding(),
        foreign,
    );
    assert!(refused.is_err());
    verify_bound_chunk(&residency, &store, chunk.semantic(), chunk.binding()).unwrap();
}

fn tampered(
    placement: &RegionChunkPlacement,
    edit: impl FnOnce(&mut serde_json::Value),
) -> Result<RegionChunkPlacement, serde_json::Error> {
    let mut value = serde_json::to_value(placement).unwrap();
    edit(&mut value);
    serde_json::from_value(value)
}

#[test]
fn placement_admission_and_cold_verification_refuse_every_m3_defect() {
    let (sealed, store) = sealed(&partition(REGION_CHUNK_LAYOUT.chunk_slots * 2, 30));
    let honest = sealed.placement;
    let refuse = |placement: &RegionChunkPlacement| placement.admit().unwrap_err();
    let mut gap = honest.clone();
    gap.chunks.remove(0);
    assert!(matches!(
        refuse(&gap),
        ChunkPlacementRefusal::Coverage(ChunkCoverageRefusal::PartitionMismatch { .. })
    ));
    let mut root = honest.clone();
    root.root_semantic_merkle[0] ^= 1;
    assert_eq!(refuse(&root), ChunkPlacementRefusal::RootMismatch);
    let mut domain = honest.clone();
    domain.entity_domain += 1;
    assert!(matches!(
        refuse(&domain),
        ChunkPlacementRefusal::Coverage(ChunkCoverageRefusal::PartitionMismatch { .. })
    ));
    let mut omitted = honest.clone();
    omitted
        .members
        .retain(|member| *member != honest.chunks[0].1.head_locator());
    assert_eq!(refuse(&omitted), ChunkPlacementRefusal::Roster);
    let mut foreign = honest.clone();
    foreign.members.extend((0..64).map(|byte| Locator {
        sha256: [byte; 32],
        length: 1,
    }));
    foreign.members.sort_unstable();
    assert_eq!(refuse(&foreign), ChunkPlacementRefusal::Roster);
    let mut unsorted = honest.clone();
    unsorted.members.reverse();
    assert_eq!(refuse(&unsorted), ChunkPlacementRefusal::Roster);
    let descending = tampered(&honest, |value| {
        value["chunks"][0][0]["head_summary"]["record_kinds"] = serde_json::json!([9, 3]);
    });
    assert!(descending.is_err(), "a non-canonical summary never decodes");
    // An omitted edge kind with a recomputed root passes admission, but the
    // cold verifier re-derives the entry from the members and refuses it.
    let mut forged = tampered(&honest, |value| {
        value["chunks"][0][0]["head_summary"]["adjacency_kinds"] = serde_json::json!([]);
    })
    .unwrap();
    let entries = forged
        .chunks
        .iter()
        .map(|(entry, _)| entry.clone())
        .collect::<Vec<ChunkSemanticEntry>>();
    forged.root_semantic_merkle = root_semantic_merkle(&entries);
    let directory = forged.admit().unwrap();
    let chunk = &directory.chunks()[0];
    assert!(!chunk
        .semantic()
        .head_summary()
        .may_contain_adjacency(KindId::new(LINK)));
    let accountant = accountant(16 << 20, 16 << 20, 4096);
    let residency = ChunkResidency::new(&accountant, ResidencyPoolKind::Head);
    assert!(matches!(
        verify_bound_chunk(&residency, &store, chunk.semantic(), chunk.binding()),
        Err(ChunkVerificationRefusal::CommitmentDiverged { .. })
    ));
}
