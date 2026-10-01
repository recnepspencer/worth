//! Independent selected-route and operation-fate joins for checkpointed V3
//! release custody. Retired source pages are a Store custody transfer here,
//! never a fresh semantic re-proof from a descriptor digest alone.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    BoundedRecoveryFilesystemDiscovery, IntegrityAdmittedRecoveryWalFrame, StoreRecoveryWalMember,
};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1, BlobReclaimSourceKind,
    BlobRecordKind, BlobRecordV1, CurrentPhysicalRecordPlacement, DropSetManifestV3,
    OriginalDropReservedV1, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
    ReleasedDropCumulativeEvidenceV1, SelectedRecordContentClass, BLOB_CONTROL_FRAME_MAX_BYTES,
};
use worth_store_recovery_physics::ReconciledOperationFates;

use super::super::record;
use super::certificates::SelectedReleaseRoster;
use crate::{
    integrity_ingress::RecoveryIntegrityIngressTrace,
    orchestration::planning::manifest_entry_budget::ManifestEntryBudget,
};

#[path = "batch_join/fate.rs"]
mod fate;
#[path = "batch_join/source.rs"]
mod source;
pub(super) use source::SelectedReleasedGeneration;

pub(super) struct SelectedReleaseJoin<'a> {
    pub(super) routes: &'a [CurrentPhysicalRecordPlacement],
    pub(super) roster: &'a SelectedReleaseRoster,
    pub(super) fates: &'a ReconciledOperationFates,
    pub(super) selected_wal: &'a [&'a IntegrityAdmittedRecoveryWalFrame],
    pub(super) selected_members: &'a [StoreRecoveryWalMember],
    pub(super) policy: [u8; 32],
    pub(super) checkpoint_cutoff: u64,
    pub(super) store: [u8; 16],
}

impl SelectedReleaseJoin<'_> {
    pub(super) fn verify(
        &self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        budget: &mut ManifestEntryBudget,
        trace: &mut RecoveryIntegrityIngressTrace,
        scratch: &mut u64,
    ) -> bool {
        let accumulator = self.roster.accumulator();
        let mut prior_count = accumulator.prior_cumulative_dropped();
        let mut prior_digest = accumulator.prior_cumulative_digest();
        for batch in self.roster.batches() {
            let Some((descriptor, descriptor_frame)) = self.read_descriptor(
                discovery,
                format,
                budget,
                trace,
                scratch,
                batch.descriptor_record(),
            ) else {
                return false;
            };
            let base = descriptor.base();
            if <[u8; 32]>::from(Sha256::digest(&descriptor_frame))
                != batch.descriptor_frame_sha256()
                || descriptor.custody_digest() != batch.custody_digest()
                || descriptor.custody().request() != batch.request()
                || base.store() != self.store
                || base.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
                || base.candidate_root_generation() != batch.candidate_root_generation()
                || base.predecessor() != batch.predecessor()
                || base.terminal() != batch.terminal()
            {
                return false;
            }
            let Some((reservation, reservation_frame)) = self.read_reservation(
                discovery,
                format,
                budget,
                trace,
                scratch,
                batch.reservation_record(),
            ) else {
                return false;
            };
            if <[u8; 32]>::from(Sha256::digest(&reservation_frame))
                != batch.reservation_frame_sha256()
                || reservation.store() != self.store
                || reservation.reclaim_attempt() != base.reclaim_attempt()
                || reservation.manifest_record() != base.manifest_record()
                || reservation.manifest_frame_sha256() != base.manifest_frame_sha256()
                || reservation.source_basis_digest() != base.source_basis_digest()
                || reservation.reserved_selected_generation() != base.source_root_generation()
                || reservation.manifest_selected_generation() >= base.source_root_generation()
                || reservation.request() != batch.request()
                || descriptor.custody().request() != reservation.request()
            {
                return false;
            }
            let Some((manifest, manifest_frame)) = self.read_manifest(
                discovery,
                format,
                budget,
                trace,
                scratch,
                base.manifest_record(),
            ) else {
                return false;
            };
            if <[u8; 32]>::from(Sha256::digest(&manifest_frame)) != base.manifest_frame_sha256()
                || manifest.store() != self.store
                || manifest.reclaim_attempt() != base.reclaim_attempt()
                || manifest.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
                || manifest.source_basis_digest() != base.source_basis_digest()
                || manifest.count() != base.manifest_count()
                || manifest.never_reserved_slot_generation()
                    != reservation.manifest_selected_generation()
                || !matches!(
                    manifest.source_basis(),
                    BlobReclaimSourceBasisV1::ReleasedGeneration(_)
                )
            {
                return false;
            }
            let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis()
            else {
                return false;
            };
            if base.predecessor().is_none()
                && (base.cumulative_dropped() != u64::from(manifest.count())
                    || manifest
                        .dropped()
                        .binary_search(&source.publication_record())
                        .is_err())
            {
                return false;
            }
            if !self.durable_fate(batch.request(), base.reclaim_attempt(), batch.fate()) {
                return false;
            }
            let Ok(step) = ReleasedDropCumulativeEvidenceV1::new(
                batch.descriptor_record(),
                batch.descriptor_frame_sha256(),
                batch.custody_digest(),
                batch.reservation_record(),
                batch.reservation_frame_sha256(),
                batch.fate(),
                batch.candidate_root_generation(),
                batch.candidate_root_sha256(),
                batch.predecessor(),
                manifest.count(),
                batch.terminal(),
            ) else {
                return false;
            };
            let Ok((count, digest)) = step.advance(prior_count, prior_digest) else {
                return false;
            };
            if count != batch.cumulative_dropped() || digest != batch.cumulative_digest() {
                return false;
            }
            prior_count = count;
            prior_digest = digest;
        }
        prior_count == accumulator.cumulative_dropped()
            && prior_digest == accumulator.cumulative_digest()
    }

    /// A later selected checkpoint may carry only the ratchet and exact tip
    /// witness. Its prior checkpoint is Store-attested custody, not a replayed
    /// historical source graph. We still join the retained selected control
    /// routes and unique durable operation fate; missing material denies.
    pub(super) fn verify_carried_tip(
        &self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        budget: &mut ManifestEntryBudget,
        trace: &mut RecoveryIntegrityIngressTrace,
        scratch: &mut u64,
    ) -> bool {
        let accumulator = self.roster.accumulator();
        if !self.roster.batches().is_empty()
            || accumulator.prior_checkpoint_sequence() == 0
            || accumulator.batch_count() != 0
            || accumulator.batch_records_digest() != [0; 32]
        {
            return false;
        }
        let tip = accumulator.tip();
        let Some((descriptor, descriptor_frame)) = self.read_descriptor(
            discovery,
            format,
            budget,
            trace,
            scratch,
            tip.descriptor_record(),
        ) else {
            return false;
        };
        let base = descriptor.base();
        if <[u8; 32]>::from(Sha256::digest(&descriptor_frame)) != tip.descriptor_frame_sha256()
            || base.store() != self.store
            || base.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
            || base.candidate_root_generation() != tip.candidate_root_generation()
            || base.terminal() != accumulator.terminal()
            || descriptor.custody().request() != tip.request()
        {
            return false;
        }
        let Some((reservation, reservation_frame)) = self.read_reservation(
            discovery,
            format,
            budget,
            trace,
            scratch,
            tip.reservation_record(),
        ) else {
            return false;
        };
        if <[u8; 32]>::from(Sha256::digest(&reservation_frame)) != tip.reservation_frame_sha256()
            || reservation.store() != self.store
            || reservation.reclaim_attempt() != base.reclaim_attempt()
            || reservation.manifest_record() != base.manifest_record()
            || reservation.manifest_frame_sha256() != base.manifest_frame_sha256()
            || reservation.source_basis_digest() != base.source_basis_digest()
            || reservation.reserved_selected_generation() != base.source_root_generation()
            || reservation.request() != tip.request()
        {
            return false;
        }
        let Some((manifest, manifest_frame)) = self.read_manifest(
            discovery,
            format,
            budget,
            trace,
            scratch,
            base.manifest_record(),
        ) else {
            return false;
        };
        if <[u8; 32]>::from(Sha256::digest(&manifest_frame)) != base.manifest_frame_sha256()
            || manifest.store() != self.store
            || manifest.reclaim_attempt() != base.reclaim_attempt()
            || manifest.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
            || manifest.source_basis_digest() != base.source_basis_digest()
            || manifest.count() != base.manifest_count()
            || manifest.never_reserved_slot_generation()
                != reservation.manifest_selected_generation()
            || !matches!(
                manifest.source_basis(),
                BlobReclaimSourceBasisV1::ReleasedGeneration(_)
            )
        {
            return false;
        }
        let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis() else {
            return false;
        };
        if base.predecessor().is_none()
            && (base.cumulative_dropped() != u64::from(manifest.count())
                || manifest
                    .dropped()
                    .binary_search(&source.publication_record())
                    .is_err())
        {
            return false;
        }
        self.durable_fate(tip.request(), base.reclaim_attempt(), tip.fate())
    }

    fn read_descriptor(
        &self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        budget: &mut ManifestEntryBudget,
        trace: &mut RecoveryIntegrityIngressTrace,
        scratch: &mut u64,
        id: PersistedRecordIdentity,
    ) -> Option<(BlobReclaimDescriptorV3, Vec<u8>)> {
        let bytes = self.read(
            discovery,
            format,
            budget,
            trace,
            scratch,
            id,
            BlobRecordKind::ReclaimDescriptorV3,
        )?;
        let Ok(BlobRecordV1::ReclaimDescriptorV3(value)) = decode_blob_record(&bytes) else {
            return None;
        };
        Some((value, bytes))
    }

    fn read_reservation(
        &self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        budget: &mut ManifestEntryBudget,
        trace: &mut RecoveryIntegrityIngressTrace,
        scratch: &mut u64,
        id: PersistedRecordIdentity,
    ) -> Option<(OriginalDropReservedV1, Vec<u8>)> {
        let bytes = self.read(
            discovery,
            format,
            budget,
            trace,
            scratch,
            id,
            BlobRecordKind::OriginalDropReserved,
        )?;
        let Ok(BlobRecordV1::OriginalDropReserved(value)) = decode_blob_record(&bytes) else {
            return None;
        };
        Some((value, bytes))
    }

    fn read_manifest(
        &self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        budget: &mut ManifestEntryBudget,
        trace: &mut RecoveryIntegrityIngressTrace,
        scratch: &mut u64,
        id: PersistedRecordIdentity,
    ) -> Option<(DropSetManifestV3, Vec<u8>)> {
        let bytes = self.read(
            discovery,
            format,
            budget,
            trace,
            scratch,
            id,
            BlobRecordKind::DropSetManifestV3,
        )?;
        let Ok(BlobRecordV1::DropSetManifestV3(value)) = decode_blob_record(&bytes) else {
            return None;
        };
        Some((value, bytes))
    }

    fn read(
        &self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        format: PhysicalRecordFormatDeclaration,
        budget: &mut ManifestEntryBudget,
        trace: &mut RecoveryIntegrityIngressTrace,
        scratch: &mut u64,
        id: PersistedRecordIdentity,
        kind: BlobRecordKind,
    ) -> Option<Vec<u8>> {
        let route = self
            .routes
            .iter()
            .copied()
            .find(|route| route.record() == id)?;
        if !matches!(route,
            CurrentPhysicalRecordPlacement::Extent(extent)
                if extent.content_class() == SelectedRecordContentClass::Blob(kind)
                    && extent.payload_bytes() <= BLOB_CONTROL_FRAME_MAX_BYTES as u64)
        {
            return None;
        }
        record::read(
            discovery,
            format,
            Some(route),
            id,
            BLOB_CONTROL_FRAME_MAX_BYTES as u64,
            budget,
            trace,
            scratch,
        )
        .ok()
    }
}
