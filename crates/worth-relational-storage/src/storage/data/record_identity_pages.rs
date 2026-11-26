//! `CHK24` paged record-identity tables.
//!
//! The four allocator tables (generation high-water marks, reusable slots,
//! per-partition append frontiers and pending reservations) are immutable
//! content-addressed paged directories. A manifest retains only their
//! fixed-size roots; reopen faults one partition's rows on first use, so its
//! memory does not grow with record, reservation or partition count.
use serde::{Deserialize, Serialize};

use super::record_identity_image::{
    DurablePendingRecordReservation, DurableRecordGenerationClass,
    DurableRecordGenerationHighWater, DurableRecordIdentityState, DurableRecordReservationOrigin,
    DurableRecordSlotFrontier, DurableReusableRecordSlot,
};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::identity::data::PartitionId;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::chunked::member_io::{ChunkMemberSink, ChunkMemberSource};
use crate::storage::residency::chunked::page_cache::PageEvictionGroup;
use crate::storage::residency::chunked::paged_directory::{
    DirectoryBuildError, DirectoryEntry, PagedDirectory, PagedDirectoryBuilder, PagedDirectoryRoot,
};
use crate::storage::residency::working_set::{ResidencyPoolKind, WorkingSetAccountant};
use std::sync::Arc;

/// `(class, partition, slot)`; class `0` is entity, `1` relation.
pub type IdentitySlotKey = (u8, u32, u64);
/// `(class, partition)`.
pub type IdentityFrontierKey = (u8, u32);
/// `(generation, prior generation)`; no prior generation is an append slot.
pub type IdentityPendingValue = (u32, Option<u32>);

/// Entries per identity directory page and the bound on one page member.
pub const IDENTITY_PAGE_ENTRIES: usize = 256;
pub const IDENTITY_PAGE_BYTES: usize = 64 * 1024;

/// The fixed-size roots a `CHK24` manifest retains for record identity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PagedRecordIdentityRoots {
    pub generation_high_water: Option<PagedDirectoryRoot>,
    pub reusable_slots: Option<PagedDirectoryRoot>,
    pub append_frontiers: Option<PagedDirectoryRoot>,
    pub pending_reservations: Option<PagedDirectoryRoot>,
    /// Highest partition with an append frontier, so partition allocation
    /// needs no frontier scan.
    pub partition_high_water: Option<PartitionId>,
}

pub const fn class_tag(class: DurableRecordGenerationClass) -> u8 {
    match class {
        DurableRecordGenerationClass::Entity => 0,
        DurableRecordGenerationClass::Relation => 1,
    }
}

pub fn class_of(tag: u8) -> Result<DurableRecordGenerationClass, Cause> {
    match tag {
        0 => Ok(DurableRecordGenerationClass::Entity),
        1 => Ok(DurableRecordGenerationClass::Relation),
        _ => Err(Cause::Damaged),
    }
}

pub fn pending_value(
    origin: DurableRecordReservationOrigin,
    generation: u32,
) -> IdentityPendingValue {
    match origin {
        DurableRecordReservationOrigin::AppendFrontier => (generation, None),
        DurableRecordReservationOrigin::Reclaimed { prior_generation } => {
            (generation, Some(prior_generation))
        }
    }
}

pub fn pending_origin(value: IdentityPendingValue) -> DurableRecordReservationOrigin {
    match value.1 {
        None => DurableRecordReservationOrigin::AppendFrontier,
        Some(prior_generation) => DurableRecordReservationOrigin::Reclaimed { prior_generation },
    }
}

/// Writes `state`'s four tables as paged directories into `sink`.
pub fn encode_record_identity(
    state: &DurableRecordIdentityState,
    sink: &mut dyn ChunkMemberSink,
) -> Result<PagedRecordIdentity, Cause> {
    let sink = &mut Roster {
        inner: sink,
        pages: Vec::new(),
    };
    let slot = |class, partition: PartitionId, slot| (class_tag(class), partition.0, slot);
    let roots = PagedRecordIdentityRoots {
        generation_high_water: build(
            sink,
            state
                .generation_high_water
                .iter()
                .map(|row| (slot(row.class, row.partition_id, row.slot), row.generation)),
        )?,
        reusable_slots: build(
            sink,
            state
                .reusable_slots
                .iter()
                .map(|row| (slot(row.class, row.partition_id, row.slot), ())),
        )?,
        append_frontiers: build(
            sink,
            state
                .append_frontiers
                .iter()
                .map(|row| ((class_tag(row.class), row.partition_id.0), row.next_slot)),
        )?,
        pending_reservations: build(
            sink,
            state.pending_reservations.iter().map(|row| {
                (
                    slot(row.class, row.partition_id, row.slot),
                    pending_value(row.origin, row.generation),
                )
            }),
        )?,
        partition_high_water: state
            .append_frontiers
            .iter()
            .map(|row| row.partition_id)
            .max(),
    };
    let mut pages = std::mem::take(&mut sink.pages);
    pages.sort_unstable();
    pages.dedup();
    Ok(PagedRecordIdentity { roots, pages })
}

/// The record-identity facts a `CHK24` manifest retains: the fixed-size roots
/// and the page roster that checkpoint custody pins. Admission reads no page;
/// the roster is one locator per `IDENTITY_PAGE_ENTRIES` rows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PagedRecordIdentity {
    pub roots: PagedRecordIdentityRoots,
    pub pages: Vec<RelationalDurableMemberLocator>,
}

/// Records every page locator a directory build writes.
struct Roster<'sink> {
    inner: &'sink mut dyn ChunkMemberSink,
    pages: Vec<RelationalDurableMemberLocator>,
}

impl ChunkMemberSink for Roster<'_> {
    fn put(
        &mut self,
        locator: RelationalDurableMemberLocator,
        bytes: Vec<u8>,
    ) -> Result<(), Cause> {
        self.pages.push(locator);
        self.inner.put(locator, bytes)
    }
}

/// Builds one directory from rows in any order (capture sorts once).
pub fn build<K: DirectoryEntry + Ord, V: DirectoryEntry>(
    sink: &mut dyn ChunkMemberSink,
    rows: impl Iterator<Item = (K, V)>,
) -> Result<Option<PagedDirectoryRoot>, Cause> {
    let mut rows = rows.collect::<Vec<_>>();
    rows.sort_unstable_by_key(|(key, _)| *key);
    let mut builder = PagedDirectoryBuilder::new(IDENTITY_PAGE_ENTRIES, IDENTITY_PAGE_BYTES)
        .map_err(build_cause)?;
    for (key, value) in rows {
        builder.push(sink, key, value).map_err(build_cause)?;
    }
    builder.finish(sink).map_err(build_cause)
}

fn build_cause(error: DirectoryBuildError) -> Cause {
    match error {
        DirectoryBuildError::Member(cause) => cause,
        // A duplicate key is a damaged allocator image, never a valid table.
        DirectoryBuildError::KeyOutOfOrder | DirectoryBuildError::PageTooSmall => Cause::Damaged,
    }
}

impl PagedRecordIdentityRoots {
    /// The whole allocator image, for expanded decode and inspection only.
    /// Reopen never calls this; it faults partitions through the paged base.
    pub fn load_resident(
        &self,
        directories: &IdentityDirectories,
        source: &dyn ChunkMemberSource,
    ) -> Result<DurableRecordIdentityState, Cause> {
        let mut generations = Vec::new();
        let mut cursor = directories.generations.range_from(source, (0, 0, 0))?;
        while let Some(((class, partition, slot), generation)) = cursor.next_entry()? {
            generations.push(DurableRecordGenerationHighWater {
                class: class_of(class)?,
                partition_id: PartitionId(partition),
                slot,
                generation,
            });
        }
        let mut reusable = Vec::new();
        let mut cursor = directories.reusable.range_from(source, (0, 0, 0))?;
        while let Some(((class, partition, slot), ())) = cursor.next_entry()? {
            reusable.push(DurableReusableRecordSlot {
                class: class_of(class)?,
                partition_id: PartitionId(partition),
                slot,
            });
        }
        let mut frontiers = Vec::new();
        let mut cursor = directories.frontiers.range_from(source, (0, 0))?;
        while let Some(((class, partition), next_slot)) = cursor.next_entry()? {
            frontiers.push(DurableRecordSlotFrontier {
                class: class_of(class)?,
                partition_id: PartitionId(partition),
                next_slot,
            });
        }
        let mut pending = Vec::new();
        let mut cursor = directories.pending.range_from(source, (0, 0, 0))?;
        while let Some(((class, partition, slot), value)) = cursor.next_entry()? {
            pending.push(DurablePendingRecordReservation {
                class: class_of(class)?,
                partition_id: PartitionId(partition),
                slot,
                generation: value.0,
                origin: pending_origin(value),
            });
        }
        Ok(DurableRecordIdentityState::current_with_memory(
            generations,
            reusable,
            frontiers,
            pending,
            None,
        ))
    }
}

/// The four directories opened over one page-eviction group charged to the
/// home's working set.
pub struct IdentityDirectories {
    pub generations: PagedDirectory<IdentitySlotKey, u32>,
    pub reusable: PagedDirectory<IdentitySlotKey, ()>,
    pub frontiers: PagedDirectory<IdentityFrontierKey, u64>,
    pub pending: PagedDirectory<IdentitySlotKey, IdentityPendingValue>,
}

impl IdentityDirectories {
    /// Opens the directories without reading any page.
    pub fn open(roots: &PagedRecordIdentityRoots, accountant: Arc<WorkingSetAccountant>) -> Self {
        let group = PageEvictionGroup::new();
        let pool = ResidencyPoolKind::Head;
        Self {
            generations: PagedDirectory::open(
                roots.generation_high_water,
                group.cache(accountant.clone(), pool),
            ),
            reusable: PagedDirectory::open(
                roots.reusable_slots,
                group.cache(accountant.clone(), pool),
            ),
            frontiers: PagedDirectory::open(
                roots.append_frontiers,
                group.cache(accountant.clone(), pool),
            ),
            pending: PagedDirectory::open(
                roots.pending_reservations,
                group.cache(accountant, pool),
            ),
        }
    }
}
