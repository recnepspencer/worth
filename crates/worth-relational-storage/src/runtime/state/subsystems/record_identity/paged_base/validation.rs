//! Roster confinement, page integrity verification, and pending-reservation
//! consistency checks for `CHK24` paged record-identity tables.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::{BaseRows, PartitionRows};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::history::data::canonical_record_allocation::RecordAllocationOrigin;
use crate::identity::data::PartitionId;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::chunked::member_frame::{self, ChunkMemberKind};
use crate::storage::residency::chunked::member_io::ChunkMemberSource;

/// Confines every cold page read to the manifest's validated, pinned `pages`
/// roster so no branch page can reference an unlisted/unpinned locator.
pub(super) struct ConfinedMemberSource {
    inner: Arc<dyn ChunkMemberSource + Send + Sync>,
    allowed: Vec<RelationalDurableMemberLocator>,
}

impl ConfinedMemberSource {
    pub(super) fn new(
        inner: Arc<dyn ChunkMemberSource + Send + Sync>,
        pages: &[RelationalDurableMemberLocator],
    ) -> Self {
        let mut allowed = pages.to_vec();
        allowed.sort_unstable();
        allowed.dedup();
        Self { inner, allowed }
    }
}

impl ChunkMemberSource for ConfinedMemberSource {
    fn read_range(
        &self,
        member: RelationalDurableMemberLocator,
        offset: u64,
        target: &mut [u8],
    ) -> Result<usize, Cause> {
        if self.allowed.binary_search(&member).is_err() {
            return Err(Cause::Damaged);
        }
        self.inner.read_range(member, offset, target)
    }
}

/// Bounds and verifies one table scan against its manifest `entry_count`.
pub(super) struct RowCount {
    seen: u64,
    expected: u64,
}

impl RowCount {
    pub(super) fn new(
        root: Option<crate::storage::residency::chunked::paged_directory::PagedDirectoryRoot>,
    ) -> Self {
        Self {
            seen: 0,
            expected: root.map_or(0, |r| r.entry_count),
        }
    }

    pub(super) fn step(&mut self) -> Result<(), Cause> {
        if self.seen >= self.expected {
            return Err(Cause::Damaged);
        }
        self.seen += 1;
        Ok(())
    }

    pub(super) fn finish(self) -> Result<(), Cause> {
        if self.seen != self.expected {
            return Err(Cause::Damaged);
        }
        Ok(())
    }
}

impl super::PagedIdentityBase {
    /// Opens paged tables backed by a managed `NativeSourceMemberState` custody
    /// pin so checkpoint replacement retains unfaulted pages until close.
    pub fn open_with_custody(
        paged: crate::storage::data::record_identity_pages::PagedRecordIdentity,
        accountant: Arc<crate::storage::residency::working_set::WorkingSetAccountant>,
        physical: crate::durability::medium::member_custody::native_source_slot::NativeSourceMemberState,
        member_limit: u64,
    ) -> Self {
        let cold = Arc::new(
            crate::storage::residency::chunked::cold_member_port::ColdMemberSource::with_custody(
                physical,
                member_limit,
            ),
        );
        let confined = Arc::new(ConfinedMemberSource::new(cold.clone(), &paged.pages));
        Self {
            directories: crate::storage::data::record_identity_pages::IdentityDirectories::open(
                &paged.roots,
                accountant,
            ),
            roots: paged.roots,
            paged,
            source: confined,
            custody: Some(cold),
        }
    }

    /// One page member's verified bytes, for a capture whose destination lacks it.
    pub fn page_bytes(&self, page: RelationalDurableMemberLocator) -> Result<Vec<u8>, Cause> {
        let limit = crate::storage::data::record_identity_pages::IDENTITY_PAGE_BYTES as u64;
        if page.length > limit {
            return Err(Cause::LimitExceeded {
                length: page.length,
                limit,
            });
        }
        let length = usize::try_from(page.length).map_err(|_| Cause::MemoryExhausted)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| Cause::MemoryExhausted)?;
        bytes.resize(length, 0);
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
        verify_page_bytes(page, &bytes)?;
        Ok(bytes)
    }

    /// Loads the entire allocator image with roster confinement, exact entry
    /// counts, and pending-reservation consistency checks.
    pub fn load_resident(
        &self,
    ) -> Result<crate::storage::data::record_identity_image::DurableRecordIdentityState, Cause>
    {
        use crate::history::data::canonical_record_allocation::RecordAllocationClass;
        use crate::storage::data::record_identity_image::{
            DurablePendingRecordReservation, DurableRecordGenerationClass,
            DurableRecordGenerationHighWater, DurableRecordIdentityState,
            DurableRecordReservationOrigin, DurableRecordSlotFrontier, DurableReusableRecordSlot,
        };
        let class = |c| match c {
            RecordAllocationClass::Entity => DurableRecordGenerationClass::Entity,
            RecordAllocationClass::Relation => DurableRecordGenerationClass::Relation,
        };
        let rows = self.all_rows()?;
        Ok(DurableRecordIdentityState::current_with_memory(
            rows.generations
                .into_iter()
                .map(
                    |((c, partition_id, slot), generation)| DurableRecordGenerationHighWater {
                        class: class(c),
                        partition_id,
                        slot: slot as u64,
                        generation,
                    },
                )
                .collect(),
            rows.reusable
                .into_iter()
                .map(|(c, partition_id, slot)| DurableReusableRecordSlot {
                    class: class(c),
                    partition_id,
                    slot: slot as u64,
                })
                .collect(),
            rows.frontiers
                .into_iter()
                .map(|((c, partition_id), next_slot)| DurableRecordSlotFrontier {
                    class: class(c),
                    partition_id,
                    next_slot: next_slot as u64,
                })
                .collect(),
            rows.pending
                .into_iter()
                .map(
                    |((c, partition_id, slot), reservation)| DurablePendingRecordReservation {
                        class: class(c),
                        partition_id,
                        slot: slot as u64,
                        generation: reservation.generation,
                        origin: match reservation.origin {
                            RecordAllocationOrigin::AppendFrontier => {
                                DurableRecordReservationOrigin::AppendFrontier
                            }
                            RecordAllocationOrigin::Reclaimed { prior_generation } => {
                                DurableRecordReservationOrigin::Reclaimed { prior_generation }
                            }
                        },
                    },
                )
                .collect(),
            None,
        ))
    }

    pub(in super::super) fn preflight_source_retirement(
        &self,
        source: &dyn crate::durability::medium::cold_source::RelationalDurableColdSource,
    ) -> Result<(), crate::durability::medium::member_custody::native_slot::NativeMemberSlotDenial>
    {
        match &self.custody {
            Some(cold) => cold.preflight_source_retirement(source),
            None => Ok(()),
        }
    }

    pub(in super::super) fn loan_source_retirement(
        &self,
        handoff: &mut Option<
            crate::durability::medium::member_custody::native_slot::NativeMemberRetirementHandoff,
        >,
    ) -> Result<bool, crate::durability::medium::member_custody::native_slot::NativeMemberSlotDenial>
    {
        match &self.custody {
            Some(cold) => cold.loan_source_retirement(handoff),
            None => Ok(true),
        }
    }

    pub(in super::super) fn return_source_retirement(
        &self,
        handoff: &mut Option<
            crate::durability::medium::member_custody::native_slot::NativeMemberRetirementHandoff,
        >,
        source: &dyn crate::durability::medium::cold_source::RelationalDurableColdSource,
    ) -> Result<
        crate::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementObservation,
        crate::durability::medium::member_custody::native_slot::NativeMemberSlotDenial,
    >{
        match &self.custody {
            Some(cold) => cold.return_source_retirement(handoff, source),
            None => Err(
                crate::durability::medium::member_custody::native_slot::NativeMemberSlotDenial::Refused(
                    crate::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementRefusal::ForeignCustody,
                ),
            ),
        }
    }
}

/// Verifies length, SHA-256 digest, and `DirectoryPage` chunk framing of one
/// raw identity page copied during capture.
pub(super) fn verify_page_bytes(
    locator: RelationalDurableMemberLocator,
    bytes: &[u8],
) -> Result<(), Cause> {
    if bytes.len() as u64 != locator.length
        || <[u8; 32]>::from(Sha256::digest(bytes)) != locator.sha256
    {
        return Err(Cause::IntegrityMismatch);
    }
    let frame = bytes
        .strip_prefix(member_frame::MAGIC)
        .ok_or(Cause::Damaged)?;
    let (&tag, _) = frame.split_first().ok_or(Cause::Damaged)?;
    if tag != ChunkMemberKind::DirectoryPage as u8 {
        return Err(Cause::Damaged);
    }
    Ok(())
}

/// Enforces the same pending-reservation consistency invariants as eager
/// `restore_pending` before mutating resident allocator maps.
pub(super) fn validate_partition_rows(rows: &PartitionRows) -> Result<(), Cause> {
    if rows.pending.is_empty() {
        return Ok(());
    }
    let generations = rows
        .generations
        .iter()
        .copied()
        .collect::<BTreeMap<usize, u32>>();
    let reusable = rows.reusable.iter().copied().collect::<BTreeSet<usize>>();
    for &(slot, reservation) in &rows.pending {
        if generations.get(&slot) != Some(&reservation.generation) {
            return Err(Cause::Damaged);
        }
        match reservation.origin {
            RecordAllocationOrigin::AppendFrontier => {
                if !rows.frontier.is_some_and(|next| slot < next) {
                    return Err(Cause::Damaged);
                }
            }
            RecordAllocationOrigin::Reclaimed { .. } => {
                if reusable.contains(&slot) {
                    return Err(Cause::Damaged);
                }
            }
        }
    }
    Ok(())
}

/// Enforces frontier high-water bounds and pending-reservation invariants
/// across a full paged-table scan.
pub(super) fn validate_all_rows(
    rows: &BaseRows,
    partition_high_water: Option<PartitionId>,
) -> Result<(), Cause> {
    for &(_, partition_id) in rows.frontiers.keys() {
        if !partition_high_water.is_some_and(|hw| partition_id <= hw) {
            return Err(Cause::Damaged);
        }
    }
    if rows.pending.is_empty() {
        return Ok(());
    }
    let reusable = rows.reusable.iter().copied().collect::<BTreeSet<_>>();
    for &(key, reservation) in &rows.pending {
        if rows.generations.get(&key) != Some(&reservation.generation) {
            return Err(Cause::Damaged);
        }
        match reservation.origin {
            RecordAllocationOrigin::AppendFrontier => {
                if !rows
                    .frontiers
                    .get(&(key.0, key.1))
                    .is_some_and(|&next| key.2 < next)
                {
                    return Err(Cause::Damaged);
                }
            }
            RecordAllocationOrigin::Reclaimed { .. } => {
                if reusable.contains(&key) {
                    return Err(Cause::Damaged);
                }
            }
        }
    }
    Ok(())
}
