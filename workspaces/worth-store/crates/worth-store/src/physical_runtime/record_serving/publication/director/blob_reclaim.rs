use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    BlobReclaimDescriptorV1, BlobRecordKind, CurrentPhysicalRecordPlacement, DropSetManifestV2,
    OriginalDropReservationRequestV1, OriginalDropReservedV1, PersistedRecordIdentity,
};

use super::RecordPublicationDirector;
use crate::physical_runtime::{
    durability::{
        AdmittedFailedIngestDrop, DisplacedArtifact, PhysicalBlobReclaimAdmissionDenial,
        PhysicalMutationDurabilityRequest, PhysicalMutationOperationFamily, RetiredArtifact,
    },
    record_serving::access::manifest_routing::{ManifestDiscoveryCounterSnapshot, ManifestReader},
    record_serving::{
        publication::{
            prepare_canonical_payload, PhysicalManifestCapacityTransition,
            PhysicalMutationPreparationOutcome, PhysicalMutationPreparationSuccess,
        },
        AdmittedRecordPlacementPolicy, RecordAppendBatch, RecordAppendDenial,
    },
    BlobPhysicalAllocation, PhysicalMutationIdempotencyKey, PhysicalMutationRequest,
    PhysicalRecordReader,
};

use super::durable_preparation::map_record_denial;

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn discover_recovered_original_drop_no_durable_effect(
        &self,
        reservation_record: PersistedRecordIdentity,
        reservation_sha256: [u8; 32],
        reservation: OriginalDropReservedV1,
        selected_root: worth_store_physical_format::RootPublicationCell,
    ) -> Option<crate::physical_runtime::durability::PhysicalRecoveredOriginalDropNoDurableEffect>
    {
        self.idempotency
            .discover_recovered_original_drop_no_durable_effect(
                reservation_record,
                reservation_sha256,
                reservation,
                selected_root,
            )
    }

    pub(in crate::physical_runtime) fn discover_manifest_residue_completed(
        &self,
        store: [u8; 16],
        attempt: [u8; 16],
        idempotency: [u8; 32],
        fingerprint: [u8; 32],
    ) -> Option<crate::physical_runtime::durability::PhysicalOriginalDropCompleted> {
        self.idempotency
            .discover_original_drop_completed(store, attempt, idempotency, fingerprint)
    }

    pub(in crate::physical_runtime) fn prepare_blob_reclaim_manifest(
        &self,
        admitted: &AdmittedFailedIngestDrop,
        manifest: DropSetManifestV2,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        if manifest.store() != self.durability.store_identity().bytes()
            || manifest.reclaim_attempt() != admitted.attempt().bytes()
            || manifest.source_basis() != admitted.basis()
            || manifest.dropped() != admitted.dropped()
            || admitted.attempt().expected_root().is_none_or(|root| {
                root.generation().get().checked_add(1)
                    != Some(manifest.never_reserved_slot_generation())
            })
        {
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        }
        self.prepare_fenced_reclaim_record(
            admitted.attempt(),
            manifest.encode(),
            BlobRecordKind::DropSetManifestV2,
            placement,
            request,
        )
    }

    pub(in crate::physical_runtime) fn prepare_blob_reclaim_reservation(
        &self,
        admitted: &AdmittedFailedIngestDrop,
        reserved: OriginalDropReservedV1,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        if reserved.store() != self.durability.store_identity().bytes()
            || reserved.reclaim_attempt() != admitted.attempt().bytes()
            || reserved.source_basis_digest()
                != admitted
                    .basis()
                    .digest(self.durability.store_identity().bytes())
            || admitted.attempt().expected_root().is_none_or(|root| {
                root.generation().get() != reserved.manifest_selected_generation()
                    || root.generation().get().checked_add(1)
                        != Some(reserved.reserved_selected_generation())
            })
        {
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        }
        self.prepare_fenced_reclaim_record(
            admitted.attempt(),
            reserved.encode(),
            BlobRecordKind::OriginalDropReserved,
            placement,
            request,
        )
    }

    pub(in crate::physical_runtime) fn expected_original_drop_fingerprint(
        &self,
        descriptor: BlobReclaimDescriptorV1,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Option<[u8; 32]> {
        self.expected_original_drop_fingerprint_bytes(descriptor.encode(), placement)
    }

    pub(super) fn expected_original_drop_fingerprint_bytes(
        &self,
        encoded: Vec<u8>,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Option<[u8; 32]> {
        let batch = RecordAppendBatch::builder()
            .push_owned(encoded)
            .build()
            .ok()?;
        let payload = prepare_canonical_payload(batch).ok()?;
        self.derive_record_append_fingerprint(
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            payload.digest,
            PhysicalMutationDurabilityRequest::PlatformDurable,
            PhysicalMutationOperationFamily::BlobRecordAppend,
        )
        .ok()
        .map(|fingerprint| fingerprint.bytes())
    }

    pub(in crate::physical_runtime) fn describe_original_drop_request(
        &self,
        key: &PhysicalMutationIdempotencyKey,
        descriptor: BlobReclaimDescriptorV1,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Option<OriginalDropReservationRequestV1> {
        OriginalDropReservationRequestV1::new(
            key.identity().bytes(),
            self.expected_original_drop_fingerprint(descriptor, placement)?,
            key.lease().issuance_generation().get(),
            key.lease().expiry_generation().get(),
        )
        .ok()
    }

    pub(in crate::physical_runtime) fn drop_reclaimed_records(
        &self,
        admitted: &AdmittedFailedIngestDrop,
        descriptor: BlobReclaimDescriptorV1,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(expected_root) = admitted.attempt().expected_root() else {
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        };
        if descriptor.store() != self.durability.store_identity().bytes()
            || descriptor.reclaim_attempt() != admitted.attempt().bytes()
            || descriptor.source_basis_digest()
                != admitted
                    .basis()
                    .digest(self.durability.store_identity().bytes())
            || usize::from(descriptor.manifest_count()) != admitted.dropped().len()
            || descriptor.source_root_generation() != expected_root.generation().get()
            || descriptor.candidate_root_generation()
                != expected_root.generation().get().saturating_add(1)
            || admitted.dropped().contains(&descriptor.manifest_record())
            || !admitted.attempt().register_drop_records(admitted.dropped())
        {
            return map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable);
        }
        let outcome = self.prepare_fenced_reclaim_record(
            admitted.attempt(),
            descriptor.encode(),
            BlobRecordKind::ReclaimDescriptor,
            placement,
            request,
        );
        match outcome.into_raw() {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                TransitionOutcome::success(PhysicalMutationPreparationSuccess::Prepared(prepared))
                    .into()
            }
            other => {
                admitted.attempt().clear_unprepared_drop_records();
                other.into()
            }
        }
    }

    pub(super) fn prepare_fenced_reclaim_record(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
        encoded: Vec<u8>,
        kind: BlobRecordKind,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let batch = match RecordAppendBatch::builder().push_owned(encoded).build() {
            Ok(batch) => batch,
            Err(denial) => return map_record_denial(denial),
        };
        let outcome = self.prepare_durable_append_classified(
            batch,
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            request,
            super::durable_preparation::ProtectedAppendKind::Blob(kind),
        );
        self.register_fenced_reclaim_preparation(attempt, outcome)
    }

    pub(super) fn prepare_fenced_released_descriptor(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
        descriptor: worth_store_physical_format::BlobReclaimDescriptorV3,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let outcome = self.prepare_released_descriptor_append(descriptor, placement, request);
        self.register_fenced_reclaim_preparation(attempt, outcome)
    }

    fn register_fenced_reclaim_preparation(
        &self,
        attempt: &crate::physical_runtime::durability::PhysicalReclaimAttempt,
        outcome: PhysicalMutationPreparationOutcome,
    ) -> PhysicalMutationPreparationOutcome {
        match outcome.into_raw() {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                if attempt.register_mutation(prepared.mutation_identity()) {
                    TransitionOutcome::success(PhysicalMutationPreparationSuccess::Prepared(
                        prepared,
                    ))
                    .into()
                } else {
                    let _ = self.cancel_prepared_before_group_seal(prepared);
                    map_record_denial(RecordAppendDenial::ReclaimFenceUnavailable)
                }
            }
            other => other.into(),
        }
    }

    /// Resolves exact current-root extent facts through C9 before the reclaim
    /// fence is admitted. A selected occurrence alone is never a drop route.
    pub(in crate::physical_runtime) fn resolve_reclaim_extents(
        &self,
        reader: &PhysicalRecordReader,
        allocation: &BlobPhysicalAllocation<'_>,
        records: &[PersistedRecordIdentity],
    ) -> Result<Vec<DisplacedArtifact>, PhysicalBlobReclaimAdmissionDenial> {
        if records.is_empty()
            || records.len() > 1024
            || reader.store != allocation.store_identity()
            || reader.protected_root().runtime() != allocation.runtime_identity()
            || reader.generation != allocation.store_generation()
        {
            return Err(PhysicalBlobReclaimAdmissionDenial::SelectedResidueInvalid);
        }
        let source = reader.current_root.generation();
        let routing = ManifestReader::serving(
            self.residency.clone(),
            self.format,
            self.access,
            reader.current_root.clone(),
        );
        let mut discoveries = ManifestDiscoveryCounterSnapshot::default();
        let mut displaced = Vec::new();
        displaced
            .try_reserve_exact(records.len())
            .map_err(|_| PhysicalBlobReclaimAdmissionDenial::Capacity)?;
        for record in records {
            let placement = routing
                .locate(allocation.operation_grant(), *record, &mut discoveries)
                .map_err(|_| PhysicalBlobReclaimAdmissionDenial::RouteUnavailable)?;
            let Some(CurrentPhysicalRecordPlacement::Extent(extent)) = placement else {
                return Err(PhysicalBlobReclaimAdmissionDenial::RouteUnavailable);
            };
            displaced.push(DisplacedArtifact {
                source_root: source,
                artifact: RetiredArtifact::Extent {
                    extent: extent.extent().get(),
                    generation: extent.extent_generation(),
                    range: extent.arena_range(),
                },
                // This is the same exact retained-artifact charge used by
                // native extent rewrite, including reserved arena tail.
                bytes: extent.arena_range().length(),
            });
        }
        displaced.sort_unstable_by_key(|entry| entry.artifact);
        if displaced
            .windows(2)
            .any(|pair| pair[0].artifact == pair[1].artifact)
        {
            return Err(PhysicalBlobReclaimAdmissionDenial::SelectedResidueInvalid);
        }
        Ok(displaced)
    }
}
