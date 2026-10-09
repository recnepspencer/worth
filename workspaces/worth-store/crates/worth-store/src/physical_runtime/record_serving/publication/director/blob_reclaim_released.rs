use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    BlobReclaimDescriptorV2, BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1,
    BlobReclaimSourceKind, BlobRecordKind, DropSetManifestV3, OriginalDropReservationRequestV1,
    OriginalDropReservedV1, PersistedRecordIdentity, ReleaseCustodyHeadKeyV1,
    ReleasedDropPredecessorV1,
};

use super::RecordPublicationDirector;
use crate::physical_runtime::{
    durability::AdmittedReleasedGenerationDrop,
    record_serving::{
        publication::{
            PhysicalMutationPreparationOutcome, PhysicalMutationPreparationSuccess,
            PreparedReleaseHeadBasis,
        },
        AdmittedRecordPlacementPolicy, RecordAppendDenial, ReleasedControlArenaPlacement,
    },
    PhysicalMutationIdempotencyKey, PhysicalMutationRequest,
};

use super::durable_preparation::map_record_denial;

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn selected_release_head_for_root(
        &self,
        inspector: crate::physical_runtime::PhysicalProtectedRootObservation,
        key: ReleaseCustodyHeadKeyV1,
    ) -> Result<
        crate::physical_runtime::durability::SelectedReleaseHeadBasis,
        crate::physical_runtime::durability::SelectedReleaseHeadDenial,
    > {
        self.root_owner
            .selected_release_head_for_root(inspector, key)
    }

    pub(in crate::physical_runtime) fn commit_selected_release_certificate(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
        selected: &crate::physical_runtime::blob::reclaim::released::SelectedReleasedDescriptorObservation,
        completed: &crate::physical_runtime::CompletedPhysicalMutation,
    ) -> Result<(), crate::physical_runtime::durability::ReleaseCertificateCapacityDenial> {
        self.root_owner.commit_selected_release_certificate(
            attempt,
            selected,
            completed,
            self.format.declaration(),
        )
    }

    pub(in crate::physical_runtime) fn prepare_released_reclaim_manifest(
        &self,
        admitted: &AdmittedReleasedGenerationDrop,
        manifest: DropSetManifestV3,
        control_placement: ReleasedControlArenaPlacement,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        if let Err(denial) = self.validate_released_reclaim_manifest(admitted, &manifest) {
            return map_record_denial(denial);
        }
        let encoded = manifest.encode();
        if !control_placement.matches(
            admitted.attempt().bytes(),
            BlobRecordKind::DropSetManifestV3,
            encoded.len() as u64,
        ) {
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        }
        let outcome = self.prepare_fenced_reclaim_record(
            admitted.attempt(),
            encoded,
            BlobRecordKind::DropSetManifestV3,
            placement,
            request,
        );
        self.attach_released_control_placement(outcome, control_placement)
    }

    pub(super) fn validate_released_reclaim_manifest(
        &self,
        admitted: &AdmittedReleasedGenerationDrop,
        manifest: &DropSetManifestV3,
    ) -> Result<(), RecordAppendDenial> {
        if manifest.store() != self.durability.store_identity().bytes()
            || manifest.reclaim_attempt() != admitted.attempt().bytes()
            || manifest.source_basis()
                != BlobReclaimSourceBasisV1::ReleasedGeneration(admitted.basis())
            || manifest.dropped() != admitted.dropped()
            || admitted.attempt().expected_root().is_none_or(|root| {
                root.generation().get().checked_add(1)
                    != Some(manifest.never_reserved_slot_generation())
            })
        {
            return Err(RecordAppendDenial::ReclaimFenceUnavailable);
        }
        Ok(())
    }

    pub(in crate::physical_runtime) fn prepare_released_reclaim_reservation(
        &self,
        admitted: &AdmittedReleasedGenerationDrop,
        reserved: OriginalDropReservedV1,
        control_placement: ReleasedControlArenaPlacement,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        if reserved.store() != self.durability.store_identity().bytes()
            || reserved.reclaim_attempt() != admitted.attempt().bytes()
            || reserved.source_basis_digest()
                != BlobReclaimSourceBasisV1::ReleasedGeneration(admitted.basis())
                    .digest(self.durability.store_identity().bytes())
            || admitted.attempt().expected_root().is_none_or(|root| {
                root.generation().get() != reserved.manifest_selected_generation()
                    || root.generation().get().checked_add(1)
                        != Some(reserved.reserved_selected_generation())
            })
        {
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        }
        let encoded = reserved.encode();
        if !control_placement.matches(
            admitted.attempt().bytes(),
            BlobRecordKind::OriginalDropReserved,
            encoded.len() as u64,
        ) {
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        }
        let outcome = self.prepare_fenced_reclaim_record(
            admitted.attempt(),
            encoded,
            BlobRecordKind::OriginalDropReserved,
            placement,
            request,
        );
        self.attach_released_control_placement(outcome, control_placement)
    }

    pub(in crate::physical_runtime) fn describe_released_original_drop_request(
        &self,
        key: &PhysicalMutationIdempotencyKey,
        descriptor: BlobReclaimDescriptorV2,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Option<OriginalDropReservationRequestV1> {
        OriginalDropReservationRequestV1::new(
            key.identity().bytes(),
            self.expected_original_drop_fingerprint_bytes(descriptor.encode(), placement)?,
            key.lease().issuance_generation().get(),
            key.lease().expiry_generation().get(),
        )
        .ok()
    }

    pub(in crate::physical_runtime) fn drop_released_generation_records(
        &self,
        admitted: &AdmittedReleasedGenerationDrop,
        descriptor: BlobReclaimDescriptorV3,
        control_placement: ReleasedControlArenaPlacement,
        reservation_record: PersistedRecordIdentity,
        reservation_frame_sha256: [u8; 32],
        directory_rebinding: Option<(
            crate::physical_runtime::record_serving::PreparedReleasedDirectoryRebinding,
            Vec<u8>,
        )>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(expected_root) = admitted.attempt().expected_root() else {
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        };
        let base = descriptor.base();
        let binding = descriptor.custody().request();
        let key = request.idempotency_key();
        let Some(head_key) =
            ReleaseCustodyHeadKeyV1::new(admitted.basis().object(), admitted.basis().generation())
        else {
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        };
        let prior = match self
            .root_owner
            .selected_release_head_for_attempt(admitted.attempt(), head_key)
        {
            Ok(prior) => prior,
            Err(_) => return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable),
        };
        let prior_predecessor = prior.map(|entry| {
            ReleasedDropPredecessorV1::new(
                entry.descriptor_record(),
                entry.descriptor_frame_sha256(),
            )
            .expect("a selected head has a nonzero descriptor digest")
        });
        if !control_placement.matches(
            admitted.attempt().bytes(),
            BlobRecordKind::ReclaimDescriptorV3,
            descriptor.encode().len() as u64,
        ) || control_placement.directory_encoded_bytes()
            != directory_rebinding
                .as_ref()
                .map(|(_, encoded)| encoded.len() as u64)
        {
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        }
        if binding.idempotency() != key.identity().bytes()
            || binding.lease_issuance_generation() != key.lease().issuance_generation().get()
            || binding.lease_expiry_generation() != key.lease().expiry_generation().get()
            || self.expected_original_drop_fingerprint_bytes(base.encode(), placement)
                != Some(binding.fingerprint())
            || base.store() != self.durability.store_identity().bytes()
            || base.reclaim_attempt() != admitted.attempt().bytes()
            || base.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
            || base.source_basis_digest()
                != BlobReclaimSourceBasisV1::ReleasedGeneration(admitted.basis())
                    .digest(self.durability.store_identity().bytes())
            || usize::from(base.manifest_count()) != admitted.dropped().len()
            || base.source_root_generation() != expected_root.generation().get()
            || base.candidate_root_generation()
                != expected_root.generation().get().saturating_add(1)
            || base.predecessor() != admitted.predecessor()
            || base.predecessor() != prior_predecessor
            || base.cumulative_dropped() != admitted.cumulative_dropped()
            || prior.is_some_and(|entry| {
                entry.terminal()
                    || entry.source_basis_digest() != base.source_basis_digest()
                    || entry
                        .cumulative_dropped()
                        .checked_add(admitted.dropped().len() as u64)
                        != Some(base.cumulative_dropped())
            })
            || base.terminal() != admitted.terminal()
            || admitted.dropped().contains(&base.manifest_record())
            || !admitted.attempt().register_drop_records(admitted.dropped())
        {
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        }
        let directory_basis = directory_rebinding.as_ref().map(|(basis, _)| *basis);
        let outcome = self.prepare_fenced_released_descriptor(
            admitted.attempt(),
            descriptor,
            directory_rebinding,
            placement,
            request,
        );
        let outcome = self.attach_released_control_placement(outcome, control_placement);
        match outcome.into_raw() {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                TransitionOutcome::success(PhysicalMutationPreparationSuccess::Prepared(
                    prepared.with_released_drop_basis(
                        crate::physical_runtime::record_serving::PreparedReleasedDropBasis::new(
                            PreparedReleaseHeadBasis::new(
                                head_key,
                                admitted.basis(),
                                prior,
                                descriptor,
                                reservation_record,
                                reservation_frame_sha256,
                            ),
                            directory_basis,
                        ),
                    ),
                ))
                .into()
            }
            other => {
                admitted.attempt().clear_unprepared_drop_records();
                other.into()
            }
        }
    }
}
