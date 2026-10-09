//! Peak retained media observations for one ordered root-step comparison.

use crate::orchestration::recovery_budget::RecoveryAllowance;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DropSetManifestV3, PersistedRecordIdentity,
    PhysicalFreeSpaceMembershipBlock, PhysicalSegmentMembershipBlock,
    RecordSegmentPageManifestEntry,
};

use crate::orchestration::planning::manifest_entry_budget::{ManifestEntryBudget, ViewEntryCap};
use crate::progression::RecoverySelectedSourceInventory;

use super::walk_failure::{Verdict, WalkFailure};

#[cfg(test)]
#[path = "resident_tests.rs"]
mod tests;

pub(super) fn manifest_retained_bytes(manifest: &DropSetManifestV3) -> Option<u64> {
    (manifest.dropped().len() as u64)
        .checked_mul(std::mem::size_of::<PersistedRecordIdentity>() as u64)?
        .checked_add(std::mem::size_of::<DropSetManifestV3>() as u64)
}

pub(super) fn inventory_resident_bytes(
    inventory: &RecoverySelectedSourceInventory,
    routes: &[CurrentPhysicalRecordPlacement],
) -> Option<u64> {
    let mut bytes = std::mem::size_of::<RecoverySelectedSourceInventory>() as u64;
    let count = |len: usize, item: usize| (len as u64).checked_mul(item as u64);
    bytes = bytes
        .checked_add(count(
            routes.len(),
            std::mem::size_of::<CurrentPhysicalRecordPlacement>(),
        )?)?
        .checked_add(count(
            inventory.free_entries.len(),
            std::mem::size_of::<worth_store_physical_format::RecordFreeSpaceManifestEntry>(),
        )?)?
        .checked_add(count(
            inventory.source_artifacts.len(),
            std::mem::size_of::<worth_store_physical_format::RecordArtifactFile>(),
        )?)?;
    let node_overhead = 8 * std::mem::size_of::<usize>();
    bytes = bytes
        .checked_add(count(
            inventory.segment_pages.len(),
            std::mem::size_of::<(u64, u64)>()
                + std::mem::size_of::<crate::progression::RecoverySelectedSegmentPage>()
                + node_overhead,
        )?)?
        .checked_add(count(
            inventory.segment_topology.len(),
            std::mem::size_of::<(u64, u64)>()
                + std::mem::size_of::<PhysicalSegmentMembershipBlock>()
                + node_overhead,
        )?)?
        .checked_add(count(
            inventory.free_topology.len(),
            std::mem::size_of::<(u64, u64)>()
                + std::mem::size_of::<PhysicalFreeSpaceMembershipBlock>()
                + node_overhead,
        )?)?;
    for block in inventory.segment_topology.values() {
        let heap = match block {
            PhysicalSegmentMembershipBlock::Leaf { entries, .. } => count(
                entries.capacity(),
                std::mem::size_of::<RecordSegmentPageManifestEntry>(),
            )?,
            PhysicalSegmentMembershipBlock::Branch { children, .. } => count(
                children.capacity(),
                children.first().map_or(0, std::mem::size_of_val),
            )?,
        };
        bytes = bytes.checked_add(heap)?;
    }
    for block in inventory.free_topology.values() {
        let heap = match block {
            PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } => count(
                entries.capacity(),
                entries.first().map_or(0, std::mem::size_of_val),
            )?,
            PhysicalFreeSpaceMembershipBlock::Branch { children, .. } => count(
                children.capacity(),
                children.first().map_or(0, std::mem::size_of_val),
            )?,
        };
        bytes = bytes.checked_add(heap)?;
    }
    Some(bytes)
}

/// One view holds `count` entries `width` bytes wide: every entry recovery
/// admits, and `available` of the walk's `staging`.
fn view_fits(
    budget: &mut ManifestEntryBudget,
    count: u64,
    width: u64,
    available: u64,
    staging: RecoveryAllowance,
) -> Result<(), WalkFailure> {
    let view = ViewEntryCap::of(budget);
    if count > view.admitted() {
        return Err(budget.view_refused(view.refuse(count)).into());
    }
    let bytes = count.checked_mul(width).ok_or(WalkFailure::CountOverflow)?;
    if bytes > available {
        return Err(WalkFailure::past_scratch(bytes, available, staging));
    }
    Ok(())
}

/// A reserved backing of `capacity` entries `width` bytes wide fits
/// `available` of the walk's `staging`.
fn backing_fits(
    capacity: usize,
    width: u64,
    available: u64,
    staging: RecoveryAllowance,
) -> Result<(), WalkFailure> {
    let bytes = (capacity as u64)
        .checked_mul(width)
        .ok_or(WalkFailure::CountOverflow)?;
    if bytes > available {
        return Err(WalkFailure::past_scratch(bytes, available, staging));
    }
    Ok(())
}

fn segment_entries_bounded(
    inventory: &RecoverySelectedSourceInventory,
    budget: &mut ManifestEntryBudget,
    available_bytes: u64,
    staging: RecoveryAllowance,
) -> Result<Vec<RecordSegmentPageManifestEntry>, WalkFailure> {
    let width = std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64;
    let count = inventory.segment_pages.len() as u64;
    view_fits(budget, count, width, available_bytes, staging)?;
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(inventory.segment_pages.len())
        .proven()?;
    backing_fits(entries.capacity(), width, available_bytes, staging)?;
    entries.extend(inventory.segment_pages.values().map(|page| page.entry));
    Ok(entries)
}

pub(super) fn segment_pair_bounded(
    source: &RecoverySelectedSourceInventory,
    result: &RecoverySelectedSourceInventory,
    budget: &mut ManifestEntryBudget,
    available_bytes: u64,
    staging: RecoveryAllowance,
) -> Result<
    (
        Vec<RecordSegmentPageManifestEntry>,
        Vec<RecordSegmentPageManifestEntry>,
        u64,
    ),
    WalkFailure,
> {
    let source_segments = segment_entries_bounded(source, budget, available_bytes, staging)?;
    let source_bytes = (source_segments.capacity() as u64)
        .checked_mul(std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64)
        .proven()?;
    let result_segments = segment_entries_bounded(
        result,
        budget,
        WalkFailure::take(staging, available_bytes, source_bytes)?,
        staging,
    )?;
    let scratch = (source_segments.capacity() as u64)
        .checked_add(result_segments.capacity() as u64)
        .and_then(|count| {
            count.checked_mul(std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64)
        })
        .proven()?;
    Ok((source_segments, result_segments, scratch))
}

pub(super) fn dropped_bounded(
    manifest: &[PersistedRecordIdentity],
    derived: &[PersistedRecordIdentity],
    budget: &mut ManifestEntryBudget,
    available_bytes: u64,
    staging: RecoveryAllowance,
) -> Result<Vec<PersistedRecordIdentity>, WalkFailure> {
    let width = std::mem::size_of::<PersistedRecordIdentity>() as u64;
    let count = manifest
        .len()
        .checked_add(derived.len())
        .ok_or(WalkFailure::CountOverflow)?;
    view_fits(budget, count as u64, width, available_bytes, staging)?;
    let mut dropped = Vec::new();
    dropped.try_reserve_exact(count).proven()?;
    backing_fits(dropped.capacity(), width, available_bytes, staging)?;
    dropped.extend_from_slice(manifest);
    dropped.extend_from_slice(derived);
    dropped.sort_unstable();
    if dropped.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(WalkFailure::Unverified);
    }
    Ok(dropped)
}
