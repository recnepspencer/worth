use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    arena_tier_at_epoch, decode_blob_record, BlobReclaimDescriptorV3, BlobRecordKind, BlobRecordV1,
    CurrentPhysicalRecordPlacement, DropSetManifestV3, DurableExtentRecordPlacement,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration, SelectedRecordContentClass,
};
use worth_store_physical_integrity::IntegrityValidatedSelectedExtentPayload;
use worth_store_recovery_physics::VerifiedSelectedCheckpointCustody;

use super::{root_checkpoint::ObservedRootCheckpoint, SelectedMediaRejoinDenial as Denial};
mod addressed;
pub(super) mod catalog;
mod extent_read;
mod route;
mod snapshot;
pub(super) use addressed::observe_addressed_base;
pub(super) use extent_read::read_extent;
pub(super) use extent_read::read_extent_with_resident;
pub(in crate::physical_runtime::recovery_construction::selected_rejoin) use extent_read::{
    read_extent_with_storage, ExtentReadStorage,
};
use route::selected_route;
pub(in crate::physical_runtime::recovery_construction) use snapshot::FundedCompletedHistoricalRawSlices;
pub(in crate::physical_runtime::recovery_construction) use snapshot::FundedHeadEffectSlices;
pub(in crate::physical_runtime) use snapshot::SelectedArtifactSlice;
pub(in crate::physical_runtime) use snapshot::SelectedControlMediaFingerprint;

/// A selected route and complete frame witnessed through the same admitted media.
pub(super) struct ObservedSelectedControlFrame {
    placement: DurableExtentRecordPlacement,
    kind: BlobRecordKind,
    bytes: Vec<u8>,
    frame_sha256: [u8; 32],
    witness: IntegrityValidatedSelectedExtentPayload,
    slices: Vec<SelectedArtifactSlice>,
}

impl ObservedSelectedControlFrame {
    pub(super) const fn placement(&self) -> DurableExtentRecordPlacement {
        self.placement
    }
    pub(super) const fn record(&self) -> PersistedRecordIdentity {
        self.placement.record()
    }
    pub(super) const fn kind(&self) -> BlobRecordKind {
        self.kind
    }
    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub(super) const fn frame_sha256(&self) -> [u8; 32] {
        self.frame_sha256
    }
    pub(super) const fn witness(&self) -> IntegrityValidatedSelectedExtentPayload {
        self.witness
    }
}

pub(super) struct ObservedSelectedControls {
    descriptor: ObservedSelectedControlFrame,
    reservation: ObservedSelectedControlFrame,
    manifest: ObservedSelectedControlFrame,
    additional: Vec<ObservedSelectedControlFrame>,
}

impl ObservedSelectedControls {
    pub(super) fn latest_for_source(
        &self,
        source: worth_store_physical_format::BlobReclaimSourceBasisV1,
    ) -> Option<(PersistedRecordIdentity, [u8; 32])> {
        catalog::latest_for_source(self, source)
    }
    pub(super) fn predecessor_controls(
        &self,
        record: PersistedRecordIdentity,
        sha256: [u8; 32],
    ) -> Option<(BlobReclaimDescriptorV3, DropSetManifestV3)> {
        catalog::predecessor_controls(self, record, sha256)
    }
    pub(super) fn retained_memory_bytes(&self) -> u64 {
        let frame_bytes = |frame: &ObservedSelectedControlFrame| {
            (frame.bytes.capacity() as u64).saturating_add(
                (frame.slices.capacity() as u64)
                    .saturating_mul(4 * std::mem::size_of::<SelectedArtifactSlice>() as u64),
            )
        };
        [&self.descriptor, &self.reservation, &self.manifest]
            .into_iter()
            .chain(self.additional.iter())
            .fold(
                (std::mem::size_of::<Self>() as u64).saturating_add(
                    (self.additional.capacity() as u64).saturating_mul(
                        4 * std::mem::size_of::<ObservedSelectedControlFrame>() as u64,
                    ),
                ),
                |sum, frame| sum.saturating_add(frame_bytes(frame)),
            )
    }

    pub(super) fn fingerprint(&self) -> SelectedControlMediaFingerprint {
        SelectedControlMediaFingerprint::observed(
            [&self.descriptor, &self.reservation, &self.manifest]
                .into_iter()
                .chain(self.additional.iter())
                .flat_map(|frame| frame.slices.iter().cloned())
                .collect(),
        )
    }
    pub(super) fn descriptor(&self) -> &ObservedSelectedControlFrame {
        &self.descriptor
    }
    pub(super) fn reservation(&self) -> &ObservedSelectedControlFrame {
        &self.reservation
    }
    pub(super) fn manifest(&self) -> &ObservedSelectedControlFrame {
        &self.manifest
    }

    pub(super) fn attempt_for_descriptor(
        &self,
        record: PersistedRecordIdentity,
    ) -> Option<[u8; 16]> {
        catalog::attempt_for_descriptor(self, record)
    }

    pub(super) fn matches_reread(&self, other: &Self) -> bool {
        self.additional.len() == other.additional.len()
            && [
                (&self.descriptor, &other.descriptor),
                (&self.reservation, &other.reservation),
                (&self.manifest, &other.manifest),
            ]
            .into_iter()
            .chain(self.additional.iter().zip(&other.additional))
            .all(|(before, after)| {
                before.placement() == after.placement()
                    && before.kind() == after.kind()
                    && before.frame_sha256() == after.frame_sha256()
                    && before.bytes() == after.bytes()
            })
    }

    pub(super) fn matches_claim(
        &self,
        claim: &VerifiedSelectedCheckpointCustody,
    ) -> Result<(), Denial> {
        let tip = claim.accumulator().tip();
        let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
            decode_blob_record(self.descriptor.bytes()).map_err(|_| Denial::ControlFrame)?
        else {
            return Err(Denial::ControlFrame);
        };
        let BlobRecordV1::OriginalDropReserved(reservation) =
            decode_blob_record(self.reservation.bytes()).map_err(|_| Denial::ControlFrame)?
        else {
            return Err(Denial::ControlFrame);
        };
        let BlobRecordV1::DropSetManifestV3(manifest) =
            decode_blob_record(self.manifest.bytes()).map_err(|_| Denial::ControlFrame)?
        else {
            return Err(Denial::ControlFrame);
        };
        let base = descriptor.base();
        if self.descriptor.record() != tip.descriptor_record()
            || self.descriptor.frame_sha256() != tip.descriptor_frame_sha256()
            || self.reservation.record() != tip.reservation_record()
            || self.reservation.frame_sha256() != tip.reservation_frame_sha256()
            || self.manifest.record() != base.manifest_record()
            || self.manifest.frame_sha256() != base.manifest_frame_sha256()
            || self.manifest.record() != reservation.manifest_record()
            || self.manifest.frame_sha256() != reservation.manifest_frame_sha256()
            || base.manifest_count() != manifest.count()
            || base.store() != reservation.store()
            || base.store() != manifest.store()
            || base.reclaim_attempt() != reservation.reclaim_attempt()
            || base.reclaim_attempt() != manifest.reclaim_attempt()
            || base.source_basis_digest() != reservation.source_basis_digest()
            || base.source_basis_digest() != manifest.source_basis_digest()
            || descriptor.custody().request() != tip.request()
            || reservation.request() != tip.request()
            || descriptor.custody_digest() != claim.tip_custody_digest()
            || base.terminal() != claim.accumulator().terminal()
        {
            return Err(Denial::ControlFrame);
        }
        let mut frames = BTreeMap::new();
        for frame in [&self.descriptor, &self.reservation, &self.manifest]
            .into_iter()
            .chain(self.additional.iter())
        {
            if frames.insert(frame.record(), frame).is_some() {
                return Err(Denial::ControlFrame);
            }
        }
        catalog::verify(claim, &frames)
    }
}

pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selected: &ObservedRootCheckpoint,
    claim: &VerifiedSelectedCheckpointCustody,
    selected_routes: &BTreeMap<PersistedRecordIdentity, CurrentPhysicalRecordPlacement>,
    tier_epoch_start: Option<u64>,
    per_frame_max_bytes: u64,
) -> Result<ObservedSelectedControls, Denial> {
    let tip = claim.accumulator().tip();
    let descriptor_id = tip.descriptor_record();
    let reservation_id = tip.reservation_record();
    if descriptor_id == reservation_id {
        return Err(Denial::ControlFrame);
    }
    let format = selected.selector().format();
    let descriptor = observe_one(
        discovery,
        selected,
        format,
        descriptor_id,
        BlobRecordKind::ReclaimDescriptorV3,
        tier_epoch_start,
        per_frame_max_bytes,
    )?;
    let BlobRecordV1::ReclaimDescriptorV3(decoded) =
        decode_blob_record(descriptor.bytes()).map_err(|_| Denial::ControlFrame)?
    else {
        return Err(Denial::ControlFrame);
    };
    let manifest_id = decoded.base().manifest_record();
    if manifest_id == descriptor_id || manifest_id == reservation_id {
        return Err(Denial::ControlFrame);
    }
    let reservation = observe_one(
        discovery,
        selected,
        format,
        reservation_id,
        BlobRecordKind::OriginalDropReserved,
        tier_epoch_start,
        per_frame_max_bytes,
    )?;
    let manifest = observe_one(
        discovery,
        selected,
        format,
        manifest_id,
        BlobRecordKind::DropSetManifestV3,
        tier_epoch_start,
        per_frame_max_bytes,
    )?;
    let mut additional = Vec::new();
    for (&record, &route) in selected_routes {
        if [descriptor_id, reservation_id, manifest_id].contains(&record) {
            continue;
        }
        let SelectedRecordContentClass::Blob(kind) = route.content_class() else {
            return Err(Denial::CertificateRoster);
        };
        if !matches!(
            kind,
            BlobRecordKind::ReclaimDescriptorV3
                | BlobRecordKind::OriginalDropReserved
                | BlobRecordKind::DropSetManifestV3
        ) {
            return Err(Denial::CertificateRoster);
        }
        additional.push(observe_one(
            discovery,
            selected,
            format,
            record,
            kind,
            tier_epoch_start,
            per_frame_max_bytes,
        )?);
    }
    let controls = ObservedSelectedControls {
        descriptor,
        reservation,
        manifest,
        additional,
    };
    controls.matches_claim(claim)?;
    Ok(controls)
}

fn observe_one(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selected: &ObservedRootCheckpoint,
    format: PhysicalRecordFormatDeclaration,
    record: PersistedRecordIdentity,
    kind: BlobRecordKind,
    tier_epoch_start: Option<u64>,
    maximum: u64,
) -> Result<ObservedSelectedControlFrame, Denial> {
    let mut slices = Vec::new();
    let route = selected_route(discovery, selected, format, record, &mut slices)?;
    if route.content_class() != SelectedRecordContentClass::Blob(kind) {
        return Err(Denial::ControlFrame);
    }
    let CurrentPhysicalRecordPlacement::Extent(placement) = route else {
        return Err(Denial::UnsupportedSelectedPlacement);
    };
    if selected.root().tier_epoch_anchor().is_some() != tier_epoch_start.is_some()
        || placement.tier_class()
            != arena_tier_at_epoch(tier_epoch_start, placement.arena_range().arena())
    {
        return Err(Denial::ControlFrame);
    }
    if placement.payload_bytes() == 0 || placement.payload_bytes() > maximum {
        return Err(Denial::BoundExceeded);
    }
    let (bytes, witness) = read_extent(discovery, format, placement, &mut slices)?;
    let decoded = decode_blob_record(&bytes).map_err(|_| Denial::ControlFrame)?;
    if decoded.kind() != kind || !witness.matches_frame(&bytes) {
        return Err(Denial::ControlFrame);
    }
    let frame_sha256 = Sha256::digest(&bytes).into();
    Ok(ObservedSelectedControlFrame {
        placement,
        kind,
        bytes,
        frame_sha256,
        witness,
        slices,
    })
}
