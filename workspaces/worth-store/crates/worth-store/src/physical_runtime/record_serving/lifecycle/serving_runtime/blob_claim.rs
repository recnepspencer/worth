use crate::physical_runtime::{
    durability::{
        PhysicalBlobSessionClaim, PhysicalBlobSessionClaimDenial,
        PhysicalBlobTerminalAdmissionDenial,
    },
    BlobSessionId, PhysicalRecordReader,
};
use sha2::{Digest, Sha256};

use super::ServingPhysicalRuntime;

impl ServingPhysicalRuntime {
    pub(in crate::physical_runtime) fn selected_release_head_for_root(
        &self,
        inspector: crate::physical_runtime::PhysicalProtectedRootObservation,
        key: worth_store_physical_format::ReleaseCustodyHeadKeyV1,
    ) -> Result<
        crate::physical_runtime::durability::SelectedReleaseHeadBasis,
        crate::physical_runtime::durability::SelectedReleaseHeadDenial,
    > {
        self.parts
            .publication
            .selected_release_head_for_root(inspector, key)
    }

    pub(in crate::physical_runtime) fn released_head_capacity_charge(
        &self,
    ) -> Option<crate::physical_runtime::durability::ReleaseHeadCapacityCharge> {
        self.parts.publication.released_head_capacity_charge()
    }

    pub(in crate::physical_runtime) fn capture_released_reclaim_candidate(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
        completed: &crate::physical_runtime::CompletedPhysicalMutation,
    ) -> Result<PhysicalRecordReader, crate::physical_runtime::blob::reclaim::BlobReclaimFailure>
    {
        let (root, protection) = self
            .parts
            .publication
            .capture_selected_released_drop_candidate(attempt, completed)
            .map_err(|_| crate::physical_runtime::blob::reclaim::BlobReclaimFailure::FenceLost)?;
        Ok(self.reader_from_captured_root(root, protection))
    }

    pub(in crate::physical_runtime) fn reserve_release_certificate_capacity(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
        key: worth_store_physical_format::ReleaseCustodyHeadKeyV1,
        needed_records: u16,
        worst_case_encoded_bytes: u32,
        head_charge: crate::physical_runtime::durability::ReleaseHeadCapacityCharge,
    ) -> Result<
        crate::physical_runtime::durability::ReleaseCertificateCapacityLease,
        crate::physical_runtime::durability::ReleaseCertificateCapacityDenial,
    > {
        self.parts.publication.reserve_release_certificate_capacity(
            attempt,
            key,
            needed_records,
            worst_case_encoded_bytes,
            head_charge,
        )
    }

    pub(in crate::physical_runtime) fn commit_selected_release_certificate(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
        selected: &crate::physical_runtime::blob::reclaim::released::SelectedReleasedDescriptorObservation,
        completed: &crate::physical_runtime::CompletedPhysicalMutation,
    ) -> Result<(), crate::physical_runtime::durability::ReleaseCertificateCapacityDenial> {
        self.parts
            .publication
            .commit_selected_release_certificate(attempt, selected, completed)
    }

    pub(in crate::physical_runtime) fn capture_released_reclaim_source(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
    ) -> Result<
        (PhysicalRecordReader, [u8; 32], [u8; 32]),
        crate::physical_runtime::blob::reclaim::BlobReclaimFailure,
    > {
        let (root, protection, free_space) = self
            .parts
            .publication
            .capture_released_drop_source(attempt)
            .map_err(|_| crate::physical_runtime::blob::reclaim::BlobReclaimFailure::FenceLost)?;
        let declaration = self.parts.format.declaration();
        let root_sha256 = Sha256::digest(root.encode(declaration)).into();
        let free_sha256 = Sha256::digest(free_space.encode(declaration)).into();
        Ok((
            self.reader_from_captured_root(root, protection),
            root_sha256,
            free_sha256,
        ))
    }

    pub(in crate::physical_runtime) fn discover_recovered_original_drop_no_durable_effect(
        &self,
        reservation_record: worth_store_physical_format::PersistedRecordIdentity,
        reservation_sha256: [u8; 32],
        reservation: worth_store_physical_format::OriginalDropReservedV1,
        selected_root: worth_store_physical_format::RootPublicationCell,
    ) -> Option<crate::physical_runtime::durability::PhysicalRecoveredOriginalDropNoDurableEffect>
    {
        self.parts
            .publication
            .discover_recovered_original_drop_no_durable_effect(
                reservation_record,
                reservation_sha256,
                reservation,
                selected_root,
            )
    }

    pub(in crate::physical_runtime) fn original_drop_binding_absent(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
    ) -> Option<bool> {
        self.parts
            .publication
            .original_drop_binding_absent(store, attempt)
    }

    pub(in crate::physical_runtime) fn discover_blob_manifest_residue_completed(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
        idempotency: [u8; 32],
        fingerprint: [u8; 32],
    ) -> Option<crate::physical_runtime::durability::PhysicalOriginalDropCompleted> {
        self.parts.publication.discover_manifest_residue_completed(
            store,
            attempt,
            idempotency,
            fingerprint,
        )
    }

    pub(in crate::physical_runtime) fn discover_blob_manifest_residue_no_effect(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
    ) -> Option<crate::physical_runtime::durability::PhysicalOriginalDropNoEffect> {
        self.parts
            .publication
            .discover_manifest_residue_no_effect(store, attempt)
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::physical_runtime) fn admit_blob_manifest_residue(
        &self,
        claim: &mut PhysicalBlobSessionClaim,
        reader: PhysicalRecordReader,
        allocation: &crate::physical_runtime::BlobPhysicalAllocation<'_>,
        basis: worth_store_physical_format::FailedIngestReclaimBasisV1,
        manifest: worth_store_physical_format::PersistedRecordIdentity,
        manifest_sha256: [u8; 32],
        manifest_count: u16,
        proof: crate::physical_runtime::durability::SelectedOriginalDropProof,
        placement: crate::physical_runtime::AdmittedRecordPlacementPolicy,
    ) -> Result<
        crate::physical_runtime::durability::AdmittedManifestResidueRetirement,
        crate::physical_runtime::durability::PhysicalBlobReclaimAdmissionDenial,
    > {
        self.parts.publication.admit_manifest_residue_cleanup(
            claim,
            reader,
            allocation,
            basis,
            manifest,
            manifest_sha256,
            manifest_count,
            proof,
            placement,
        )
    }

    pub(in crate::physical_runtime) fn publish_blob_manifest_residue(
        &self,
        admitted: crate::physical_runtime::durability::AdmittedManifestResidueRetirement,
    ) -> Result<
        (
            u64,
            crate::physical_runtime::durability::ManifestResidueDisplacement,
        ),
        crate::physical_runtime::PhysicalRetirementDenial,
    > {
        self.parts
            .publication
            .publish_manifest_residue_cleanup(admitted)
    }

    pub(in crate::physical_runtime) fn claimed_blob_reclaim_reader(
        &self,
        session: BlobSessionId,
    ) -> Result<
        (
            PhysicalRecordReader,
            PhysicalBlobSessionClaim,
            Option<crate::physical_runtime::durability::CompletedDurableCheckpointWitness>,
        ),
        PhysicalBlobSessionClaimDenial,
    > {
        self.claimed_blob_reader_with_checkpoint(session)
    }

    pub(in crate::physical_runtime) fn admit_blob_reclaim_drop(
        &self,
        selected: crate::physical_runtime::blob::reclaim::selection::SelectedFailedBlobResidue,
        claim: &mut PhysicalBlobSessionClaim,
        allocation: &crate::physical_runtime::BlobPhysicalAllocation<'_>,
    ) -> Result<
        crate::physical_runtime::durability::AdmittedFailedIngestDrop,
        crate::physical_runtime::durability::PhysicalBlobReclaimAdmissionDenial,
    > {
        use crate::physical_runtime::durability::PhysicalBlobReclaimAdmissionDenial as Denial;
        let (reader, basis, dropped, remaining, occupied_attempts, recovered_reservations) =
            selected.into_reclaim_parts();
        let displaced = self
            .parts
            .publication
            .resolve_reclaim_extents(&reader, allocation, &dropped)?;
        let attempt = self.parts.publication.admit_blob_reclaim(
            claim,
            reader.protected_root(),
            &displaced,
            &occupied_attempts,
            recovered_reservations,
        )?;
        crate::physical_runtime::durability::AdmittedFailedIngestDrop::new(
            reader, attempt, basis, dropped, displaced, remaining,
        )
        .ok_or(Denial::SelectedResidueInvalid)
    }

    pub(in crate::physical_runtime) fn admit_released_blob_reclaim_drop(
        &self,
        selected: crate::physical_runtime::blob::reclaim::released::SelectedReleasedBlob,
        claim: &mut PhysicalBlobSessionClaim,
        allocation: &crate::physical_runtime::BlobPhysicalAllocation<'_>,
    ) -> Result<
        crate::physical_runtime::durability::AdmittedReleasedGenerationDrop,
        crate::physical_runtime::durability::PhysicalBlobReclaimAdmissionDenial,
    > {
        use crate::physical_runtime::durability::PhysicalBlobReclaimAdmissionDenial as Denial;
        let (
            reader,
            basis,
            dropped,
            remaining,
            occupied_attempts,
            recovered_reservations,
            predecessor,
            cumulative_dropped,
            terminal,
        ) = selected.into_reclaim_parts();
        let displaced = self
            .parts
            .publication
            .resolve_reclaim_extents(&reader, allocation, &dropped)?;
        let attempt = self.parts.publication.admit_blob_reclaim(
            claim,
            reader.protected_root(),
            &displaced,
            &occupied_attempts,
            recovered_reservations,
        )?;
        let admitted = crate::physical_runtime::durability::AdmittedReleasedGenerationDrop::new(
            reader,
            attempt,
            basis,
            dropped,
            displaced,
            remaining,
            predecessor,
            cumulative_dropped,
            terminal,
        )
        .ok_or(Denial::SelectedResidueInvalid)?;
        self.parts
            .publication
            .require_release_certificate_for_attempt(admitted.attempt())
            .map_err(|cause| match cause {
                crate::physical_runtime::durability::ReleaseCertificateCapacityDenial::Resident(
                    cause,
                ) => Denial::ReleaseCertificateBacking(cause),
                _ => Denial::AlreadyFenced,
            })?;
        Ok(admitted)
    }

    pub(in crate::physical_runtime) fn promote_blob_terminal(
        &self,
        claim: &mut PhysicalBlobSessionClaim,
    ) -> Result<(), PhysicalBlobTerminalAdmissionDenial> {
        self.parts.publication.promote_blob_terminal(claim)
    }

    /// The checkpoint witness is checked on both sides of the atomic
    /// claim/root capture. A change cannot be treated as a stable expiry
    /// decision, even if the selected scan itself takes a long time.
    pub(in crate::physical_runtime) fn claimed_blob_reader(
        &self,
        session: BlobSessionId,
    ) -> Result<(PhysicalRecordReader, PhysicalBlobSessionClaim, u64), PhysicalBlobSessionClaimDenial>
    {
        let (reader, claim, witness) = self.claimed_blob_reader_with_checkpoint(session)?;
        Ok((
            reader,
            claim,
            witness.map_or(0, |value| value.sequence().get()),
        ))
    }

    pub(in crate::physical_runtime) fn claimed_blob_reader_with_checkpoint(
        &self,
        session: BlobSessionId,
    ) -> Result<
        (
            PhysicalRecordReader,
            PhysicalBlobSessionClaim,
            Option<crate::physical_runtime::durability::CompletedDurableCheckpointWitness>,
        ),
        PhysicalBlobSessionClaimDenial,
    > {
        for _ in 0..2 {
            let before = self.selected_completed_checkpoint();
            let (root, protection, claim) =
                self.parts.publication.capture_blob_inspection(session)?;
            let after = self.selected_completed_checkpoint();
            if before == after {
                return Ok((
                    self.reader_from_captured_root(root, protection),
                    claim,
                    after,
                ));
            }
            drop(claim);
            drop(protection);
        }
        Err(PhysicalBlobSessionClaimDenial::CheckpointChanged)
    }
}
