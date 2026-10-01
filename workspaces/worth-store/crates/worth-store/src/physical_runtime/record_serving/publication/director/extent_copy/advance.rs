use super::super::{
    selected_segment_rewrite::damaged, PhysicalRecordSubmission, RecordPublicationDirector,
};
use super::{
    evidence::CopyWriteAccumulator,
    session::{CopyPhase, ExtentCopySession},
    CompletedExtentCopy,
};
use crate::physical_runtime::record_serving::{RecordAppendDenial, RecordAppendError};
use sha2::{Digest, Sha256};
use worth_store_physical_format::PhysicalExtentCopyIntent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalExtentCopyPhase {
    Hashing,
    DurableIntent,
    Copying,
    Manifest,
    Synchronizing,
    Verifying,
    ReadyForAdoption,
    InspectionRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalExtentCopyProgress {
    pub operation: [u8; 32],
    pub phase: PhysicalExtentCopyPhase,
    pub completed_payload_bytes: u64,
    pub logical_bytes: u64,
}

impl ExtentCopySession {
    pub(super) fn progress(&self) -> PhysicalExtentCopyProgress {
        PhysicalExtentCopyProgress {
            operation: self.operation,
            phase: match self.phase {
                CopyPhase::Hashing => PhysicalExtentCopyPhase::Hashing,
                CopyPhase::Intent => PhysicalExtentCopyPhase::DurableIntent,
                CopyPhase::Copying => PhysicalExtentCopyPhase::Copying,
                CopyPhase::Manifest => PhysicalExtentCopyPhase::Manifest,
                CopyPhase::Synchronizing => PhysicalExtentCopyPhase::Synchronizing,
                CopyPhase::Verifying => PhysicalExtentCopyPhase::Verifying,
                CopyPhase::Complete => PhysicalExtentCopyPhase::ReadyForAdoption,
                CopyPhase::InspectionRequired => PhysicalExtentCopyPhase::InspectionRequired,
            },
            completed_payload_bytes: self.completed,
            logical_bytes: self.source.payload_bytes(),
        }
    }
}

impl PhysicalRecordSubmission {
    /// Advances at most one admitted source/destination frame. The director,
    /// not this borrowed poll, retains every unresolved copy obligation.
    pub fn advance_extent_copy(&self) -> Result<PhysicalExtentCopyProgress, RecordAppendError> {
        self.director
            .upgrade()
            .ok_or(RecordAppendError::Denied(
                RecordAppendDenial::PublicationAuthorityReleased,
            ))?
            .advance_extent_copy()
    }
}

impl RecordPublicationDirector {
    fn advance_extent_copy(&self) -> Result<PhysicalExtentCopyProgress, RecordAppendError> {
        let mut slot = self.extent_copy.lock().unwrap_or_else(|e| e.into_inner());
        let session = slot.as_mut().ok_or_else(damaged)?;
        {
            let obligation = self
                .copy_obligation
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let state = obligation
                .as_ref()
                .ok_or_else(damaged)?
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if state.resolution_requested.is_some() || state.inspection {
                return Err(damaged());
            }
        }
        session.source_lease.require_live().map_err(|_| damaged())?;
        match session.phase {
            CopyPhase::Hashing => {
                if session
                    .hash_quantum()
                    .map_err(super::source::source_failure)?
                {
                    let digest = session.digest.clone().finalize().into();
                    session.intent = Some(
                        PhysicalExtentCopyIntent::new_with_target_tier(
                            self.format.declaration(),
                            session.operation,
                            session.source_root,
                            session.source,
                            session.reservation.range(),
                            session.cursor.manifest().alignment(),
                            digest,
                            session.target_tier,
                        )
                        .ok_or_else(damaged)?,
                    );
                    session.phase = CopyPhase::Intent;
                }
            }
            CopyPhase::Intent => {
                // Unrelated append publications do not invalidate the source hash.
                let (current, _) = self.root_owner.snapshot();
                if !session.intent_escaped
                    && self.current_extent_source(&current, session.source.record())?
                        != session.source
                {
                    return Err(RecordAppendError::Denied(
                        RecordAppendDenial::RewriteSpanNotLive,
                    ));
                }
                let payload = session.intent_payload();
                if session.durable.is_none() {
                    session.reservation.expose_to_wal();
                    let durable = self.append_copy_intent(session, &payload)?;
                    let expected: [u8; 32] = Sha256::digest(&payload).into();
                    if durable.payload_digest() != expected {
                        return Err(damaged());
                    }
                    session.durable = Some(durable);
                }
                session.cursor = self.copy_source_cursor(session.source, &session.allocation)?;
                session.writes = Some(CopyWriteAccumulator::new(
                    session.intent.unwrap(),
                    session.durable.as_ref().unwrap().interval().2,
                ));
                session.completed = 0;
                session.digest = Sha256::new();
                session.phase = CopyPhase::Copying;
            }
            CopyPhase::Copying => self.copy_frame_quantum(session)?,
            CopyPhase::Manifest => self.copy_manifest_quantum(session)?,
            CopyPhase::Synchronizing => self.synchronize_extent_copy(session)?,
            CopyPhase::Verifying => self.verify_copy_quantum(session)?,
            CopyPhase::Complete => {}
            CopyPhase::InspectionRequired => return Err(damaged()),
        }
        Ok(session.progress())
    }

    fn append_copy_intent(
        &self,
        session: &mut ExtentCopySession,
        payload: &[u8],
    ) -> Result<crate::physical_runtime::durability::DurableMaintenanceReceipt, RecordAppendError>
    {
        use crate::physical_runtime::durability::ScheduledMaintenanceDenial;
        match self.wal.append_scheduled_maintenance_receipt(payload) {
            Ok(receipt) => {
                session.intent_escaped = true;
                self.retain_copy_intent(session, receipt.interval().2)?;
                Ok(receipt)
            }
            Err(ScheduledMaintenanceDenial::NotStarted(_)) => {
                if !session.intent_escaped {
                    session.reservation.wal_proven_no_effect();
                }
                Err(RecordAppendError::Denied(
                    RecordAppendDenial::PhysicalPressure,
                ))
            }
            Err(ScheduledMaintenanceDenial::WrittenAwaitingBarrier { interval }) => {
                session.intent_escaped = true;
                self.retain_copy_intent(session, interval.2)?;
                Err(RecordAppendError::Denied(
                    RecordAppendDenial::PhysicalPressure,
                ))
            }
            Err(_) => {
                session.intent_escaped = true;
                session.phase = CopyPhase::InspectionRequired;
                if let Some(runtime) = self.runtime.upgrade() {
                    runtime.health.revoke();
                }
                Err(damaged())
            }
        }
    }

    fn retain_copy_intent(
        &self,
        session: &ExtentCopySession,
        lsn: u64,
    ) -> Result<(), RecordAppendError> {
        let slot = self
            .copy_obligation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut obligation = slot
            .as_ref()
            .ok_or_else(damaged)?
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let intent = session.intent.ok_or_else(damaged)?;
        let digest: [u8; 32] = Sha256::digest(session.intent_payload()).into();
        let binding = (intent, lsn, digest);
        if obligation.operation != intent.operation()
            || obligation.intent.is_some_and(|prior| prior != binding)
        {
            return Err(damaged());
        }
        obligation.intent = Some(binding);
        Ok(())
    }

    pub(in crate::physical_runtime::record_serving::publication::director) fn take_completed_extent_copy(
        &self,
    ) -> Result<
        (
            crate::physical_runtime::PreparedPhysicalMutation,
            CompletedExtentCopy,
        ),
        RecordAppendError,
    > {
        let mut slot = self.extent_copy.lock().unwrap_or_else(|e| e.into_inner());
        if slot
            .as_ref()
            .is_none_or(|copy| copy.phase != CopyPhase::Complete)
        {
            return Err(damaged());
        }
        let copy = slot
            .take()
            .expect("complete session was checked under its mutex");
        let obligation_slot = self
            .copy_obligation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let obligation = super::obligation::CopyCapabilityHold::issue(
            obligation_slot.as_ref().ok_or_else(damaged)?,
        )
        .map_err(|_| damaged())?;
        let intent = copy.intent.expect("completed copy has durable intent");
        let writes = copy
            .writes
            .expect("completed copy has receipts")
            .finish(intent.chunk_count() + 1)
            .map_err(|_| damaged())?;
        Ok((
            copy.prepared,
            CompletedExtentCopy {
                intent,
                durable: copy.durable.expect("completed copy retains WAL receipt"),
                writes,
                synchronization: copy
                    .synchronization
                    .expect("complete follows synchronization"),
                source_lease: copy.source_lease,
                reservation: copy.reservation,
                obligation,
            },
        ))
    }
}
