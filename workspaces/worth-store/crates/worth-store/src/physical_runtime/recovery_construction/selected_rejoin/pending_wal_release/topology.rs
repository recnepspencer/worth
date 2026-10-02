//! Streaming, integrity-validated free and segment membership rewalk. Route
//! leaves are streamed by `tier::routes`; no whole inventory map is retained.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    FreeSpaceMembershipBlockScopeIdentity, PhysicalInventoryTranscriptBuilderV1,
    PhysicalInventoryTranscriptV1, PhysicalRecordFormatDeclaration, PhysicalTreeIdentity,
    RecordArtifactFile, RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
    SegmentMembershipBlockScopeIdentity,
};
use worth_store_physical_integrity::{
    validate_free_space_membership_block, validate_segment_membership_block,
    FreeSpaceMembershipBlockIntegrityValidation, PhysicalArtifactScope, PhysicalByteRange,
    SegmentMembershipBlockIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::super::tier::routes::{RouteWalkStorage, VisitedNodes};
use super::super::{
    control_frames::SelectedArtifactSlice, SelectedMediaRejoinDenial as Denial,
    MAX_DISCOVERY_ENTRIES,
};

// The admitted discovery caps total artifact reads at this count. At most one
// compact SHA/length slice is retained per visited block (not its page bytes).
// Both membership families share the same slice cap; routes have their own
// bounded slices under the same discovery charge.
const MAX_BLOCKS: usize = MAX_DISCOVERY_ENTRIES as usize;

pub(super) fn require_transcript(
    expected: PhysicalInventoryTranscriptV1,
    observed: PhysicalInventoryTranscriptV1,
) -> Result<(), Denial> {
    (expected == observed)
        .then_some(())
        .ok_or(Denial::RoutingFrame)
}

pub(super) fn observe_memberships(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    transcript: &mut PhysicalInventoryTranscriptBuilderV1,
) -> Result<Vec<SelectedArtifactSlice>, Denial> {
    let mut storage = ();
    let mut slices = storage.reserve_vec(0)?;
    observe_segments(
        discovery,
        root,
        format,
        transcript,
        &mut slices,
        None,
        u64::MAX,
        &mut storage,
    )?;
    observe_free(
        discovery,
        free,
        format,
        transcript,
        &mut slices,
        None,
        &mut storage,
    )?;
    Ok(slices)
}

pub(super) fn observe_membership_snapshot(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    transcript: &mut PhysicalInventoryTranscriptBuilderV1,
    maximum_segments: u64,
) -> Result<
    (
        Vec<RecordSegmentPageManifestEntry>,
        Vec<RecordFreeSpaceManifestEntry>,
        Vec<SelectedArtifactSlice>,
    ),
    Denial,
> {
    let mut storage = ();
    observe_membership_snapshot_inner(
        discovery,
        root,
        free,
        format,
        transcript,
        maximum_segments,
        &mut storage,
    )
}

pub(super) fn observe_membership_snapshot_with_storage<S: RouteWalkStorage>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    transcript: &mut PhysicalInventoryTranscriptBuilderV1,
    maximum_segments: u64,
    storage: &mut S,
) -> Result<
    (
        Vec<RecordSegmentPageManifestEntry>,
        Vec<RecordFreeSpaceManifestEntry>,
        Vec<SelectedArtifactSlice>,
    ),
    Denial,
> {
    observe_membership_snapshot_inner(
        discovery,
        root,
        free,
        format,
        transcript,
        maximum_segments,
        storage,
    )
}

fn observe_membership_snapshot_inner<S: RouteWalkStorage>(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    transcript: &mut PhysicalInventoryTranscriptBuilderV1,
    maximum_segments: u64,
    storage: &mut S,
) -> Result<
    (
        Vec<RecordSegmentPageManifestEntry>,
        Vec<RecordFreeSpaceManifestEntry>,
        Vec<SelectedArtifactSlice>,
    ),
    Denial,
> {
    let mut slices = storage.reserve_vec(0)?;
    let mut segments = storage.reserve_vec(0)?;
    let mut free_entries = storage
        .reserve_vec(usize::try_from(free.entry_count()).map_err(|_| Denial::BoundExceeded)?)?;
    observe_segments(
        discovery,
        root,
        format,
        transcript,
        &mut slices,
        Some(&mut segments),
        maximum_segments,
        storage,
    )?;
    observe_free(
        discovery,
        free,
        format,
        transcript,
        &mut slices,
        Some(&mut free_entries),
        storage,
    )?;
    Ok((segments, free_entries, slices))
}

#[cfg(test)]
#[path = "topology/tests.rs"]
mod tests;

fn observe_segments(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    transcript: &mut PhysicalInventoryTranscriptBuilderV1,
    slices: &mut Vec<SelectedArtifactSlice>,
    mut snapshot: Option<&mut Vec<RecordSegmentPageManifestEntry>>,
    maximum_segments: u64,
    storage: &mut impl RouteWalkStorage,
) -> Result<(), Denial> {
    let tree = PhysicalTreeIdentity::new(root.tree_identity()).ok_or(Denial::RootBinding)?;
    let mut stack = storage.reserve_vec(usize::from(root.segment_root().is_some()))?;
    if let Some(reference) = root.segment_root() {
        stack.push(reference);
    }
    let mut seen = VisitedNodes::default();
    while let Some(reference) = stack.pop() {
        if !seen.insert((reference.generation(), reference.block()), storage)? {
            return Err(Denial::BoundExceeded);
        }
        if slices.len() >= MAX_BLOCKS {
            return Err(Denial::BoundExceeded);
        }
        storage.grow_vec_geometrically(slices, 1)?;
        let artifact = RecordArtifactFile::SegmentMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let frame =
            storage.read_page(discovery, artifact, u64::from(format.page_size().bytes()))?;
        let bytes = frame.bytes().ok_or(Denial::MissingFrame)?;
        let identity = SegmentMembershipBlockScopeIdentity::new(tree, reference);
        let range =
            PhysicalByteRange::new(0, bytes.len() as u64).map_err(|_| Denial::RoutingFrame)?;
        let scope = PhysicalArtifactScope::segment_membership_block(
            discovery.store_identity(),
            format,
            identity,
            range,
        );
        let (validation, _) = validate_segment_membership_block(
            UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            scope,
        );
        let SegmentMembershipBlockIntegrityValidation::Intact(block) = validation else {
            return Err(Denial::RoutingFrame);
        };
        slices.push(
            SelectedArtifactSlice::observed(artifact, 0, bytes, true)
                .ok_or(Denial::BoundExceeded)?,
        );
        if let Some(entries) = block.entries() {
            for entry in entries {
                if !reference.contains(worth_store_physical_format::SegmentPageKey::from(*entry)) {
                    return Err(Denial::RoutingFrame);
                }
                transcript
                    .include_segment(*entry)
                    .map_err(|_| Denial::RoutingFrame)?;
                if let Some(snapshot) = snapshot.as_deref_mut() {
                    if snapshot.len() as u64 >= maximum_segments {
                        return Err(Denial::BoundExceeded);
                    }
                    storage.grow_vec_geometrically(snapshot, 1)?;
                    snapshot.push(*entry);
                }
            }
        } else {
            let children = block.children().ok_or(Denial::RoutingFrame)?;
            storage.grow_vec(&mut stack, children.len())?;
            for child in children.iter().rev() {
                if child.level().checked_add(1) != Some(reference.level())
                    || child.generation() > reference.generation()
                {
                    return Err(Denial::RoutingFrame);
                }
                stack.push(*child);
            }
        }
        drop(block);
        storage.discard_frame(frame)?;
    }
    storage.discard_vec(stack)?;
    seen.discard(storage)?;
    Ok(())
}

fn observe_free(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    transcript: &mut PhysicalInventoryTranscriptBuilderV1,
    slices: &mut Vec<SelectedArtifactSlice>,
    mut snapshot: Option<&mut Vec<RecordFreeSpaceManifestEntry>>,
    storage: &mut impl RouteWalkStorage,
) -> Result<(), Denial> {
    let tree = PhysicalTreeIdentity::new(free.tree_identity()).ok_or(Denial::RootBinding)?;
    let mut stack = storage.reserve_vec(usize::from(free.root().is_some()))?;
    if let Some(reference) = free.root() {
        stack.push(reference);
    }
    let mut seen = VisitedNodes::default();
    while let Some(reference) = stack.pop() {
        if !seen.insert((reference.generation(), reference.block()), storage)? {
            return Err(Denial::BoundExceeded);
        }
        if slices.len() >= MAX_BLOCKS {
            return Err(Denial::BoundExceeded);
        }
        storage.grow_vec_geometrically(slices, 1)?;
        let artifact = RecordArtifactFile::FreeSpaceMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let frame =
            storage.read_page(discovery, artifact, u64::from(format.page_size().bytes()))?;
        let bytes = frame.bytes().ok_or(Denial::MissingFrame)?;
        let identity = FreeSpaceMembershipBlockScopeIdentity::new(tree, reference);
        let range =
            PhysicalByteRange::new(0, bytes.len() as u64).map_err(|_| Denial::RoutingFrame)?;
        let scope = PhysicalArtifactScope::free_space_membership_block(
            discovery.store_identity(),
            format,
            identity,
            range,
        );
        let (validation, _) = validate_free_space_membership_block(
            UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            scope,
        );
        let FreeSpaceMembershipBlockIntegrityValidation::Intact(block) = validation else {
            return Err(Denial::RoutingFrame);
        };
        slices.push(
            SelectedArtifactSlice::observed(artifact, 0, bytes, true)
                .ok_or(Denial::BoundExceeded)?,
        );
        if let Some(entries) = block.entries() {
            for entry in entries {
                if !reference.contains(worth_store_physical_format::FreeSpaceKey::from(*entry)) {
                    return Err(Denial::RoutingFrame);
                }
                transcript
                    .include_free(*entry)
                    .map_err(|_| Denial::RoutingFrame)?;
                if let Some(snapshot) = snapshot.as_deref_mut() {
                    storage.grow_vec_geometrically(snapshot, 1)?;
                    snapshot.push(*entry);
                }
            }
        } else {
            let children = block.children().ok_or(Denial::RoutingFrame)?;
            storage.grow_vec(&mut stack, children.len())?;
            for child in children.iter().rev() {
                if child.level().checked_add(1) != Some(reference.level())
                    || child.generation() > reference.generation()
                {
                    return Err(Denial::RoutingFrame);
                }
                stack.push(*child);
            }
        }
        drop(block);
        storage.discard_frame(frame)?;
    }
    storage.discard_vec(stack)?;
    seen.discard(storage)?;
    Ok(())
}
