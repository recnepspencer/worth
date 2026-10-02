//! Actual selected V3 controls for a completed ordered edge. The manifest
//! stays in its native-funded frame buffer until the Store head fold borrows
//! its allocation-free decoded view.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV3, BlobRecordV1, CurrentPhysicalRecordPlacement,
    DropSetManifestV3View, OriginalDropReservedV1, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadControlIdentityV1,
};
use worth_store_recovery_physics::{
    VerifiedOrderedPendingWalReleaseBatch, VerifiedReleasedRootEdge,
};

use super::{verify_batch_binding, verify_source_routes};
use crate::physical_runtime::recovery_construction::selected_rejoin::{
    completed_history::{native_storage::HistoricalWalkStorage, ObservedCompletedHeadControls},
    control_frames::{
        read_extent_with_storage, SelectedArtifactSlice, SelectedControlMediaFingerprint,
    },
    pending_wal_release::delta,
    SelectedMediaRejoinDenial as Denial, MAX_CONTROL_FRAME_BYTES,
};

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct ObservedReleasedControls
{
    fingerprint: SelectedControlMediaFingerprint,
    descriptor: BlobReclaimDescriptorV3,
    reservation: OriginalDropReservedV1,
    manifest_bytes: Vec<u8>,
    frames: ReleaseCustodyHeadControlIdentityV1,
}

impl ObservedReleasedControls {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn head_controls(
        &self,
    ) -> Result<ObservedCompletedHeadControls<'_>, Denial> {
        let manifest_view = DropSetManifestV3View::decode(&self.manifest_bytes)
            .map_err(|_| Denial::ControlFrame)?;
        Ok(ObservedCompletedHeadControls {
            frames: self.frames,
            descriptor: &self.descriptor,
            reservation: &self.reservation,
            manifest_view,
        })
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn into_fingerprint(
        self,
        storage: &mut HistoricalWalkStorage<'_, '_>,
    ) -> Result<SelectedControlMediaFingerprint, Denial> {
        let Self {
            fingerprint,
            manifest_bytes,
            ..
        } = self;
        storage.discard_vec(manifest_bytes)?;
        Ok(fingerprint)
    }
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn verify_with_storage(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    source: &delta::Snapshot,
    result: &delta::Snapshot,
    edge: &VerifiedReleasedRootEdge,
    batch: &VerifiedOrderedPendingWalReleaseBatch,
    format: PhysicalRecordFormatDeclaration,
    storage: &mut HistoricalWalkStorage<'_, '_>,
) -> Result<ObservedReleasedControls, Denial> {
    let records = verify_batch_binding(discovery, edge, batch)?;
    let mut slices = storage.reserve_vec::<SelectedArtifactSlice>(0)?;
    let mut descriptor = None;
    let mut reservation = None;
    let mut manifest_bytes = None;
    let mut hashes = [[0_u8; 32]; 3];
    for (ordinal, frame) in [
        batch.descriptor_frame(),
        batch.reservation_frame(),
        batch.manifest_frame(),
    ]
    .into_iter()
    .enumerate()
    {
        if frame.candidate_root_frame_sha256() != edge.result_root_frame_sha256()
            || frame.bytes().len() as u64 > MAX_CONTROL_FRAME_BYTES
        {
            return Err(Denial::ControlFrame);
        }
        let route = result
            .routes
            .binary_search_by_key(&frame.record(), |route| route.record())
            .ok()
            .map(|index| result.routes[index])
            .ok_or(Denial::ControlFrame)?;
        let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
            return Err(Denial::UnsupportedSelectedPlacement);
        };
        let (bytes, _) = read_extent_with_storage(discovery, format, extent, &mut slices, storage)?;
        let hash: [u8; 32] = Sha256::digest(&bytes).into();
        if bytes != frame.bytes() || hash != frame.payload_sha256() {
            return Err(Denial::ControlFrame);
        }
        hashes[ordinal] = hash;
        match ordinal {
            0 => {
                let BlobRecordV1::ReclaimDescriptorV3(actual) =
                    decode_blob_record(&bytes).map_err(|_| Denial::ControlFrame)?
                else {
                    return Err(Denial::ControlFrame);
                };
                if actual != batch.descriptor() || actual.canonical_frame_sha256() != hash {
                    return Err(Denial::ControlFrame);
                }
                descriptor = Some(actual);
                storage.discard_vec(bytes)?;
            }
            1 => {
                let BlobRecordV1::OriginalDropReserved(actual) =
                    decode_blob_record(&bytes).map_err(|_| Denial::ControlFrame)?
                else {
                    return Err(Denial::ControlFrame);
                };
                if actual != batch.reservation() || actual.canonical_frame_sha256() != hash {
                    return Err(Denial::ControlFrame);
                }
                reservation = Some(actual);
                storage.discard_vec(bytes)?;
            }
            _ => {
                let view =
                    DropSetManifestV3View::decode(&bytes).map_err(|_| Denial::ControlFrame)?;
                if view.canonical_frame_sha256() != hash {
                    return Err(Denial::ControlFrame);
                }
                manifest_bytes = Some(bytes);
            }
        }
    }
    verify_source_routes(source, result, records)?;
    let frames = ReleaseCustodyHeadControlIdentityV1::new(
        records[0], hashes[0], records[1], hashes[1], records[2], hashes[2],
    )
    .map_err(|_| Denial::ControlFrame)?;
    Ok(ObservedReleasedControls {
        fingerprint: SelectedControlMediaFingerprint::observed(slices),
        descriptor: descriptor.ok_or(Denial::ControlFrame)?,
        reservation: reservation.ok_or(Denial::ControlFrame)?,
        manifest_bytes: manifest_bytes.ok_or(Denial::ControlFrame)?,
        frames,
    })
}
