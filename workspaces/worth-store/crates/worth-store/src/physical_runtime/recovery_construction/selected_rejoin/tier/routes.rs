//! Bounded, C.9-scoped reread of every selected route before tier custody.

use std::collections::{BTreeMap, BTreeSet};

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    arena_tier_at_epoch, BlobRecordKind, CurrentPhysicalRecordPlacement,
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    PersistedBlobSemanticRecordBinding, PersistedRecordIdentity,
    PhysicalInventoryTranscriptBuilderV1, PhysicalRecordFormatDeclaration, PhysicalTierClass,
    PhysicalTreeIdentity, RecordArtifactFile, RootRoutingBlockScopeIdentity,
    SelectedRecordContentClass,
};
use worth_store_physical_integrity::{
    validate_root_routing_block, PhysicalArtifactScope, PhysicalByteRange,
    RootRoutingBlockIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::super::control_frames::{SelectedArtifactSlice, SelectedControlMediaFingerprint};
use super::super::SelectedMediaRejoinDenial as Denial;
use super::super::MAX_DISCOVERY_BYTES;
use super::no_release_controls;
mod resident;
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) use resident::{
    verify_snapshot_with_storage, verify_with_resident, verify_with_storage,
    ResidentRouteProvenance, RouteWalkStorage, VisitedNodes,
};

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) const MAX_BLOCKS: usize =
    65_536;

#[derive(Debug, PartialEq, Eq)]
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct NoReleaseControlProvenance
{
    proven_drops: BTreeMap<PersistedRecordIdentity, ([u8; 32], u64)>,
    selected_routes: BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    slices: Vec<SelectedArtifactSlice>,
}

impl NoReleaseControlProvenance {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn retained_memory_bytes(
        &self,
    ) -> u64 {
        let map_entry_bytes = std::mem::size_of::<PersistedRecordIdentity>()
            + std::mem::size_of::<CurrentPhysicalRecordPlacement>()
            + 8 * std::mem::size_of::<usize>();
        let drops_entry_bytes = std::mem::size_of::<PersistedRecordIdentity>()
            + std::mem::size_of::<([u8; 32], u64)>()
            + 8 * std::mem::size_of::<usize>();
        (std::mem::size_of::<Self>() as u64)
            .saturating_add(
                (self.proven_drops.len() as u64).saturating_mul(4 * drops_entry_bytes as u64),
            )
            .saturating_add(
                (self.selected_routes.len() as u64).saturating_mul(4 * map_entry_bytes as u64),
            )
            .saturating_add(
                (self.slices.capacity() as u64)
                    .saturating_mul(4 * std::mem::size_of::<SelectedArtifactSlice>() as u64),
            )
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn admits_drop(
        &self,
        binding: PersistedBlobSemanticRecordBinding,
    ) -> bool {
        self.proven_drops.get(&binding.record())
            == Some(&(
                binding.record_payload_sha256(),
                binding.candidate_root_generation(),
            ))
    }

    pub(in crate::physical_runtime) fn into_media_fingerprint(
        self,
    ) -> SelectedControlMediaFingerprint {
        SelectedControlMediaFingerprint::observed(self.slices)
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn selected_routes(
        &self,
    ) -> &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement> {
        &self.selected_routes
    }
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    no_release_checkpoint_sequence: Option<u64>,
) -> Result<NoReleaseControlProvenance, Denial> {
    verify_inner(
        discovery,
        root,
        free,
        format,
        no_release_checkpoint_sequence,
        None,
        None,
    )
}

/// The pending-WAL rejoin streams every route into a versioned semantic
/// transcript while retaining only the control routes needed by its roster.
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_transcribed(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    transcript: &mut PhysicalInventoryTranscriptBuilderV1,
) -> Result<NoReleaseControlProvenance, Denial> {
    verify_inner(discovery, root, free, format, None, Some(transcript), None)
}

/// Temporary full canonical route inventory for an independent V3 delta
/// check. The caller charges its retained capacity against recovery memory.
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_snapshot(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    transcript: &mut PhysicalInventoryTranscriptBuilderV1,
) -> Result<
    (
        NoReleaseControlProvenance,
        Vec<CurrentPhysicalRecordPlacement>,
    ),
    Denial,
> {
    let count = usize::try_from(root.record_count()).map_err(|_| Denial::BoundExceeded)?;
    let mut placements = Vec::new();
    placements
        .try_reserve_exact(count)
        .map_err(|_| Denial::BoundExceeded)?;
    let provenance = verify_inner(
        discovery,
        root,
        free,
        format,
        None,
        Some(transcript),
        Some(&mut placements),
    )?;
    Ok((provenance, placements))
}

fn verify_inner(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    no_release_checkpoint_sequence: Option<u64>,
    mut transcript: Option<&mut PhysicalInventoryTranscriptBuilderV1>,
    mut snapshot: Option<&mut Vec<CurrentPhysicalRecordPlacement>>,
) -> Result<NoReleaseControlProvenance, Denial> {
    // Four charged placement widths per retained entry leave room for the
    // opposite inventory, checked-delta scratch, and compact frame slices.
    const MAX_ENTRIES: u64 =
        MAX_DISCOVERY_BYTES / (4 * std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64);
    if root.record_count() > MAX_ENTRIES {
        return Err(Denial::BoundExceeded);
    }
    let mut stack = root.routing_root().into_iter().collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    let mut records = BTreeSet::new();
    let mut selected_routes = BTreeMap::new();
    let mut slices = Vec::new();
    let tree = PhysicalTreeIdentity::new(root.tree_identity()).ok_or(Denial::RootBinding)?;
    let mut entry_count = 0_u64;
    while let Some(reference) = stack.pop() {
        if seen.len() >= MAX_BLOCKS || !seen.insert((reference.generation(), reference.block())) {
            return Err(Denial::RoutingFrame);
        }
        let frame = discovery
            .read_root_routing_block(
                reference.generation(),
                reference.block(),
                u64::from(format.page_size().bytes()),
            )
            .map_err(Denial::Discovery)?;
        let bytes = frame.bytes().ok_or(Denial::MissingRoute)?;
        let range =
            PhysicalByteRange::new(0, bytes.len() as u64).map_err(|_| Denial::RoutingFrame)?;
        let scope = PhysicalArtifactScope::root_routing_block(
            discovery.store_identity(),
            format,
            RootRoutingBlockScopeIdentity::new(tree, reference),
            range,
        );
        let (validation, _) = validate_root_routing_block(
            UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            scope,
        );
        let RootRoutingBlockIntegrityValidation::Intact(block) = validation else {
            return Err(Denial::RoutingFrame);
        };
        slices.push(
            SelectedArtifactSlice::observed(
                RecordArtifactFile::RootRoutingBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                0,
                bytes,
                true,
            )
            .ok_or(Denial::RoutingFrame)?,
        );
        if let Some(entries) = block.entries() {
            validate_leaf_count(entries.len(), root, entry_count)?;
            for placement in entries {
                if !reference.contains(placement.record()) {
                    return Err(Denial::RoutingFrame);
                }
                if let Some(transcript) = transcript.as_deref_mut() {
                    transcript
                        .include_route(*placement)
                        .map_err(|_| Denial::RoutingFrame)?;
                } else if !records.insert(placement.record()) {
                    return Err(Denial::RoutingFrame);
                }
                if let Some(snapshot) = snapshot.as_deref_mut() {
                    snapshot.push(*placement);
                }
                if selected_control(*placement, no_release_checkpoint_sequence.is_some()) {
                    selected_routes.insert(placement.record(), *placement);
                }
                validate_route(*placement, free)?;
                entry_count = entry_count.checked_add(1).ok_or(Denial::BoundExceeded)?;
            }
        } else {
            let children = block.children().ok_or(Denial::RoutingFrame)?;
            if children.len() > usize::from(root.node_capacity()) {
                return Err(Denial::RoutingFrame);
            }
            validate_stack_growth(stack.len(), children.len())?;
            for child in children.iter().rev() {
                validate_child(*child, reference)?;
                stack.push(*child);
            }
        }
    }
    if entry_count != root.record_count() {
        return Err(Denial::RoutingFrame);
    }
    let proven_drops = if let Some(checkpoint_sequence) = no_release_checkpoint_sequence {
        no_release_controls::verify(
            discovery,
            &selected_routes,
            format,
            root.generation(),
            checkpoint_sequence,
            &mut slices,
        )?
    } else {
        BTreeMap::new()
    };
    Ok(NoReleaseControlProvenance {
        proven_drops,
        selected_routes,
        slices,
    })
}

fn validate_leaf_count(
    len: usize,
    root: &DurablePhysicalRootManifest,
    previous: u64,
) -> Result<(), Denial> {
    if len > usize::from(root.node_capacity()) {
        return Err(Denial::RoutingFrame);
    }
    let leaf = u64::try_from(len).map_err(|_| Denial::BoundExceeded)?;
    if previous.checked_add(leaf).ok_or(Denial::BoundExceeded)? > root.record_count() {
        return Err(Denial::BoundExceeded);
    }
    Ok(())
}

fn validate_child(
    child: worth_store_physical_format::ManifestBlockReference,
    parent: worth_store_physical_format::ManifestBlockReference,
) -> Result<(), Denial> {
    if child.level().checked_add(1) != Some(parent.level())
        || child.generation() > parent.generation()
    {
        Err(Denial::RoutingFrame)
    } else {
        Ok(())
    }
}

fn validate_stack_growth(existing: usize, additional: usize) -> Result<(), Denial> {
    if existing
        .checked_add(additional)
        .ok_or(Denial::BoundExceeded)?
        > MAX_BLOCKS
    {
        Err(Denial::BoundExceeded)
    } else {
        Ok(())
    }
}

fn selected_control(placement: CurrentPhysicalRecordPlacement, all: bool) -> bool {
    all || matches!(
        placement.content_class(),
        SelectedRecordContentClass::UnknownLegacy
            | SelectedRecordContentClass::Blob(
                BlobRecordKind::DropSetManifest
                    | BlobRecordKind::DropSetManifestV2
                    | BlobRecordKind::ReclaimDescriptor
                    | BlobRecordKind::ReclaimDescriptorV2
                    | BlobRecordKind::SessionDeclared
                    | BlobRecordKind::SessionAbandoned
                    | BlobRecordKind::GenerationPublished
                    | BlobRecordKind::DropSetManifestV3
                    | BlobRecordKind::ReclaimDescriptorV3
                    | BlobRecordKind::OriginalDropReserved
            )
    )
}

fn validate_route(
    placement: CurrentPhysicalRecordPlacement,
    free: &DurableFreeSpaceManifestHeader,
) -> Result<(), Denial> {
    match placement {
        CurrentPhysicalRecordPlacement::Inline(inline)
            if inline.tier_class() != PhysicalTierClass::Primary =>
        {
            Err(Denial::RoutingFrame)
        }
        CurrentPhysicalRecordPlacement::Extent(extent)
            if extent.arena_range().arena().get() >= free.next_arena()
                || extent.tier_class()
                    != arena_tier_at_epoch(
                        free.tier_epoch_start(),
                        extent.arena_range().arena(),
                    ) =>
        {
            Err(Denial::RoutingFrame)
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
#[path = "routes/tests.rs"]
mod count_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_release_tail_requires_exact_selected_failed_ingest_descriptor_binding() {
        let record = PersistedRecordIdentity::new([7; 16], 3).unwrap();
        let other = PersistedRecordIdentity::new([7; 16], 4).unwrap();
        let proof = NoReleaseControlProvenance {
            proven_drops: BTreeMap::from([(record, ([8; 32], 12))]),
            selected_routes: BTreeMap::new(),
            slices: Vec::new(),
        };
        for (identity, digest, generation, admitted) in [
            (record, [8; 32], 12, true),
            (other, [8; 32], 12, false),
            (record, [9; 32], 12, false),
            (record, [8; 32], 13, false),
        ] {
            let binding =
                PersistedBlobSemanticRecordBinding::new(identity, digest, generation).unwrap();
            assert_eq!(proof.admits_drop(binding), admitted);
        }
    }
}
