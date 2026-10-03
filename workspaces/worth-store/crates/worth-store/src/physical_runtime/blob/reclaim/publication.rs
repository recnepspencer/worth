use sha2::{Digest, Sha256};
mod release_certificate;
mod released_directory;
mod result;
mod source;
use result::{failure, one_record, released_descriptor_record};
use source::ReclaimDescriptor;
pub(super) use source::ReclaimSource;
use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    BlobReclaimDescriptorV1, BlobReclaimDescriptorV2, BlobReclaimSourceBasisV1,
    BlobReclaimSourceKind, DropSetManifestV2, DropSetManifestV3, OriginalDropReservedV1,
    PersistedRecordIdentity,
};

use crate::physical_runtime::durability::ReleaseCertificateCapacityLease;
use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, BlobAppendFailure, CompletedPhysicalMutation,
    PhysicalMutationDeadline, PhysicalMutationIdempotencyKey, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, ServingPhysicalRuntime,
};

use super::{BlobReclaimFailure, BlobReclaimPublicationStage};

pub(super) struct ReclaimPublication<'a, 'runtime> {
    pub(super) runtime: &'runtime ServingPhysicalRuntime,
    pub(super) admitted: ReclaimSource<'a>,
    pub(super) placement: AdmittedRecordPlacementPolicy,
    pub(super) deadline: PhysicalMutationDeadline,
    pub(super) limits: super::BlobReclaimLimits,
    pub(super) allocation: &'a crate::physical_runtime::BlobPhysicalAllocation<'runtime>,
}

impl ReclaimPublication<'_, '_> {
    pub(super) fn publish(&self) -> Result<u64, BlobReclaimFailure> {
        let mut release_capacity = self.reserve_release_capacity()?;
        let mut directory_rebinding = match self.admitted {
            ReclaimSource::Released(admitted) => {
                released_directory::prepare_before_first_effect(self.runtime, admitted)?
            }
            ReclaimSource::Failed(_) => None,
        };
        let mut released_reservation = None;
        let mut released_descriptor = None;
        let source = self
            .admitted
            .attempt()
            .expected_root()
            .ok_or(BlobReclaimFailure::FenceLost)?
            .generation()
            .get();
        let manifest_selected = source.checked_add(1).ok_or(BlobReclaimFailure::FenceLost)?;
        let stage = BlobReclaimPublicationStage::Manifest;
        let request = self
            .request(stage)
            .map_err(|cause| failure(stage, None, cause))?;
        let (digest, source_digest, count, prepared) = match self.admitted {
            ReclaimSource::Failed(admitted) => {
                let manifest = DropSetManifestV2::new(
                    self.runtime.store_identity().bytes(),
                    admitted.attempt().bytes(),
                    admitted.basis(),
                    admitted.dropped().to_vec(),
                    manifest_selected,
                )
                .map_err(BlobReclaimFailure::Format)?;
                let digest = Sha256::digest(manifest.encode()).into();
                let source_digest = manifest.source_basis_digest();
                let count = manifest.count();
                let prepared = self
                    .runtime
                    .record_submission()
                    .prepare_blob_reclaim_manifest(admitted, manifest, self.placement, request);
                (digest, source_digest, count, prepared)
            }
            ReclaimSource::Released(admitted) => {
                let manifest = DropSetManifestV3::new(
                    self.runtime.store_identity().bytes(),
                    admitted.attempt().bytes(),
                    BlobReclaimSourceBasisV1::ReleasedGeneration(admitted.basis()),
                    admitted.dropped().to_vec(),
                    manifest_selected,
                )
                .map_err(BlobReclaimFailure::Format)?;
                let controls = self
                    .runtime
                    .record_submission()
                    .reserve_released_control_placements(
                        admitted,
                        &manifest,
                        self.placement,
                        self.allocation,
                        directory_rebinding
                            .as_ref()
                            .map(|rebind| rebind.encoded_bytes()),
                    )
                    .map_err(|cause| failure(stage, None, BlobAppendFailure::Preparation(cause)))?;
                let (manifest_placement, reservation_placement, descriptor_placement) =
                    controls.into_parts();
                released_reservation = Some(reservation_placement);
                released_descriptor = Some(descriptor_placement);
                let digest = Sha256::digest(manifest.encode()).into();
                let source_digest = manifest.source_basis_digest();
                let count = manifest.count();
                let prepared = self
                    .runtime
                    .record_submission()
                    .prepare_released_reclaim_manifest(
                        admitted,
                        manifest,
                        manifest_placement,
                        self.placement,
                        request,
                    );
                (digest, source_digest, count, prepared)
            }
        };
        let completed = self.execute(prepared, stage, None, release_capacity.as_mut())?;
        let manifest_record =
            one_record(&completed).map_err(|cause| failure(stage, None, cause))?;
        self.runtime
            .release_blob_ingest_clean_frames(completed.completed_data_frame_coordinates());

        let reserved_selected = manifest_selected
            .checked_add(1)
            .ok_or(BlobReclaimFailure::FenceLost)?;
        let candidate = reserved_selected
            .checked_add(1)
            .ok_or(BlobReclaimFailure::FenceLost)?;
        let descriptor = match self.admitted {
            ReclaimSource::Failed(_) => ReclaimDescriptor::Failed(
                BlobReclaimDescriptorV1::new(
                    self.runtime.store_identity().bytes(),
                    self.admitted.attempt().bytes(),
                    source_digest,
                    manifest_record,
                    digest,
                    count,
                    reserved_selected,
                    candidate,
                )
                .map_err(BlobReclaimFailure::Format)?,
            ),
            ReclaimSource::Released(admitted) => ReclaimDescriptor::Released(
                BlobReclaimDescriptorV2::new(
                    self.runtime.store_identity().bytes(),
                    self.admitted.attempt().bytes(),
                    BlobReclaimSourceKind::ReleasedGeneration,
                    source_digest,
                    manifest_record,
                    digest,
                    count,
                    reserved_selected,
                    candidate,
                    admitted.predecessor(),
                    admitted.cumulative_dropped(),
                    admitted.terminal(),
                )
                .map_err(BlobReclaimFailure::Format)?,
            ),
        };
        let drop_key = self
            .request_key(BlobReclaimPublicationStage::Drop)
            .map_err(|cause| self.reservation_failure(manifest_record, cause))?;
        let request_binding = self
            .describe_drop_request(&drop_key, descriptor)
            .ok_or_else(|| {
                self.admitted.attempt().settle_manifest_without_drop();
                BlobReclaimFailure::ConflictingSelectedFate
            })?;
        let reserved = OriginalDropReservedV1::new(
            self.runtime.store_identity().bytes(),
            self.admitted.attempt().bytes(),
            manifest_record,
            digest,
            source_digest,
            manifest_selected,
            reserved_selected,
            request_binding,
        )
        .map_err(|cause| {
            self.admitted.attempt().settle_manifest_without_drop();
            BlobReclaimFailure::Format(cause)
        })?;
        let reservation_sha256: [u8; 32] = Sha256::digest(reserved.encode()).into();
        let stage = BlobReclaimPublicationStage::Reservation;
        let request = self
            .request(stage)
            .map_err(|cause| self.reservation_failure(manifest_record, cause))?;
        let prepared = match self.admitted {
            ReclaimSource::Failed(admitted) => self
                .runtime
                .record_submission()
                .prepare_blob_reclaim_reservation(admitted, reserved, self.placement, request),
            ReclaimSource::Released(admitted) => self
                .runtime
                .record_submission()
                .prepare_released_reclaim_reservation(
                    admitted,
                    reserved,
                    released_reservation
                        .take()
                        .expect("released Manifest admitted all three placements"),
                    self.placement,
                    request,
                ),
        };
        let completed = self.execute(prepared, stage, Some(manifest_record), None)?;
        let reserved_record =
            one_record(&completed).map_err(|cause| failure(stage, Some(manifest_record), cause))?;
        self.runtime
            .release_blob_ingest_clean_frames(completed.completed_data_frame_coordinates());

        let released_certificate = self.certify_released(
            descriptor,
            manifest_record,
            digest,
            reserved_record,
            reservation_sha256,
            reserved,
            request_binding,
        )?;

        let stage = BlobReclaimPublicationStage::Drop;
        if let (ReclaimSource::Released(admitted), Some(rebinding)) =
            (self.admitted, directory_rebinding.as_ref())
        {
            released_directory::revalidate_before_drop(self.runtime, admitted, rebinding.basis())?;
        }
        let request = PhysicalMutationRequest::platform_durable(drop_key, self.deadline);
        let prepared = match (self.admitted, descriptor) {
            (ReclaimSource::Failed(admitted), ReclaimDescriptor::Failed(descriptor)) => self
                .runtime
                .record_submission()
                .drop_reclaimed_records(admitted, descriptor, self.placement, request),
            (ReclaimSource::Released(admitted), ReclaimDescriptor::Released(_)) => self
                .runtime
                .record_submission()
                .drop_released_generation_records(
                    admitted,
                    released_certificate
                        .as_ref()
                        .ok_or(BlobReclaimFailure::ConflictingSelectedFate)?
                        .1,
                    released_descriptor
                        .take()
                        .expect("released Manifest admitted all three placements"),
                    reserved_record,
                    reservation_sha256,
                    directory_rebinding
                        .as_mut()
                        .map(|rebind| (rebind.basis(), rebind.take_encoded())),
                    self.placement,
                    request,
                ),
            _ => unreachable!("source and descriptor constructed together"),
        };
        let completed = self.execute(prepared, stage, Some(manifest_record), None)?;
        if let (ReclaimSource::Released(admitted), Some((_, descriptor))) =
            (self.admitted, released_certificate.as_ref())
        {
            self.commit_release_certificate(
                admitted,
                *descriptor,
                reserved_record,
                reserved,
                manifest_record,
                &completed,
                directory_rebinding.is_some(),
            )?;
        }
        self.runtime
            .release_blob_ingest_clean_frames(completed.completed_data_frame_coordinates());
        Ok(reserved_selected)
    }

    fn describe_drop_request(
        &self,
        key: &PhysicalMutationIdempotencyKey,
        descriptor: ReclaimDescriptor,
    ) -> Option<worth_store_physical_format::OriginalDropReservationRequestV1> {
        match descriptor {
            ReclaimDescriptor::Failed(value) => self
                .runtime
                .record_submission()
                .describe_original_drop_request(key, value, self.placement),
            ReclaimDescriptor::Released(value) => self
                .runtime
                .record_submission()
                .describe_released_original_drop_request(key, value, self.placement),
        }
    }

    fn request(
        &self,
        stage: BlobReclaimPublicationStage,
    ) -> Result<PhysicalMutationRequest, BlobAppendFailure> {
        Ok(PhysicalMutationRequest::platform_durable(
            self.request_key(stage)?,
            self.deadline,
        ))
    }

    fn request_key(
        &self,
        stage: BlobReclaimPublicationStage,
    ) -> Result<PhysicalMutationIdempotencyKey, BlobAppendFailure> {
        let mut sha = Sha256::new();
        sha.update(b"worth.store.blob.reclaim.mutation.v1");
        sha.update(self.runtime.store_identity().bytes());
        sha.update(self.admitted.attempt().bytes());
        sha.update([match stage {
            BlobReclaimPublicationStage::Manifest => 1,
            BlobReclaimPublicationStage::Reservation => 3,
            BlobReclaimPublicationStage::Drop => 2,
        }]);
        let key = self
            .runtime
            .record_submission()
            .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(
                sha.finalize().into(),
            ))
            .map_err(BlobAppendFailure::Idempotency)?;
        Ok(key)
    }

    fn execute(
        &self,
        outcome: PhysicalMutationPreparationOutcome,
        stage: BlobReclaimPublicationStage,
        manifest: Option<PersistedRecordIdentity>,
        mut release_capacity: Option<&mut ReleaseCertificateCapacityLease>,
    ) -> Result<CompletedPhysicalMutation, BlobReclaimFailure> {
        let result = match outcome.into_raw() {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                if release_capacity
                    .as_mut()
                    .is_some_and(|lease| !lease.mark_effect_may_exist())
                {
                    return Err(BlobReclaimFailure::FenceLost);
                }
                self.admitted.attempt().mark_effect_started();
                match prepared.execute() {
                    PhysicalMutationOutcome::Completed(value) => Ok(value),
                    PhysicalMutationOutcome::ProvenNoEffect(value) => {
                        Err(BlobAppendFailure::ProvenNoEffect(value))
                    }
                    PhysicalMutationOutcome::Indeterminate(value) => {
                        Err(BlobAppendFailure::Indeterminate(value))
                    }
                }
            }
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Completed(value)) => {
                if release_capacity
                    .as_mut()
                    .is_some_and(|lease| !lease.mark_effect_may_exist())
                {
                    return Err(BlobReclaimFailure::FenceLost);
                }
                Ok(value)
            }
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::ProvenNoEffect(
                value,
            )) => Err(BlobAppendFailure::ProvenNoEffect(value)),
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Indeterminate(
                value,
            )) => {
                if release_capacity
                    .as_mut()
                    .is_some_and(|lease| !lease.mark_effect_may_exist())
                {
                    return Err(BlobReclaimFailure::FenceLost);
                }
                Err(BlobAppendFailure::Indeterminate(value))
            }
            other => Err(BlobAppendFailure::Preparation(other.into())),
        };
        if stage == BlobReclaimPublicationStage::Manifest {
            if let Err(BlobAppendFailure::ProvenNoEffect(proof)) = &result {
                self.admitted.attempt().prove_manifest_no_effect(proof);
            }
        }
        result.map_err(|cause| match manifest {
            Some(record) if stage == BlobReclaimPublicationStage::Reservation => {
                self.reservation_failure(record, cause)
            }
            Some(record) if stage == BlobReclaimPublicationStage::Drop => {
                self.drop_failure(record, cause)
            }
            _ => failure(stage, manifest, cause),
        })
    }
}
