//! Streaming, integrity-validated free and segment membership rewalk. Route
//! leaves are streamed by `tier::routes`; no whole inventory map is retained.

use std::collections::BTreeSet;

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
    let mut slices = Vec::new();
    observe_segments(
        discovery,
        root,
        format,
        transcript,
        &mut slices,
        None,
        u64::MAX,
    )?;
    observe_free(discovery, free, format, transcript, &mut slices, None)?;
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
    let mut slices = Vec::new();
    let mut segments = Vec::new();
    let mut free_entries = Vec::new();
    free_entries
        .try_reserve_exact(usize::try_from(free.entry_count()).map_err(|_| Denial::BoundExceeded)?)
        .map_err(|_| Denial::BoundExceeded)?;
    observe_segments(
        discovery,
        root,
        format,
        transcript,
        &mut slices,
        Some(&mut segments),
        maximum_segments,
    )?;
    observe_free(
        discovery,
        free,
        format,
        transcript,
        &mut slices,
        Some(&mut free_entries),
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
) -> Result<(), Denial> {
    let tree = PhysicalTreeIdentity::new(root.tree_identity()).ok_or(Denial::RootBinding)?;
    let mut stack = root.segment_root().into_iter().collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    while let Some(reference) = stack.pop() {
        if seen.len() >= MAX_BLOCKS || !seen.insert((reference.generation(), reference.block())) {
            return Err(Denial::BoundExceeded);
        }
        let bytes = discovery
            .read_segment_membership_block(
                reference.generation(),
                reference.block(),
                u64::from(format.page_size().bytes()),
            )
            .map_err(Denial::Discovery)?
            .into_bytes()
            .ok_or(Denial::MissingFrame)?;
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
            UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
            scope,
        );
        let SegmentMembershipBlockIntegrityValidation::Intact(block) = validation else {
            return Err(Denial::RoutingFrame);
        };
        if slices.len() >= MAX_BLOCKS {
            return Err(Denial::BoundExceeded);
        }
        slices.push(
            SelectedArtifactSlice::observed(
                RecordArtifactFile::SegmentMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                0,
                &bytes,
                true,
            )
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
                    snapshot
                        .try_reserve_exact(1)
                        .map_err(|_| Denial::BoundExceeded)?;
                    snapshot.push(*entry);
                }
            }
        } else {
            let children = block.children().ok_or(Denial::RoutingFrame)?;
            for child in children.iter().rev() {
                if child.level().checked_add(1) != Some(reference.level())
                    || child.generation() > reference.generation()
                {
                    return Err(Denial::RoutingFrame);
                }
                stack.push(*child);
            }
        }
    }
    Ok(())
}

fn observe_free(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    transcript: &mut PhysicalInventoryTranscriptBuilderV1,
    slices: &mut Vec<SelectedArtifactSlice>,
    mut snapshot: Option<&mut Vec<RecordFreeSpaceManifestEntry>>,
) -> Result<(), Denial> {
    let tree = PhysicalTreeIdentity::new(free.tree_identity()).ok_or(Denial::RootBinding)?;
    let mut stack = free.root().into_iter().collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    while let Some(reference) = stack.pop() {
        if seen.len() >= MAX_BLOCKS || !seen.insert((reference.generation(), reference.block())) {
            return Err(Denial::BoundExceeded);
        }
        let bytes = discovery
            .read_free_space_membership_block(
                reference.generation(),
                reference.block(),
                u64::from(format.page_size().bytes()),
            )
            .map_err(Denial::Discovery)?
            .into_bytes()
            .ok_or(Denial::MissingFrame)?;
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
            UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
            scope,
        );
        let FreeSpaceMembershipBlockIntegrityValidation::Intact(block) = validation else {
            return Err(Denial::RoutingFrame);
        };
        if slices.len() >= MAX_BLOCKS {
            return Err(Denial::BoundExceeded);
        }
        slices.push(
            SelectedArtifactSlice::observed(
                RecordArtifactFile::FreeSpaceMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                0,
                &bytes,
                true,
            )
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
                    snapshot.push(*entry);
                }
            }
        } else {
            let children = block.children().ok_or(Denial::RoutingFrame)?;
            for child in children.iter().rev() {
                if child.level().checked_add(1) != Some(reference.level())
                    || child.generation() > reference.generation()
                {
                    return Err(Denial::RoutingFrame);
                }
                stack.push(*child);
            }
        }
    }
    Ok(())
}
