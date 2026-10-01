use super::*;

impl RecordPublicationDirector {
    #[allow(clippy::too_many_arguments)]
    fn matches_original_drop_fingerprint(
        &self,
        basis: worth_store_physical_format::FailedIngestReclaimBasisV1,
        manifest: worth_store_physical_format::PersistedRecordIdentity,
        manifest_sha256: [u8; 32],
        manifest_count: u16,
        original: crate::physical_runtime::durability::PhysicalOriginalDropNoEffect,
        descriptor_source_generation: u64,
        placement: AdmittedRecordPlacementPolicy,
    ) -> bool {
        let Some(candidate) = descriptor_source_generation.checked_add(1) else {
            return false;
        };
        let Ok(descriptor) = BlobReclaimDescriptorV1::new(
            original.store(),
            original.attempt(),
            basis.digest(original.store()),
            manifest,
            manifest_sha256,
            manifest_count,
            descriptor_source_generation,
            candidate,
        ) else {
            return false;
        };
        self.expected_original_drop_fingerprint(descriptor, placement)
            .is_some_and(|expected| expected == original.request_fingerprint())
    }

    pub(in crate::physical_runtime) fn discover_manifest_residue_no_effect(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
    ) -> Option<crate::physical_runtime::durability::PhysicalOriginalDropNoEffect> {
        self.idempotency
            .discover_original_drop_no_effect(store, attempt)
    }

    pub(in crate::physical_runtime) fn original_drop_binding_absent(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
    ) -> Option<bool> {
        self.idempotency
            .original_drop_binding_absent(store, attempt)
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::physical_runtime) fn admit_manifest_residue_cleanup(
        &self,
        claim: &mut crate::physical_runtime::durability::PhysicalBlobSessionClaim,
        reader: crate::physical_runtime::PhysicalRecordReader,
        allocation: &crate::physical_runtime::BlobPhysicalAllocation<'_>,
        basis: worth_store_physical_format::FailedIngestReclaimBasisV1,
        manifest: worth_store_physical_format::PersistedRecordIdentity,
        manifest_sha256: [u8; 32],
        manifest_count: u16,
        selected_proof: SelectedOriginalDropProof,
        placement: crate::physical_runtime::AdmittedRecordPlacementPolicy,
    ) -> Result<
        AdmittedManifestResidueRetirement,
        crate::physical_runtime::durability::PhysicalBlobReclaimAdmissionDenial,
    > {
        use crate::physical_runtime::durability::PhysicalBlobReclaimAdmissionDenial as Denial;
        let store = reader.store_identity().bytes();
        let selected_root = reader.protected_root().root();
        let proof = match selected_proof {
            SelectedOriginalDropProof::V1(original)
            | SelectedOriginalDropProof::V2ProvenNoEffect {
                original,
                reserved: _,
                reserved_selected_generation: _,
            } if original.store() != store => return Err(Denial::SelectedResidueInvalid),
            SelectedOriginalDropProof::V1(original) => self.reconciled_proof(
                &reader,
                basis,
                manifest,
                manifest_sha256,
                manifest_count,
                original,
                selected_root.generation().get(),
                placement,
                None,
            )?,
            SelectedOriginalDropProof::V2NeverReserved(value) => {
                ManifestResidueProof::never_reserved(
                    &value,
                    manifest,
                    manifest_sha256,
                    selected_root,
                )
                .filter(|_| {
                    value.store() == store
                        && value.source_basis() == basis
                        && value.count() == manifest_count
                        && value.never_reserved_slot_generation()
                            <= selected_root.generation().get()
                })
                .ok_or(Denial::SelectedResidueInvalid)?
            }
            SelectedOriginalDropProof::V2ProvenNoEffect {
                original,
                reserved,
                reserved_selected_generation,
            } => self.reconciled_proof(
                &reader,
                basis,
                manifest,
                manifest_sha256,
                manifest_count,
                original,
                reserved_selected_generation,
                placement,
                Some(reserved),
            )?,
            SelectedOriginalDropProof::V2RecoveredNoBinding {
                recovered,
                reserved,
            } => {
                let original = recovered.reservation();
                let descriptor = BlobReclaimDescriptorV1::new(
                    store,
                    original.reclaim_attempt(),
                    basis.digest(store),
                    manifest,
                    manifest_sha256,
                    manifest_count,
                    original.reserved_selected_generation(),
                    original
                        .reserved_selected_generation()
                        .checked_add(1)
                        .ok_or(Denial::SelectedResidueInvalid)?,
                )
                .map_err(|_| Denial::SelectedResidueInvalid)?;
                if original.store() != store
                    || original.manifest_record() != manifest
                    || original.manifest_frame_sha256() != manifest_sha256
                    || original.source_basis_digest() != basis.digest(store)
                    || recovered.reservation_record() != reserved.record()
                    || recovered.reservation_sha256() != reserved.frame_sha256()
                    || recovered.selected_root() != selected_root
                    || self.expected_original_drop_fingerprint(descriptor, placement)
                        != Some(original.request().fingerprint())
                {
                    return Err(Denial::SelectedResidueInvalid);
                }
                ManifestResidueProof::RecoveredNoBinding {
                    recovered,
                    reserved,
                }
            }
        };
        let manifest_extent = self.resolve_reclaim_extents(&reader, allocation, &[manifest])?;
        let [manifest_extent] = manifest_extent.as_slice() else {
            return Err(Denial::SelectedResidueInvalid);
        };
        let reserved_extent = if let Some(binding) = proof.reserved() {
            let extent = self.resolve_reclaim_extents(&reader, allocation, &[binding.record()])?;
            let [extent] = extent.as_slice() else {
                return Err(Denial::SelectedResidueInvalid);
            };
            Some(*extent)
        } else {
            None
        };
        let displaced = ManifestResidueDisplacement {
            manifest: *manifest_extent,
            reserved: reserved_extent,
        };
        self.root_owner.admit_manifest_residue_retirement(
            claim,
            reader,
            basis,
            manifest,
            manifest_sha256,
            displaced,
            proof,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn reconciled_proof(
        &self,
        reader: &crate::physical_runtime::PhysicalRecordReader,
        basis: worth_store_physical_format::FailedIngestReclaimBasisV1,
        manifest: worth_store_physical_format::PersistedRecordIdentity,
        sha256: [u8; 32],
        count: u16,
        original: crate::physical_runtime::durability::PhysicalOriginalDropNoEffect,
        descriptor_source_generation: u64,
        placement: AdmittedRecordPlacementPolicy,
        reserved: Option<worth_store_physical_format::ReservedDropRecordV1>,
    ) -> Result<
        ManifestResidueProof,
        crate::physical_runtime::durability::PhysicalBlobReclaimAdmissionDenial,
    > {
        use crate::physical_runtime::durability::PhysicalBlobReclaimAdmissionDenial as Denial;
        if !self.matches_original_drop_fingerprint(
            basis,
            manifest,
            sha256,
            count,
            original,
            descriptor_source_generation,
            placement,
        ) {
            return Err(Denial::SelectedResidueInvalid);
        }
        let fate = crate::physical_runtime::durability::PhysicalReconciledReclaimDescriptorFate::from_original_drop_no_effect(
            original, basis, manifest, sha256, reader.protected_root().root(),
        ).ok_or(Denial::SelectedResidueInvalid)?;
        Ok(ManifestResidueProof::positive(fate, reserved))
    }
}
