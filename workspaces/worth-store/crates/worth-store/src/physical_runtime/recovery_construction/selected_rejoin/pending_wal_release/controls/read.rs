//! Bounded read of one selected routed release control frame.

use super::super::selection::Selection;
use super::{
    tier, BoundedRecoveryFilesystemDiscovery, CurrentPhysicalRecordPlacement, Denial,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration, SelectedArtifactSlice,
    MAX_CONTROL_FRAME_BYTES,
};
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_physics::VerifiedPendingWalReleaseCustody;

pub(super) fn frame(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &std::collections::BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
    record: PersistedRecordIdentity,
    slices: &mut Vec<SelectedArtifactSlice>,
) -> Result<Vec<u8>, Denial> {
    let Some(CurrentPhysicalRecordPlacement::Extent(extent)) = routes.get(&record) else {
        return Err(Denial::UnsupportedSelectedPlacement);
    };
    tier::no_release_frame::read(discovery, format, *extent, MAX_CONTROL_FRAME_BYTES, slices)
}

pub(super) fn fingerprint_selection(
    slices: &mut Vec<SelectedArtifactSlice>,
    selected: &Selection,
    claim: &VerifiedPendingWalReleaseCustody,
) -> Result<(), Denial> {
    for (artifact, bytes) in [
        (
            RecordArtifactFile::CurrentRootSelector,
            selected.selector.as_slice(),
        ),
        (
            RecordArtifactFile::RootManifest {
                generation: selected.root.generation(),
            },
            selected.root_bytes.as_slice(),
        ),
        (
            RecordArtifactFile::FreeSpaceManifest {
                generation: selected.root.generation(),
            },
            selected.free_bytes.as_slice(),
        ),
        (
            RecordArtifactFile::RootManifest {
                generation: claim.source_root().generation(),
            },
            selected.source_bytes.as_slice(),
        ),
        (
            RecordArtifactFile::FreeSpaceManifest {
                generation: claim.source_root().generation(),
            },
            selected.source_free_bytes.as_slice(),
        ),
        (
            RecordArtifactFile::RootManifest {
                generation: claim.checkpoint().source().root().generation(),
            },
            selected.checkpoint_source_bytes.as_slice(),
        ),
        (
            RecordArtifactFile::FreeSpaceManifest {
                generation: claim.checkpoint().source().root().generation(),
            },
            selected.checkpoint_source_free_bytes.as_slice(),
        ),
    ] {
        slices.push(
            SelectedArtifactSlice::observed(artifact, 0, bytes, true)
                .ok_or(Denial::BoundExceeded)?,
        );
    }
    Ok(())
}
