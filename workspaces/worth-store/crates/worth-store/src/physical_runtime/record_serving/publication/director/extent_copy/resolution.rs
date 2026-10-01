use super::super::{
    selected_segment_rewrite::damaged, PhysicalRecordSubmission, RecordPublicationDirector,
};
use super::obligation::CopyObligation;
use crate::physical_runtime::durability::ScheduledMaintenanceDenial;
use crate::physical_runtime::record_serving::{RecordAppendDenial, RecordAppendError};
use crate::physical_runtime::{CompletedPhysicalCheckpoint, PhysicalMutationProvenNoEffectCause};
use worth_store_physical_format::{
    PhysicalExtentCopyRecord, PhysicalExtentCopyResolution,
    PhysicalExtentCopyResolutionKind as Kind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalExtentCopyResolutionProgress {
    AwaitingCheckpoint { resolution_lsn: u64 },
    Resolved,
}

impl PhysicalRecordSubmission {
    /// Abandons a quiescent copy only through its existing WAL owner. A copied
    /// destination remains reserved until finalize observes checkpoint coverage.
    pub fn cancel_extent_copy(
        &self,
    ) -> Result<PhysicalExtentCopyResolutionProgress, RecordAppendError> {
        self.director
            .upgrade()
            .ok_or_else(damaged)?
            .cancel_extent_copy()
    }
    pub fn finalize_extent_copy(
        &self,
        checkpoint: &CompletedPhysicalCheckpoint,
    ) -> Result<PhysicalExtentCopyResolutionProgress, RecordAppendError> {
        self.director
            .upgrade()
            .ok_or_else(damaged)?
            .finalize_extent_copy(checkpoint)
    }
}

impl RecordPublicationDirector {
    pub(in crate::physical_runtime) fn observe_published_extent_copy(
        &self,
        operation: [u8; 32],
        root: u64,
        publication_lsn: u64,
    ) -> Result<(), ()> {
        let slot = self
            .copy_obligation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut state = slot
            .as_ref()
            .ok_or(())?
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if state.operation != operation
            || state.publication_lsn != Some(publication_lsn)
            || state.resolved
            || state.resolution.is_some()
        {
            return Err(());
        }
        state.published_root = Some(root);
        Ok(())
    }

    fn cancel_extent_copy(
        &self,
    ) -> Result<PhysicalExtentCopyResolutionProgress, RecordAppendError> {
        let mut session = self.extent_copy.lock().unwrap_or_else(|e| e.into_inner());
        let blob_movement = session
            .as_ref()
            .is_some_and(|copy| copy.producer == super::session::CopyProducer::BlobMovement);
        let mut slot = self
            .copy_obligation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let obligation = slot.as_ref().ok_or_else(damaged)?.clone();
        let mut state = obligation.lock().unwrap_or_else(|e| e.into_inner());
        if state.carrier_alive
            || state.publication_lsn.is_some()
            || state.inspection
            || session
                .as_ref()
                .is_some_and(|copy| copy.intent_escaped && copy.durable.is_none())
        {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::PhysicalPressure,
            ));
        }
        if state.intent.is_none() {
            self.cancel_unstarted_copy_binding(&state.binding)?;
            state
                .reservation
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .cancel_after_resolution()
                .map_err(|_| damaged())?;
            state.resolved = true;
            *session = None;
            *slot = None;
            if blob_movement {
                self.mutation.cancel_blob_movement_background();
            }
            return Ok(PhysicalExtentCopyResolutionProgress::Resolved);
        }
        let lsn = self.append_copy_resolution(&mut state, Kind::Cancelled)?;
        *session = None;
        if blob_movement {
            self.mutation.cancel_blob_movement_background();
        }
        Ok(PhysicalExtentCopyResolutionProgress::AwaitingCheckpoint {
            resolution_lsn: lsn,
        })
    }

    fn finalize_extent_copy(
        &self,
        checkpoint: &CompletedPhysicalCheckpoint,
    ) -> Result<PhysicalExtentCopyResolutionProgress, RecordAppendError> {
        let mut slot = self
            .copy_obligation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let obligation = slot.as_ref().ok_or_else(damaged)?.clone();
        let mut state = obligation.lock().unwrap_or_else(|e| e.into_inner());
        let source = checkpoint.basis().source();
        if source.identity().store_identity() != self.durability.store_identity()
            || checkpoint.basis().policy_identity() != self.durability.policy_identity()
            || state.inspection
        {
            return Err(damaged());
        }
        if state.resolution.is_none() {
            let root = state.published_root.ok_or_else(damaged)?;
            let publication_lsn = state.publication_lsn.ok_or_else(damaged)?;
            if source.root().generation() < root
                || source.wal().covered_end_lsn_exclusive() <= publication_lsn
            {
                return Err(RecordAppendError::Denied(
                    RecordAppendDenial::PhysicalPressure,
                ));
            }
            let lsn = self.append_copy_resolution(
                &mut state,
                Kind::Published {
                    root_generation: root,
                    publication_lsn,
                },
            )?;
            return Ok(PhysicalExtentCopyResolutionProgress::AwaitingCheckpoint {
                resolution_lsn: lsn,
            });
        }
        let resolution = state.resolution.as_ref().ok_or_else(damaged)?;
        if source.wal().covered_end_lsn_exclusive() < resolution.end_lsn() {
            return Ok(PhysicalExtentCopyResolutionProgress::AwaitingCheckpoint {
                resolution_lsn: resolution.start_lsn(),
            });
        }
        if resolution.kind() == Kind::Cancelled {
            self.settle_durably_cancelled_copy_binding(&state)?;
            state
                .reservation
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .cancel_after_resolution()
                .map_err(|_| damaged())?;
        }
        state.resolved = true;
        *slot = None;
        Ok(PhysicalExtentCopyResolutionProgress::Resolved)
    }

    fn append_copy_resolution(
        &self,
        state: &mut CopyObligation,
        kind: Kind,
    ) -> Result<u64, RecordAppendError> {
        if let Some(existing) = &state.resolution {
            return if existing.kind() == kind {
                Ok(existing.start_lsn())
            } else {
                Err(damaged())
            };
        }
        let (intent, lsn, digest) = state.intent.ok_or_else(damaged)?;
        if state
            .resolution_requested
            .is_some_and(|requested| requested != kind)
        {
            return Err(damaged());
        }
        state.resolution_requested = Some(kind);
        let resolution = PhysicalExtentCopyResolution::new(intent.operation(), digest, lsn, kind)
            .ok_or_else(damaged)?;
        let payload = PhysicalExtentCopyRecord::Resolved(resolution).encode();
        let receipt = match self.wal.append_scheduled_maintenance_receipt(&payload) {
            Ok(receipt) => receipt,
            Err(
                ScheduledMaintenanceDenial::NotStarted(_)
                | ScheduledMaintenanceDenial::WrittenAwaitingBarrier { .. },
            ) => {
                return Err(RecordAppendError::Denied(
                    RecordAppendDenial::PhysicalPressure,
                ))
            }
            Err(_) => {
                state.inspection = true;
                if let Some(runtime) = self.runtime.upgrade() {
                    runtime.health.revoke();
                }
                return Err(damaged());
            }
        };
        let lsn = receipt.interval().2;
        state.resolution = Some(super::obligation::CopyResolution::Live(kind, receipt));
        Ok(lsn)
    }

    fn cancel_unstarted_copy_binding(
        &self,
        binding: &super::obligation::CopyBinding,
    ) -> Result<(), RecordAppendError> {
        if let super::obligation::CopyBinding::Live(binding) = binding {
            self.idempotency
                .cancel_before_group_seal(
                    *binding,
                    PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal,
                )
                .map_err(|_| damaged())?;
        }
        Ok(())
    }

    /// The durable cancellation marker and covering checkpoint settle the
    /// copy's storage obligation. A final mutation attempt that stopped before
    /// its member WAL effect may already have an indeterminate idempotency
    /// terminal; preserve that terminal instead of claiming ProvenNoEffect.
    fn settle_durably_cancelled_copy_binding(
        &self,
        state: &CopyObligation,
    ) -> Result<(), RecordAppendError> {
        if state.publication_lsn.is_some() || state.published_root.is_some() {
            return Err(damaged());
        }
        let super::obligation::CopyBinding::Live(binding) = &state.binding else {
            return Ok(());
        };
        match self.idempotency.cancel_before_group_seal(
            *binding,
            PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal,
        ) {
            Ok(_) | Err(crate::physical_runtime::durability::PhysicalMutationPreSealCancellationDenial::GroupSealed) => Ok(()),
            Err(_) => Err(damaged()),
        }
    }
}
