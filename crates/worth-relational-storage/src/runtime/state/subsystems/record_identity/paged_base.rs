//! A reopened allocator's checkpoint tables, faulted one partition at a time
//! (`G7`). Reopen retains only the fixed-size roots; the first operation on a
//! `(class, partition)` merges that partition's rows into the resident maps.
//! Every later operation on it is the ordinary resident algorithm, so slot
//! reuse, generations, frontiers and pending reservations stay exact.
use std::collections::BTreeMap;
use std::sync::Arc;

mod validation;

use super::{PendingRecordReservation, RecordFrontierKey, RecordIdentityState, RecordSlotKey};
use crate::history::data::canonical_record_allocation::{
    RecordAllocationClass, RecordAllocationOrigin,
};
use crate::identity::data::PartitionId;
use crate::storage::data::record_identity_pages::{
    IdentityDirectories, IdentitySlotKey, PagedRecordIdentity, PagedRecordIdentityRoots,
};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::chunked::member_io::ChunkMemberSource;
use crate::storage::residency::working_set::WorkingSetAccountant;

pub struct PagedIdentityBase {
    roots: PagedRecordIdentityRoots,
    paged: PagedRecordIdentity,
    directories: IdentityDirectories,
    source: Arc<dyn ChunkMemberSource + Send + Sync>,
}

impl std::fmt::Debug for PagedIdentityBase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PagedIdentityBase")
            .field("roots", &self.roots)
            .finish_non_exhaustive()
    }
}

/// One partition's checkpoint rows in one allocation class.
#[derive(Default)]
pub(super) struct PartitionRows {
    generations: Vec<(usize, u32)>,
    reusable: Vec<usize>,
    frontier: Option<usize>,
    pending: Vec<(usize, PendingRecordReservation)>,
}

pub(super) const fn class_tag(class: RecordAllocationClass) -> u8 {
    match class {
        RecordAllocationClass::Entity => 0,
        RecordAllocationClass::Relation => 1,
    }
}

fn class_of(tag: u8) -> Result<RecordAllocationClass, Cause> {
    match tag {
        0 => Ok(RecordAllocationClass::Entity),
        1 => Ok(RecordAllocationClass::Relation),
        _ => Err(Cause::Damaged),
    }
}

fn slot(value: u64) -> Result<usize, Cause> {
    usize::try_from(value).map_err(|_| Cause::Damaged)
}

fn origin(prior: Option<u32>) -> RecordAllocationOrigin {
    prior.map_or(RecordAllocationOrigin::AppendFrontier, |prior_generation| {
        RecordAllocationOrigin::Reclaimed { prior_generation }
    })
}

impl PagedIdentityBase {
    /// Opens the tables without reading a page; pages charge `accountant` and
    /// every cold read is confined to `paged.pages`.
    pub fn open(
        paged: PagedRecordIdentity,
        accountant: Arc<WorkingSetAccountant>,
        source: Arc<dyn ChunkMemberSource + Send + Sync>,
    ) -> Self {
        let confined = Arc::new(validation::ConfinedMemberSource::new(source, &paged.pages));
        Self {
            directories: IdentityDirectories::open(&paged.roots, accountant),
            roots: paged.roots,
            paged,
            source: confined,
        }
    }

    /// The checkpoint facts these tables were reopened from.
    pub fn paged(&self) -> &PagedRecordIdentity {
        &self.paged
    }

    /// One page member's verified bytes, for a capture whose destination lacks it.
    pub fn page_bytes(
        &self,
        page: crate::durability::medium::RelationalDurableMemberLocator,
    ) -> Result<Vec<u8>, Cause> {
        let length = usize::try_from(page.length).map_err(|_| Cause::MemoryExhausted)?;
        let mut bytes = vec![0; length];
        let mut filled = 0;
        while filled < length {
            match self
                .source
                .read_range(page, filled as u64, &mut bytes[filled..])?
            {
                0 => return Err(Cause::MissingMember),
                read => filled += read,
            }
        }
        validation::verify_page_bytes(page, &bytes)?;
        Ok(bytes)
    }

    pub fn roots(&self) -> PagedRecordIdentityRoots {
        self.roots
    }

    /// Rows of one table whose key lies in `(tag, partition, *)`.
    fn scan<V: crate::storage::residency::chunked::paged_directory::DirectoryEntry>(
        &self,
        directory: &crate::storage::residency::chunked::paged_directory::PagedDirectory<
            IdentitySlotKey,
            V,
        >,
        tag: u8,
        partition: u32,
        mut row: impl FnMut(u64, V) -> Result<(), Cause>,
    ) -> Result<(), Cause> {
        let mut cursor = directory.range_from(self.source.as_ref(), (tag, partition, 0))?;
        while let Some(((found_tag, found_partition, slot), value)) = cursor.next_entry()? {
            if (found_tag, found_partition) != (tag, partition) {
                break;
            }
            row(slot, value)?;
        }
        Ok(())
    }

    pub(super) fn partition_rows(
        &self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<PartitionRows, Cause> {
        let (tag, partition) = (class_tag(class), partition_id.0);
        let mut rows = PartitionRows::default();
        self.scan(
            &self.directories.generations,
            tag,
            partition,
            |at, generation| {
                rows.generations.push((slot(at)?, generation));
                Ok(())
            },
        )?;
        self.scan(&self.directories.reusable, tag, partition, |at, ()| {
            rows.reusable.push(slot(at)?);
            Ok(())
        })?;
        self.scan(
            &self.directories.pending,
            tag,
            partition,
            |at, (generation, prior)| {
                let reservation = PendingRecordReservation {
                    generation,
                    origin: origin(prior),
                };
                rows.pending.push((slot(at)?, reservation));
                Ok(())
            },
        )?;
        rows.frontier = self
            .directories
            .frontiers
            .get(self.source.as_ref(), (tag, partition))?
            .map(slot)
            .transpose()?;
        if rows.frontier.is_some()
            && !self
                .roots
                .partition_high_water
                .is_some_and(|hw| partition_id <= hw)
        {
            return Err(Cause::Damaged);
        }
        validation::validate_partition_rows(&rows)?;
        Ok(rows)
    }

    /// Every row of the four tables, for a checkpoint snapshot.
    pub(super) fn all_rows(&self) -> Result<BaseRows, Cause> {
        let mut rows = BaseRows::default();
        let source = self.source.as_ref();
        let mut cursor = self.directories.generations.range_from(source, (0, 0, 0))?;
        while let Some(((tag, partition, at), generation)) = cursor.next_entry()? {
            let key = (class_of(tag)?, PartitionId(partition), slot(at)?);
            rows.generations.insert(key, generation);
        }
        let mut cursor = self.directories.reusable.range_from(source, (0, 0, 0))?;
        while let Some(((tag, partition, at), ())) = cursor.next_entry()? {
            rows.reusable
                .push((class_of(tag)?, PartitionId(partition), slot(at)?));
        }
        let mut cursor = self.directories.frontiers.range_from(source, (0, 0))?;
        while let Some(((tag, partition), next)) = cursor.next_entry()? {
            rows.frontiers
                .insert((class_of(tag)?, PartitionId(partition)), slot(next)?);
        }
        let mut cursor = self.directories.pending.range_from(source, (0, 0, 0))?;
        while let Some(((tag, partition, at), (generation, prior))) = cursor.next_entry()? {
            let key = (class_of(tag)?, PartitionId(partition), slot(at)?);
            let reservation = PendingRecordReservation {
                generation,
                origin: origin(prior),
            };
            rows.pending.push((key, reservation));
        }
        validation::validate_all_rows(&rows, self.roots.partition_high_water)?;
        Ok(rows)
    }

    pub(super) fn entry_count(&self) -> Option<u64> {
        [
            self.roots.generation_high_water,
            self.roots.reusable_slots,
            self.roots.append_frontiers,
            self.roots.pending_reservations,
        ]
        .into_iter()
        .flatten()
        .try_fold(0u64, |acc, root| acc.checked_add(root.entry_count))
    }
}

#[derive(Default)]
pub(super) struct BaseRows {
    pub(super) generations: BTreeMap<RecordSlotKey, u32>,
    pub(super) reusable: Vec<RecordSlotKey>,
    pub(super) frontiers: BTreeMap<RecordFrontierKey, usize>,
    pub(super) pending: Vec<(RecordSlotKey, PendingRecordReservation)>,
}

impl RecordIdentityState {
    /// Merges one partition's checkpoint rows before its first operation.
    /// Resident writes made meanwhile (reclamation) are unions and maxima, so
    /// merging later is exact. A refusal leaves the partition unfaulted.
    pub(super) fn fault(
        &mut self,
        class: RecordAllocationClass,
        partition_id: PartitionId,
    ) -> Result<(), Cause> {
        let Some(base) = self.paged.clone() else {
            return Ok(());
        };
        if self.faulted.contains(&(class, partition_id)) {
            return Ok(());
        }
        let rows = base.partition_rows(class, partition_id)?;
        for (slot, generation) in rows.generations {
            let held = self
                .generation_high_water
                .entry((class, partition_id, slot))
                .or_default();
            *held = (*held).max(generation);
        }
        for slot in rows.reusable {
            self.reusable_insert((class, partition_id, slot));
        }
        if let Some(next) = rows.frontier
            && self
                .next_slots
                .get(&(class, partition_id))
                .is_none_or(|held| *held < next)
        {
            self.next_slots.insert((class, partition_id), next);
        }
        for (slot, reservation) in rows.pending {
            let key = (class, partition_id, slot);
            if !self.base_pending_released {
                self.pending.insert(key, reservation);
            } else if matches!(reservation.origin, RecordAllocationOrigin::Reclaimed { .. }) {
                // Released after replay exactly as the live drop releases it.
                self.reusable_insert(key);
            }
        }
        self.faulted.insert((class, partition_id));
        Ok(())
    }

    fn reusable_insert(&mut self, key: RecordSlotKey) {
        if !self.reusable.contains_key(&key) {
            self.reusable.insert(key, ());
        }
    }
}

/// The allocator image a checkpoint captures: resident maps for faulted
/// partitions, checkpoint rows merged by the same union/maximum rules for
/// the rest.
pub(super) struct MergedRows {
    pub(super) generations: BTreeMap<RecordSlotKey, u32>,
    pub(super) reusable: std::collections::BTreeSet<RecordSlotKey>,
    pub(super) frontiers: BTreeMap<RecordFrontierKey, usize>,
    pub(super) pending: BTreeMap<RecordSlotKey, PendingRecordReservation>,
}

impl RecordIdentityState {
    pub(super) fn merged_rows(&self) -> Result<Option<MergedRows>, Cause> {
        let Some(base) = &self.paged else {
            return Ok(None);
        };
        let rows = base.all_rows()?;
        let faulted = |key: &RecordSlotKey| self.faulted.contains(&(key.0, key.1));
        let mut generations = rows.generations;
        for (&key, &generation) in &self.generation_high_water {
            let held = generations.entry(key).or_default();
            *held = (*held).max(generation);
        }
        let mut frontiers = rows.frontiers;
        for (&key, &next) in self.next_slots.iter() {
            let held = frontiers.entry(key).or_default();
            *held = (*held).max(next);
        }
        let mut reusable = rows
            .reusable
            .into_iter()
            .filter(|key| !faulted(key))
            .collect::<std::collections::BTreeSet<_>>();
        reusable.extend(self.reusable.iter().map(|(key, ())| *key));
        let mut pending = self.pending.clone();
        for (key, reservation) in rows.pending.into_iter().filter(|(key, _)| !faulted(key)) {
            if !self.base_pending_released {
                pending.insert(key, reservation);
            } else if matches!(reservation.origin, RecordAllocationOrigin::Reclaimed { .. }) {
                reusable.insert(key);
            }
        }
        Ok(Some(MergedRows {
            generations,
            reusable,
            frontiers,
            pending,
        }))
    }
}

/// The allocator image one checkpoint capture freezes.
pub enum FrozenRecordIdentity {
    /// The allocator changed since reopen (or never reopened from pages).
    Resident(crate::storage::data::record_identity_image::DurableRecordIdentityState),
    /// Unchanged since reopen: its immutable checkpoint pages are the image.
    Carried(Arc<PagedIdentityBase>),
}

impl RecordIdentityState {
    /// Whether these maps add nothing to the reopened checkpoint tables.
    fn unchanged_since_reopen(&self) -> Option<Arc<PagedIdentityBase>> {
        let base = self.paged.as_ref()?;
        let released_nothing =
            !self.base_pending_released || base.roots.pending_reservations.is_none();
        (self.faulted.is_empty()
            && self.generation_high_water.is_empty()
            && self.reusable.is_empty()
            && self.next_slots.is_empty()
            && self.pending.is_empty()
            && released_nothing)
            .then(|| base.clone())
    }
}

impl super::RecordIdentitySubsystem {
    /// Returns the reopened paged base when the allocator has not changed since
    /// reopen, so ordinary and complete captures can carry its pages unread.
    pub fn unchanged_paged_identity(&self) -> Option<Arc<PagedIdentityBase>> {
        self.lock().unchanged_since_reopen()
    }

    /// Freezes the image a capture records, carrying untouched checkpoint
    /// pages without reading them.
    pub fn frozen_checkpoint_identity_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<FrozenRecordIdentity, crate::durability::data::recovery_stop::RelationalRecoveryStop>
    {
        if let Some(base) = self.lock().unchanged_since_reopen() {
            return Ok(FrozenRecordIdentity::Carried(base));
        }
        self.checkpoint_identity_for_request(request)
            .map(FrozenRecordIdentity::Resident)
    }
}
