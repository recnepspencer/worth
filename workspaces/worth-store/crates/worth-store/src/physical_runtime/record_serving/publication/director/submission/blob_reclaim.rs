use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    BlobReclaimDescriptorV1, BlobReclaimDescriptorV2, BlobReclaimDescriptorV3, DropSetManifestV2,
    DropSetManifestV3, OriginalDropReservationRequestV1, OriginalDropReservedV1,
    PersistedRecordIdentity,
};

use super::PhysicalRecordSubmission;
use crate::physical_runtime::{
    durability::{AdmittedFailedIngestDrop, AdmittedReleasedGenerationDrop},
    record_serving::{
        publication::{PhysicalMutationPreparationOutcome, PhysicalMutationPreparationStale},
        AdmittedRecordPlacementPolicy, ReleasedControlArenaPlacement,
        ReleasedControlArenaReservations,
    },
    PhysicalMutationIdempotencyKey, PhysicalMutationRequest,
};

impl PhysicalRecordSubmission {
    pub(in crate::physical_runtime) fn reserve_released_control_placements(
        &self,
        admitted: &AdmittedReleasedGenerationDrop,
        manifest: &DropSetManifestV3,
        placement: AdmittedRecordPlacementPolicy,
        allocation: &crate::physical_runtime::BlobPhysicalAllocation<'_>,
    ) -> Result<ReleasedControlArenaReservations, PhysicalMutationPreparationOutcome> {
        let Some(director) = self.director.upgrade() else {
            return Err(TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into());
        };
        director.reserve_released_control_placements(admitted, manifest, placement, allocation)
    }

    pub(in crate::physical_runtime) fn prepare_released_reclaim_manifest(
        &self,
        admitted: &AdmittedReleasedGenerationDrop,
        manifest: DropSetManifestV3,
        control_placement: ReleasedControlArenaPlacement,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into();
        };
        director.prepare_released_reclaim_manifest(
            admitted,
            manifest,
            control_placement,
            placement,
            request,
        )
    }

    pub(in crate::physical_runtime) fn prepare_released_reclaim_reservation(
        &self,
        admitted: &AdmittedReleasedGenerationDrop,
        reserved: OriginalDropReservedV1,
        control_placement: ReleasedControlArenaPlacement,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into();
        };
        director.prepare_released_reclaim_reservation(
            admitted,
            reserved,
            control_placement,
            placement,
            request,
        )
    }

    pub(in crate::physical_runtime) fn describe_released_original_drop_request(
        &self,
        key: &PhysicalMutationIdempotencyKey,
        descriptor: BlobReclaimDescriptorV2,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Option<OriginalDropReservationRequestV1> {
        self.director
            .upgrade()?
            .describe_released_original_drop_request(key, descriptor, placement)
    }

    pub(in crate::physical_runtime) fn drop_released_generation_records(
        &self,
        admitted: &AdmittedReleasedGenerationDrop,
        descriptor: BlobReclaimDescriptorV3,
        control_placement: ReleasedControlArenaPlacement,
        reservation_record: PersistedRecordIdentity,
        reservation_frame_sha256: [u8; 32],
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into();
        };
        director.drop_released_generation_records(
            admitted,
            descriptor,
            control_placement,
            reservation_record,
            reservation_frame_sha256,
            placement,
            request,
        )
    }

    pub(in crate::physical_runtime) fn expected_original_drop_fingerprint(
        &self,
        descriptor: BlobReclaimDescriptorV1,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Option<[u8; 32]> {
        self.director
            .upgrade()?
            .expected_original_drop_fingerprint(descriptor, placement)
    }

    pub(in crate::physical_runtime) fn prepare_blob_reclaim_manifest(
        &self,
        admitted: &AdmittedFailedIngestDrop,
        manifest: DropSetManifestV2,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into();
        };
        director.prepare_blob_reclaim_manifest(admitted, manifest, placement, request)
    }

    pub(in crate::physical_runtime) fn describe_original_drop_request(
        &self,
        key: &PhysicalMutationIdempotencyKey,
        descriptor: BlobReclaimDescriptorV1,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Option<OriginalDropReservationRequestV1> {
        self.director
            .upgrade()?
            .describe_original_drop_request(key, descriptor, placement)
    }

    pub(in crate::physical_runtime) fn prepare_blob_reclaim_reservation(
        &self,
        admitted: &AdmittedFailedIngestDrop,
        reserved: OriginalDropReservedV1,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into();
        };
        director.prepare_blob_reclaim_reservation(admitted, reserved, placement, request)
    }

    pub(in crate::physical_runtime) fn drop_reclaimed_records(
        &self,
        admitted: &AdmittedFailedIngestDrop,
        descriptor: BlobReclaimDescriptorV1,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into();
        };
        director.drop_reclaimed_records(admitted, descriptor, placement, request)
    }
}
