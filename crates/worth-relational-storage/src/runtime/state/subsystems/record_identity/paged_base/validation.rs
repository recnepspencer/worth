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

    fn cold_source_identity(&self) -> Option<usize> {
        self.inner.cold_source_identity()
    }
}

impl super::PagedIdentityBase {
    pub(in super::super) fn source_matches(
        &self,
        source: &dyn crate::durability::medium::cold_source::RelationalDurableColdSource,
    ) -> bool {
        let expected = (source
            as *const dyn crate::durability::medium::cold_source::RelationalDurableColdSource)
            .cast::<()>() as usize;
        self.source.cold_source_identity() == Some(expected)
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
