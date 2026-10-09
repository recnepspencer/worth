//! V2 root/free observations and their selected-media fingerprint slices.

use worth_store_physical_backend::{
    ArtifactCeiling, BoundedRecoveryFilesystemDiscovery, PageAddress, ReadGrant, UnchargedRead,
};
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    RecordArtifactFile,
};

use super::super::{
    control_frames::SelectedArtifactSlice,
    resident::{discovery_allocation_denial, StoreRejoinResidentLedger},
    root_checkpoint::{ObservedCheckpointSourceRoot, ObservedRootCheckpoint},
    tier, SelectedMediaRejoinDenial as Denial,
};

pub(super) fn read_header(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<(DurableFreeSpaceManifestHeader, Vec<u8>), Denial> {
    let bytes = discovery
        .read_with_allocator(
            ArtifactCeiling::page(
                format,
                PageAddress::FreeSpaceManifest {
                    generation: root.generation(),
                },
            ),
            ReadGrant::ceiling_only(),
            |count| resident.reserve_bytes(count),
        )
        .observed()
        .map_err(discovery_allocation_denial)?
        .into_bytes()
        .ok_or(Denial::MissingFrame)?;
    let free = tier::selection::validate_selected_free_header(
        &bytes,
        discovery.store_identity(),
        format,
        root,
    )?;
    Ok((free, bytes))
}

pub(super) fn fingerprint_slices(
    selected: &ObservedRootCheckpoint,
    selected_free: &[u8],
    source: &ObservedCheckpointSourceRoot,
    source_free: Option<&[u8]>,
    resident: &mut StoreRejoinResidentLedger,
) -> Result<Vec<SelectedArtifactSlice>, Denial> {
    let mut slices = resident
        .reserve_vec(if source_free.is_some() { 4 } else { 2 })
        .map_err(Denial::Resident)?;
    slices.push(root_slice(selected.root(), selected.root_bytes())?);
    slices.push(free_slice(selected.root(), selected_free)?);
    if let Some(bytes) = source_free {
        slices.push(root_slice(source.root(), source.bytes())?);
        slices.push(free_slice(source.root(), bytes)?);
    }
    Ok(slices)
}

fn root_slice(
    root: &DurablePhysicalRootManifest,
    bytes: &[u8],
) -> Result<SelectedArtifactSlice, Denial> {
    SelectedArtifactSlice::observed(
        RecordArtifactFile::RootManifest {
            generation: root.generation(),
        },
        0,
        bytes,
        true,
    )
    .ok_or(Denial::BoundExceeded)
}

fn free_slice(
    root: &DurablePhysicalRootManifest,
    bytes: &[u8],
) -> Result<SelectedArtifactSlice, Denial> {
    SelectedArtifactSlice::observed(
        RecordArtifactFile::FreeSpaceManifest {
            generation: root.generation(),
        },
        0,
        bytes,
        true,
    )
    .ok_or(Denial::BoundExceeded)
}
