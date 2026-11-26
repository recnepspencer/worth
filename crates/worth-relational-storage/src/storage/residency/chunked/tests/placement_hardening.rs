//! Hardening courts for Store M4.3c placement admission, roster confinement,
//! consolidation validation, incremental capture accounting, and serving
//! custody retention.
use super::super::canonical_read::ChunkResidency;
use super::super::cold_member_port::ConfinedMemberSource;
use super::super::consolidation::ChunkVerificationRefusal;
use super::super::member_io::ChunkMemberSource;
use super::super::region_placement::{
    ChunkPlacementRefusal, RegionChunkPlacement, REGION_CHUNK_LAYOUT,
};
use super::super::region_projection::{seal_region_chunks, RegionChunkCaptureError};
use super::super::slot_range::SlotDomain;
use super::super::summary_scan::{scan_chunks, ChunkScan, ScanPruning};
use super::member_store::{accountant, MemberStore};
use crate::config::data::{AdjacencyBackend, AdjacencyPolicy};
use crate::durability::medium::member_custody::native_slot::NativeMemberRetirementSlot;
use crate::durability::medium::member_custody::RelationalDurableMemberCustody;
use crate::durability::medium::RelationalDurableMemberLocator as Locator;
use crate::identity::data::{KindId, PartitionId, RelationId, VersionId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::adjacency::AdjacencySetFixturePort as _;
use crate::storage::residency::working_set::ResidencyPoolKind;
use crate::storage::substrate::record_arena::arena::slot_lifecycle::metadata::RecordArenaFixturePort as _;
use crate::storage::substrate::{EntityArena, EntityExtra, RelationArena, SlotInit};

const ENTITY_KIND: u32 = 1;
const LINK_KIND: u32 = 40;

fn sample_partition(entities: u64, hub_edges: u64) -> PartitionState {
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
        let init = SlotInit {
            partition_id: id,
            kind_id: KindId::new(ENTITY_KIND),
            version_id: VersionId(1 + slot),
            extra: EntityExtra::default(),
        };
        partition
            .entity_arena
            .write_reserved_slot_for_test(init, slot as usize, 1)
            .unwrap();
    }
    if hub_edges > 0 {
        let hub = partition.adjacency.ensure(0, &policy);
        for relation in 0..hub_edges {
            let id = RelationId::new(id, relation, 1);
            hub.insert_adjacent_id_at_for_test(KindId::new(LINK_KIND), id, VersionId(1));
        }
    }
    partition
}

fn sealed_with_store(partition: &PartitionState) -> (RegionChunkPlacement, MemberStore) {
    let sealed = seal_region_chunks(partition, &mut |_, _| Ok::<(), ()>(())).unwrap();
    let store = MemberStore::default();
    for (locator, bytes) in sealed.members {
        store.replace(locator, bytes);
    }
    (sealed.placement, store)
}

#[test]
fn confined_member_source_refuses_unlisted_continuation_locator() {
    let (placement, store) = sealed_with_store(&sample_partition(4, 20_000));
    let directory = placement.admit().unwrap();
    let binding = directory.chunks()[0].binding();
    let root = binding
        .continuation_directory_locator()
        .expect("hub spills into continuation pages")
        .root_page;
    // Allow only the head, history and continuation-directory root, omitting
    // the continuation pages that remain present in the backing store.
    let confined = ConfinedMemberSource::new(
        store,
        vec![binding.head_locator(), binding.history_locator(), root],
    );
    let mut buf = [0_u8; 16];
    let omitted = placement
        .members
        .iter()
        .copied()
        .find(|m| *m != binding.head_locator() && *m != binding.history_locator() && *m != root)
        .expect("continuation page exists");
    assert_eq!(
        confined.read_range(omitted, 0, &mut buf),
        Err(Cause::Damaged)
    );
    let accountant = accountant(16 << 20, 16 << 20, 4096);
    let residency = ChunkResidency::new(&accountant, ResidencyPoolKind::Head);
    let scan = scan_chunks(
        &directory,
        &residency,
        &confined,
        SlotDomain::Entity,
        ChunkScan::Adjacency(KindId::new(LINK_KIND)),
        ScanPruning::Canonical,
    );
    assert_eq!(scan.unwrap_err(), Cause::Damaged);
}

#[test]
fn admit_refuses_empty_or_overflowing_continuation_and_unknown_wire_fields() {
    let (honest, _) = sealed_with_store(&sample_partition(4, 20_000));
    let edit = |mutate: &dyn Fn(&mut serde_json::Value)| -> Result<RegionChunkPlacement, _> {
        let mut value = serde_json::to_value(&honest).unwrap();
        mutate(&mut value);
        serde_json::from_value(value)
    };
    let zero_entries = edit(&|value| {
        value["chunks"][0][1]["continuation_directory_locator"]["entry_count"] =
            serde_json::json!(0);
    })
    .unwrap();
    assert_eq!(
        zero_entries.admit().unwrap_err(),
        ChunkPlacementRefusal::Roster
    );
    let mut overflow_entries = edit(&|value| {
        value["chunks"][0][1]["continuation_directory_locator"]["entry_count"] =
            serde_json::json!(u64::MAX);
    })
    .unwrap();
    overflow_entries.members.extend((0..32).map(|b| Locator {
        sha256: [b; 32],
        length: 1,
    }));
    overflow_entries.members.sort_unstable();
    overflow_entries.members.dedup();
    assert_eq!(
        overflow_entries.admit().unwrap_err(),
        ChunkPlacementRefusal::Roster
    );
    assert!(
        edit(&|value| {
            value["chunks"][0][1]["extra_field"] = serde_json::json!(1);
        })
        .is_err(),
        "ChunkLocatorBinding must reject unknown wire fields"
    );
    assert!(
        edit(&|value| {
            value["chunks"][0][1]["slot_range"]["extra_field"] = serde_json::json!(1);
        })
        .is_err(),
        "SlotRange must reject unknown wire fields"
    );
}

#[test]
fn consolidate_history_refuses_mismatched_directory() {
    let (placement_a, store) = sealed_with_store(&sample_partition(4, 0));
    let (placement_b, _) = sealed_with_store(&sample_partition(5, 0));
    let directory_b = placement_b.admit().unwrap();
    let accountant = accountant(16 << 20, 16 << 20, 4096);
    let residency = ChunkResidency::new(&accountant, ResidencyPoolKind::Historical);
    let result = placement_a.consolidate_history(&directory_b, &residency, &store, &mut &store);
    assert_eq!(result.unwrap_err(), ChunkVerificationRefusal::RangeMismatch);
}

#[test]
fn seal_region_chunks_charges_edges_and_member_bytes_incrementally() {
    let plain = sample_partition(4, 0);
    let hub = sample_partition(4, 3_000);
    let mut plain_units = 0_u64;
    seal_region_chunks(&plain, &mut |units, _| {
        plain_units += units;
        Ok::<(), ()>(())
    })
    .unwrap();
    let mut hub_units = 0_u64;
    let mut byte_calls = 0_u64;
    seal_region_chunks(&hub, &mut |units, bytes| {
        hub_units += units;
        if bytes > 0 {
            byte_calls += 1;
        }
        Ok::<(), ()>(())
    })
    .unwrap();
    assert!(
        hub_units >= plain_units + 3_000,
        "outgoing adjacency edges must be charged to work units"
    );
    assert!(
        byte_calls > 1,
        "member bytes must be charged incrementally as each unique member is sealed"
    );
    let stopped = seal_region_chunks(
        &sample_partition(REGION_CHUNK_LAYOUT.chunk_slots + 2, 0),
        &mut |_, bytes| if bytes > 0 { Err("budget") } else { Ok(()) },
    );
    assert!(matches!(
        stopped,
        Err(RegionChunkCaptureError::Stop("budget"))
    ));
}

#[test]
fn native_member_slot_try_retain_serving_returns_none_while_loaned() {
    let mut slot = NativeMemberRetirementSlot::Original(RelationalDurableMemberCustody::new(()));
    assert!(slot.try_retain_serving().is_some());
    let handoff = slot.loan().unwrap();
    assert!(handoff.is_some());
    assert!(slot.try_retain_serving().is_none());
}
