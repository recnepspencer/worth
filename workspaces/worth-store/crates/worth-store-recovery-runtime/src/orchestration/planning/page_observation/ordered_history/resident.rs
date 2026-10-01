//! Peak retained media observations for one ordered root-step comparison.

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DropSetManifestV3, PersistedRecordIdentity,
    PhysicalFreeSpaceMembershipBlock, PhysicalSegmentMembershipBlock,
    RecordSegmentPageManifestEntry,
};

use crate::progression::RecoverySelectedSourceInventory;

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

fn segment_entries_bounded(
    inventory: &RecoverySelectedSourceInventory,
    maximum_entries: u64,
    available_bytes: u64,
) -> Option<Vec<RecordSegmentPageManifestEntry>> {
    let count = u64::try_from(inventory.segment_pages.len()).ok()?;
    let bytes = count.checked_mul(std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64)?;
    if count > maximum_entries || bytes > available_bytes {
        return None;
    }
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(inventory.segment_pages.len())
        .ok()?;
    if (entries.capacity() as u64)
        .checked_mul(std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64)?
        > available_bytes
    {
        return None;
    }
    entries.extend(inventory.segment_pages.values().map(|page| page.entry));
    Some(entries)
}

pub(super) fn segment_pair_bounded(
    source: &RecoverySelectedSourceInventory,
    result: &RecoverySelectedSourceInventory,
    maximum_entries: u64,
    available_bytes: u64,
) -> Option<(
    Vec<RecordSegmentPageManifestEntry>,
    Vec<RecordSegmentPageManifestEntry>,
    u64,
)> {
    let source_segments = segment_entries_bounded(source, maximum_entries, available_bytes)?;
    let source_bytes = (source_segments.capacity() as u64)
        .checked_mul(std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64)?;
    let result_segments = segment_entries_bounded(
        result,
        maximum_entries,
        available_bytes.checked_sub(source_bytes)?,
    )?;
    let scratch = (source_segments.capacity() as u64)
        .checked_add(result_segments.capacity() as u64)?
        .checked_mul(std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64)?;
    Some((source_segments, result_segments, scratch))
}

pub(super) fn dropped_bounded(
    manifest: &[PersistedRecordIdentity],
    derived: &[PersistedRecordIdentity],
    maximum_entries: u64,
    available_bytes: u64,
) -> Option<Vec<PersistedRecordIdentity>> {
    let count = manifest.len().checked_add(derived.len())?;
    let bytes = (count as u64).checked_mul(std::mem::size_of::<PersistedRecordIdentity>() as u64)?;
    if count as u64 > maximum_entries || bytes > available_bytes {
        return None;
    }
    let mut dropped = Vec::new();
    dropped.try_reserve_exact(count).ok()?;
    if (dropped.capacity() as u64)
        .checked_mul(std::mem::size_of::<PersistedRecordIdentity>() as u64)?
        > available_bytes
    {
        return None;
    }
    dropped.extend_from_slice(manifest);
    dropped.extend_from_slice(derived);
    dropped.sort_unstable();
    if dropped.windows(2).any(|pair| pair[0] == pair[1]) {
        return None;
    }
    Some(dropped)
}
