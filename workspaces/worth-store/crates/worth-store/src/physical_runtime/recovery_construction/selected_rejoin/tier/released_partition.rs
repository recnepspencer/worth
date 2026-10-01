//! Partition selected reclaim controls by their actual manifest source family.

use std::collections::BTreeMap;

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, CurrentPhysicalRecordPlacement,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration, SelectedRecordContentClass,
};

use super::super::control_frames::SelectedArtifactSlice;
use super::super::{SelectedMediaRejoinDenial as Denial, MAX_CONTROL_FRAME_BYTES};
use super::no_release_frame;

#[derive(PartialEq, Eq)]
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) struct PartitionedControlRoutes
{
    released: BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    failed_ingest: BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    reservation_slices: Vec<SelectedArtifactSlice>,
}

impl PartitionedControlRoutes {
    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn released(
        &self,
    ) -> &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement> {
        &self.released
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn failed_ingest(
        &self,
    ) -> &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement> {
        &self.failed_ingest
    }

    pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn reservation_slices(
        &self,
    ) -> &[SelectedArtifactSlice] {
        &self.reservation_slices
    }
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn partition(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
) -> Result<PartitionedControlRoutes, Denial> {
    partition_with_slice_limit(discovery, routes, format, usize::MAX)
}

pub(in crate::physical_runtime::recovery_construction::selected_rejoin) fn partition_with_slice_limit(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    format: PhysicalRecordFormatDeclaration,
    max_slices: usize,
) -> Result<PartitionedControlRoutes, Denial> {
    let mut released = BTreeMap::new();
    let mut failed_ingest = BTreeMap::new();
    let mut reservations = Vec::new();
    for (&record, &route) in routes {
        match route.content_class() {
            SelectedRecordContentClass::Blob(
                BlobRecordKind::DropSetManifestV3 | BlobRecordKind::ReclaimDescriptorV3,
            ) => {
                released.insert(record, route);
            }
            SelectedRecordContentClass::Blob(
                BlobRecordKind::DropSetManifest
                | BlobRecordKind::DropSetManifestV2
                | BlobRecordKind::ReclaimDescriptor
                | BlobRecordKind::ReclaimDescriptorV2
                | BlobRecordKind::GenerationPublished
                | BlobRecordKind::SessionDeclared
                | BlobRecordKind::SessionAbandoned,
            ) => {
                failed_ingest.insert(record, route);
            }
            SelectedRecordContentClass::Blob(BlobRecordKind::OriginalDropReserved) => {
                reservations.push((record, route));
            }
            SelectedRecordContentClass::UnknownLegacy => {
                return Err(Denial::CertificateRoster);
            }
            _ => {}
        }
    }
    let mut reservation_slices = Vec::new();
    for (record, route) in reservations {
        let CurrentPhysicalRecordPlacement::Extent(extent) = route else {
            return Err(Denial::UnsupportedSelectedPlacement);
        };
        let bytes = no_release_frame::read_with_slice_limit(
            discovery,
            format,
            extent,
            MAX_CONTROL_FRAME_BYTES,
            &mut reservation_slices,
            max_slices,
        )?;
        let BlobRecordV1::OriginalDropReserved(reservation) =
            decode_blob_record(&bytes).map_err(|_| Denial::ControlFrame)?
        else {
            return Err(Denial::ControlFrame);
        };
        if reservation.encode() != bytes {
            return Err(Denial::ControlFrame);
        }
        let manifest = routes
            .get(&reservation.manifest_record())
            .ok_or(Denial::CertificateRoster)?;
        match manifest.content_class() {
            SelectedRecordContentClass::Blob(BlobRecordKind::DropSetManifestV3) => {
                released.insert(record, route);
            }
            SelectedRecordContentClass::Blob(
                BlobRecordKind::DropSetManifest | BlobRecordKind::DropSetManifestV2,
            ) => {
                failed_ingest.insert(record, route);
            }
            _ => return Err(Denial::CertificateRoster),
        }
    }
    Ok(PartitionedControlRoutes {
        released,
        failed_ingest,
        reservation_slices,
    })
}
