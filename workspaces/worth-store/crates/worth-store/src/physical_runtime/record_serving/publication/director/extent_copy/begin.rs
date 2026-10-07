use super::super::{selected_segment_rewrite::damaged, RecordPublicationDirector};
use super::{
    session::{CopyPhase, CopyProducer, ExtentCopySession},
    PhysicalExtentCopyProgress,
};
use crate::physical_runtime::record_serving::{
    arena::{ArenaEvacuationLease, ArenaReservation},
    RecordAppendDenial, RecordAppendError,
};
use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
};
use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    arena_tier_at_epoch, DurableExtentRecordPlacement, PhysicalTierClass,
};

impl RecordPublicationDirector {
    pub(in crate::physical_runtime::record_serving::publication::director) fn begin_extent_copy(
        &self,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
        selected: DurableExtentRecordPlacement,
        producer: CopyProducer,
    ) -> Result<
        Result<PhysicalExtentCopyProgress, PhysicalMutationPreparationOutcome>,
        RecordAppendError,
    > {
        self.begin_extent_copy_to_tier(
            placement,
            request,
            selected,
            selected.tier_class(),
            producer,
        )
    }

    fn begin_extent_copy_to_tier(
        &self,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
        selected: DurableExtentRecordPlacement,
        target_tier: PhysicalTierClass,
        producer: CopyProducer,
    ) -> Result<
        Result<PhysicalExtentCopyProgress, PhysicalMutationPreparationOutcome>,
        RecordAppendError,
    > {
        let mut slot = self.extent_copy.lock().unwrap_or_else(|e| e.into_inner());
        if slot.is_some() {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::PhysicalPressure,
            ));
        }
        let mut obligation_slot = self
            .copy_obligation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if obligation_slot.is_some() {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::PhysicalPressure,
            ));
        }
        let operation = request.idempotency_key().identity().bytes();
        let allocation = self.copy_allocation()?;
        let (root, lease) = self.root_owner.capture_read_root().map_err(|_| damaged())?;
        if self.current_extent_source(&root, selected.record())? != selected {
            return Err(damaged());
        }
        let cursor = self.copy_source_cursor(selected, &allocation)?;
        let (current, free) = self.root_owner.snapshot();
        if arena_tier_at_epoch(free.tier_epoch_start(), selected.arena_range().arena())
            != selected.tier_class()
        {
            return Err(damaged());
        }
        if target_tier != selected.tier_class()
            && (root.tier_epoch_anchor().is_none()
                || free.tier_epoch_start().is_none()
                || current.root_cell() != root.root_cell()
                || free.generation() != root.generation()
                || free.tree_identity() != root.tree_identity())
        {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::BlobMovementUnsupportedTier,
            ));
        }
        let owner = self.arena_allocation_owner(&allocation, placement, &free)?;
        let reservation =
            ArenaReservation::reserve_in_tier(&owner, selected.arena_range().length(), target_tier)
                .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::PhysicalPressure))?;
        if reservation.range().arena() == selected.arena_range().arena() {
            return Err(damaged());
        }
        let growth = self
            .root_owner
            .publication_admission()
            .reserve_retained_bytes(reservation.range().length())
            .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::RetentionPressure))?;
        let prepared = match self
            .prepare_extent_copy_request(placement, request, selected.record())
            .into_raw()
        {
            TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
                prepared
            }
            other => return Ok(Err(other.into())),
        };
        if prepared.extent_rewrite_source() != Some(selected) {
            return Err(damaged());
        }
        let binding =
            crate::physical_runtime::durability::PhysicalMutationUnresolvedBindingObservation::new(
                prepared.idempotency_identity(),
                prepared.request_fingerprint(),
                prepared.mutation_identity(),
            );
        *obligation_slot = Some(std::sync::Arc::new(std::sync::Mutex::new(
            super::obligation::CopyObligation {
                operation,
                source_lease: lease.clone(),
                reservation: std::sync::Arc::new(std::sync::Mutex::new(
                    super::obligation::CopyDestination::Reserved(reservation.retain_obligation()),
                )),
                physical_growth: Some(growth),
                binding: super::obligation::CopyBinding::Live(binding),
                intent: None,
                carrier_alive: false,
                publication_lsn: None,
                resolved: false,
                published_root: None,
                resolution: None,
                inspection: false,
                resolution_requested: None,
            },
        )));
        let session = ExtentCopySession {
            producer,
            prepared: prepared.mark_extent_copy(root.generation(), selected),
            operation,
            source_root: root.generation(),
            source: selected,
            target_tier,
            source_lease: lease,
            reservation,
            allocation,
            cursor,
            digest: Sha256::new(),
            completed: 0,
            intent: None,
            durable: None,
            writes: None,
            next_ordinal: 1,
            pending: None,
            synchronization: None,
            verified_membership: None,
            intent_escaped: false,
            phase: CopyPhase::Hashing,
        };
        let progress = session.progress();
        *slot = Some(session);
        Ok(Ok(progress))
    }
}

impl super::super::PhysicalRecordSubmission {
    /// Admission is reachable only with a Store-authenticated blob tree edge
    /// and its retained protected read. A bare RecordId cannot start movement.
    pub(in crate::physical_runtime) fn begin_selected_blob_chunk_copy(
        &self,
        hold: &crate::physical_runtime::BlobMovementReadHold<'_>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> Result<
        Result<PhysicalExtentCopyProgress, PhysicalMutationPreparationOutcome>,
        RecordAppendError,
    > {
        self.begin_selected_blob_chunk_copy_inner(hold, placement, request, None)
    }

    /// Store-only tier target. No caller-facing target authority is exposed
    /// until C8 has installed a verified epoch/custody basis on reopen.
    pub(in crate::physical_runtime) fn begin_selected_blob_chunk_copy_in_tier(
        &self,
        hold: &crate::physical_runtime::BlobMovementReadHold<'_>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
        target_tier: PhysicalTierClass,
    ) -> Result<
        Result<PhysicalExtentCopyProgress, PhysicalMutationPreparationOutcome>,
        RecordAppendError,
    > {
        self.begin_selected_blob_chunk_copy_inner(hold, placement, request, Some(target_tier))
    }

    fn begin_selected_blob_chunk_copy_inner(
        &self,
        hold: &crate::physical_runtime::BlobMovementReadHold<'_>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
        target_tier: Option<PhysicalTierClass>,
    ) -> Result<
        Result<PhysicalExtentCopyProgress, PhysicalMutationPreparationOutcome>,
        RecordAppendError,
    > {
        let director = self.director.upgrade().ok_or_else(damaged)?;
        if !placement.admits(director.format) {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::PlacementFormatMismatch,
            ));
        }
        let (root, _) = director.root_owner.snapshot();
        if root.generation() != hold.selected_root_generation() {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::RewriteSpanNotLive,
            ));
        }
        let selected = director.current_extent_source(&root, hold.selected_record())?;
        if selected.content_class()
            != worth_store_physical_format::SelectedRecordContentClass::Blob(
                worth_store_physical_format::BlobRecordKind::Chunk,
            )
        {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::BlobMovementSourceClassMismatch,
            ));
        }
        if target_tier.is_none() && selected.tier_class() != PhysicalTierClass::Primary {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::BlobMovementUnsupportedTier,
            ));
        }
        let target_tier = target_tier.unwrap_or(selected.tier_class());
        // Keep the source arena out of destination selection. Once begin has
        // reserved a distinct range, the copy's own root lease and reservation
        // retain both sides for its full lifecycle.
        let allocation = director.copy_allocation()?;
        let (selected_root, free) = director.root_owner.snapshot();
        if (selected.tier_class() != PhysicalTierClass::Primary
            || target_tier != PhysicalTierClass::Primary)
            && (selected_root.tier_epoch_anchor().is_none()
                || free.tier_epoch_start().is_none()
                || selected_root.root_cell() != root.root_cell())
        {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::BlobMovementUnsupportedTier,
            ));
        }
        let owner = director.arena_allocation_owner(&allocation, placement, &free)?;
        let _exclusion = ArenaEvacuationLease::acquire(&owner, selected.arena_range().arena())
            .map_err(|_| RecordAppendError::Denied(RecordAppendDenial::PhysicalPressure))?;
        director.begin_extent_copy_to_tier(
            placement,
            request,
            selected,
            target_tier,
            CopyProducer::BlobMovement,
        )
    }
}
