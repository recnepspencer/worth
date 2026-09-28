use super::super::{selected_segment_rewrite::damaged, RecordPublicationDirector};
use super::{
    session::{CopyPhase, ExtentCopySession},
    PhysicalExtentCopyProgress,
};
use crate::physical_runtime::record_serving::{
    arena::ArenaReservation, RecordAppendDenial, RecordAppendError,
};
use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
};
use sha2::{Digest, Sha256};
use worth_proof::TransitionOutcome;
use worth_store_physical_format::DurableExtentRecordPlacement;

impl RecordPublicationDirector {
    pub(in crate::physical_runtime::record_serving::publication::director) fn begin_extent_copy(
        &self,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
        selected: DurableExtentRecordPlacement,
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
        let (_, free) = self.root_owner.snapshot();
        let owner = self.arena_allocation_owner(&allocation, placement, &free)?;
        let reservation = ArenaReservation::reserve(&owner, selected.arena_range().length())
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
            placement,
            prepared: prepared.mark_extent_copy(root.generation(), selected),
            operation,
            source_root: root.generation(),
            source: selected,
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
