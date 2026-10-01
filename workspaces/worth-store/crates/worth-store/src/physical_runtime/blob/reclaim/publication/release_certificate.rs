use worth_store_physical_format::{
    BlobReclaimDescriptorV3, OriginalDropReservationRequestV1, OriginalDropReservedV1,
    PersistedRecordIdentity, ReleaseCustodyHeadKeyV1,
};

use crate::physical_runtime::{
    durability::{
        AdmittedReleasedGenerationDrop, ReleaseCertificateCapacityDenial,
        ReleaseCertificateCapacityLease,
    },
    CompletedPhysicalMutation,
};

use super::{failure, one_record, ReclaimDescriptor, ReclaimPublication, ReclaimSource};
use crate::physical_runtime::blob::reclaim::{
    released, BlobReclaimDeferral, BlobReclaimFailure, BlobReclaimPublicationStage,
};

impl ReclaimPublication<'_, '_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn certify_released(
        &self,
        descriptor: ReclaimDescriptor,
        manifest_record: PersistedRecordIdentity,
        manifest_sha256: [u8; 32],
        reservation_record: PersistedRecordIdentity,
        reservation_sha256: [u8; 32],
        reservation: OriginalDropReservedV1,
        request: OriginalDropReservationRequestV1,
    ) -> Result<
        Option<(
            released::CertifiedPendingReleasedDrop,
            BlobReclaimDescriptorV3,
        )>,
        BlobReclaimFailure,
    > {
        let (ReclaimSource::Released(admitted), ReclaimDescriptor::Released(base)) =
            (self.admitted, descriptor)
        else {
            return Ok(None);
        };
        let (reader, root_sha256, free_sha256) = self
            .runtime
            .capture_released_reclaim_source(admitted.attempt())?;
        let certified = released::certify_pending_drop(
            self.runtime,
            reader,
            root_sha256,
            free_sha256,
            admitted.basis(),
            base,
            manifest_record,
            manifest_sha256,
            reservation_record,
            reservation_sha256,
            reservation,
            admitted.dropped(),
            self.limits,
            self.allocation,
        )?;
        let custody = certified.custody(request)?;
        let descriptor =
            BlobReclaimDescriptorV3::new(base, custody).map_err(BlobReclaimFailure::Format)?;
        Ok(Some((certified, descriptor)))
    }

    pub(super) fn reserve_release_capacity(
        &self,
    ) -> Result<Option<ReleaseCertificateCapacityLease>, BlobReclaimFailure> {
        let ReclaimSource::Released(admitted) = self.admitted else {
            return Ok(None);
        };
        // The same pre-effect lease covers tag7 and the selected head's full
        // recovery closure. Neither admission can be deferred until after WAL.
        let worst_case_encoded_bytes =
            (worth_store_physical_format::RELEASE_CHECKPOINT_BATCH_WIRE_BYTES
                + worth_store_physical_format::RELEASE_CHECKPOINT_ACCUMULATOR_V2_WIRE_BYTES
                + 40) as u32;
        let key =
            ReleaseCustodyHeadKeyV1::new(admitted.basis().object(), admitted.basis().generation())
                .ok_or(BlobReclaimFailure::FenceLost)?;
        let head_charge =
            self.runtime
                .released_head_capacity_charge()
                .ok_or(BlobReclaimFailure::Deferred(
                    BlobReclaimDeferral::ReleaseCertificateCapacity,
                ))?;
        self.runtime
            .reserve_release_certificate_capacity(
                admitted.attempt(),
                key,
                2,
                worst_case_encoded_bytes,
                head_charge,
            )
            .map(Some)
            .map_err(|denial| match denial {
                ReleaseCertificateCapacityDenial::Resident(cause) => {
                    BlobReclaimFailure::ReleaseCertificateBacking(cause)
                }
                ReleaseCertificateCapacityDenial::CapacityExhausted
                | ReleaseCertificateCapacityDenial::SelectedLedgerUnavailable => {
                    BlobReclaimFailure::Deferred(BlobReclaimDeferral::ReleaseCertificateCapacity)
                }
                _ => BlobReclaimFailure::FenceLost,
            })
    }

    pub(super) fn commit_release_certificate(
        &self,
        admitted: &AdmittedReleasedGenerationDrop,
        descriptor: BlobReclaimDescriptorV3,
        reserved_record: PersistedRecordIdentity,
        reserved: OriginalDropReservedV1,
        manifest_record: PersistedRecordIdentity,
        completed: &CompletedPhysicalMutation,
    ) -> Result<(), BlobReclaimFailure> {
        let descriptor_record = one_record(completed).map_err(|cause| {
            failure(
                BlobReclaimPublicationStage::Drop,
                Some(manifest_record),
                cause,
            )
        })?;
        let selected = released::observe_selected_descriptor(
            self.runtime,
            admitted.attempt(),
            completed,
            descriptor_record,
            descriptor,
            reserved_record,
            reserved,
            self.limits,
        )?;
        self.runtime
            .commit_selected_release_certificate(admitted.attempt(), &selected, completed)
            .map_err(|_| BlobReclaimFailure::ConflictingSelectedFate)
    }
}
