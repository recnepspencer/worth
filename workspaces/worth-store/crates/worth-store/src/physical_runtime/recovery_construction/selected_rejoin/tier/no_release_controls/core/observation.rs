//! Ordered selected-control census and actual frame observation. Every Vec is
//! admitted through the caller's storage before its first inserted element.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobGenerationPublicationV1, BlobRecordKind, BlobRecordV1,
    CurrentPhysicalRecordPlacement, DropSetManifestV2View, OriginalDropReservedV1,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration, SelectedRecordContentClass,
};

use super::super::super::super::control_frames::SelectedArtifactSlice;
use super::super::super::super::{SelectedMediaRejoinDenial as Denial, MAX_CONTROL_FRAME_BYTES};
use super::super::semantic::{Descriptor, Manifest};
use super::super::{FailedIngestFrameStorage, FailedIngestRoutes};

struct ControlCensus {
    manifests: usize,
    descriptors: usize,
    reservations: usize,
    publications: usize,
}

impl ControlCensus {
    fn of<R: FailedIngestRoutes + ?Sized>(routes: &R) -> Result<Self, Denial> {
        let mut last_record = None;
        for route in routes.ordered() {
            if last_record.is_some_and(|last| last >= route.record()) {
                return Err(Denial::CertificateRoster);
            }
            last_record = Some(route.record());
        }
        for route in routes.ordered() {
            match route.content_class() {
                SelectedRecordContentClass::Blob(kind) => {
                    read_failed_ingest_kind(kind, route.record(), route.content_class())?;
                }
                SelectedRecordContentClass::UnknownLegacy => return Err(Denial::CertificateRoster),
                _ => {}
            }
        }
        Ok(Self {
            manifests: routes
                .ordered()
                .filter(|route| {
                    route.content_class()
                        == SelectedRecordContentClass::Blob(BlobRecordKind::DropSetManifestV2)
                })
                .count(),
            descriptors: routes
                .ordered()
                .filter(|route| {
                    route.content_class()
                        == SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptor)
                })
                .count(),
            reservations: routes
                .ordered()
                .filter(|route| {
                    route.content_class()
                        == SelectedRecordContentClass::Blob(BlobRecordKind::OriginalDropReserved)
                })
                .count(),
            publications: routes
                .ordered()
                .filter(|route| {
                    route.content_class()
                        == SelectedRecordContentClass::Blob(BlobRecordKind::GenerationPublished)
                })
                .count(),
        })
    }
}

pub(super) struct ObservedFailedIngestControls {
    pub(super) manifests: Vec<(PersistedRecordIdentity, Manifest)>,
    pub(super) descriptors: Vec<Descriptor>,
    pub(super) reservations: Vec<OriginalDropReservedV1>,
    pub(super) publications: Vec<BlobGenerationPublicationV1>,
}

impl ObservedFailedIngestControls {
    pub(super) fn prepare<S: FailedIngestFrameStorage, R: FailedIngestRoutes + ?Sized>(
        routes: &R,
        storage: &mut S,
    ) -> Result<Self, Denial> {
        let count = ControlCensus::of(routes)?;
        Ok(Self {
            manifests: storage.reserve_vec(count.manifests)?,
            descriptors: storage.reserve_vec(count.descriptors)?,
            reservations: storage.reserve_vec(count.reservations)?,
            publications: storage.reserve_vec(count.publications)?,
        })
    }

    pub(super) fn observe_one<S: FailedIngestFrameStorage>(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        route: CurrentPhysicalRecordPlacement,
        format: PhysicalRecordFormatDeclaration,
        slices: &mut Vec<SelectedArtifactSlice>,
        max_slices: usize,
        storage: &mut S,
    ) -> Result<(), Denial> {
        let SelectedRecordContentClass::Blob(kind) = route.content_class() else {
            return if route.content_class() == SelectedRecordContentClass::UnknownLegacy {
                Err(Denial::CertificateRoster)
            } else {
                Ok(())
            };
        };
        if !read_failed_ingest_kind(kind, route.record(), route.content_class())? {
            return Ok(());
        }
        let bytes = storage.read_selected(
            discovery,
            format,
            route,
            MAX_CONTROL_FRAME_BYTES,
            slices,
            max_slices,
        )?;
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        match kind {
            BlobRecordKind::DropSetManifestV2 => {
                let value =
                    DropSetManifestV2View::decode(&bytes).map_err(|_| Denial::ControlFrame)?;
                if value.canonical_frame_sha256() != digest {
                    return Err(Denial::ControlFrame);
                }
                self.manifests.push((
                    route.record(),
                    Manifest {
                        store: value.store(),
                        attempt: value.reclaim_attempt(),
                        basis: value.source_basis(),
                        digest,
                        count: value.count(),
                        selected_slot: Some(value.never_reserved_slot_generation()),
                    },
                ));
            }
            _ => {
                let decoded = decode_blob_record(&bytes).map_err(|_| Denial::ControlFrame)?;
                if decoded.kind() != kind {
                    return Err(Denial::ControlFrame);
                }
                match decoded {
                    BlobRecordV1::ReclaimDescriptor(value) => self.descriptors.push(Descriptor {
                        record: route.record(),
                        frame_digest: digest,
                        store: value.store(),
                        attempt: value.reclaim_attempt(),
                        basis_digest: value.source_basis_digest(),
                        manifest: value.manifest_record(),
                        manifest_digest: value.manifest_frame_sha256(),
                        count: value.manifest_count(),
                        source_generation: value.source_root_generation(),
                        candidate_generation: value.candidate_root_generation(),
                    }),
                    BlobRecordV1::OriginalDropReserved(value) => self.reservations.push(value),
                    BlobRecordV1::GenerationPublished(value) => self.publications.push(value),
                    _ => return Err(Denial::CertificateRoster),
                }
            }
        }
        storage.discard_vec(bytes)
    }

    pub(super) fn discard<S: FailedIngestFrameStorage>(
        self,
        storage: &mut S,
    ) -> Result<(), Denial> {
        storage.discard_vec(self.manifests)?;
        storage.discard_vec(self.descriptors)?;
        storage.discard_vec(self.reservations)?;
        storage.discard_vec(self.publications)
    }
}

fn read_failed_ingest_kind(
    kind: BlobRecordKind,
    record: PersistedRecordIdentity,
    content_class: SelectedRecordContentClass,
) -> Result<bool, Denial> {
    match kind {
        BlobRecordKind::DropSetManifestV2
        | BlobRecordKind::ReclaimDescriptor
        | BlobRecordKind::OriginalDropReserved
        | BlobRecordKind::GenerationPublished => Ok(true),
        BlobRecordKind::DropSetManifestV3 | BlobRecordKind::ReclaimDescriptorV3 => {
            Err(Denial::CertificateRoster)
        }
        BlobRecordKind::DropSetManifest | BlobRecordKind::ReclaimDescriptorV2 => {
            Err(Denial::UnsupportedSelectedControl {
                record,
                content_class,
            })
        }
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_v1_manifest_is_unsupported_but_current_v2_and_v1_descriptor_remain_admitted() {
        let record = PersistedRecordIdentity::new([4; 16], 1).unwrap();
        let v1 = SelectedRecordContentClass::Blob(BlobRecordKind::DropSetManifest);
        assert!(matches!(
            read_failed_ingest_kind(BlobRecordKind::DropSetManifest, record, v1),
            Err(Denial::UnsupportedSelectedControl { record: rejected, .. }) if rejected == record
        ));
        assert!(matches!(
            read_failed_ingest_kind(
                BlobRecordKind::DropSetManifestV2,
                record,
                SelectedRecordContentClass::Blob(BlobRecordKind::DropSetManifestV2)
            ),
            Ok(true)
        ));
        assert!(matches!(
            read_failed_ingest_kind(
                BlobRecordKind::ReclaimDescriptor,
                record,
                SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptor)
            ),
            Ok(true)
        ));
        assert!(
            matches!(read_failed_ingest_kind(BlobRecordKind::ReclaimDescriptorV2, record,
            SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV2)),
            Err(Denial::UnsupportedSelectedControl { record: rejected, .. }) if rejected == record)
        );
    }
}
