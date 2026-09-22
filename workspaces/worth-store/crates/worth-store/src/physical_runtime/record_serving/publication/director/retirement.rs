use worth_store_physical_format::RecordArtifactFile;

#[cfg(feature = "certification-test-authority")]
pub(super) struct RetirementIntentGate {
    armed: std::sync::atomic::AtomicBool,
    arrived: std::sync::atomic::AtomicBool,
    release: std::sync::atomic::AtomicBool,
}

#[cfg(feature = "certification-test-authority")]
impl RetirementIntentGate {
    pub(super) const fn new() -> Self {
        Self {
            armed: std::sync::atomic::AtomicBool::new(false),
            arrived: std::sync::atomic::AtomicBool::new(false),
            release: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

use super::RecordPublicationDirector;
use crate::physical_runtime::durability::{
    encode_retirement, PhysicalRootPublicationWorkFailure, ScheduledMaintenanceDenial,
    RETIREMENT_COMPLETION, RETIREMENT_INTENT,
};
use crate::physical_runtime::{
    PhysicalPublicationEffect, PhysicalRetirementDenial, PhysicalSchedulerDenial,
    PhysicalWalAppendFailureCause, PhysicalWorkEffectFate,
    PhysicalWorkSettlementEvidence,
};

impl RecordPublicationDirector {
    /// One in-flight retirement owns the claim. A concurrent caller must not
    /// append its own intent after this attempt has completed.
    pub(in crate::physical_runtime) fn try_begin_retirement(
        &self,
    ) -> Option<std::sync::MutexGuard<'_, ()>> {
        match self.retirement_owner.try_lock() {
            Ok(guard) => Some(guard),
            Err(std::sync::TryLockError::WouldBlock) => None,
            Err(std::sync::TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner()),
        }
    }

    pub(in crate::physical_runtime) fn commit_retirement_intent(
        &self,
    ) -> Result<Option<crate::physical_runtime::durability::DisplacedSegment>, PhysicalRetirementDenial>
    {
        let Some(displaced) = self.root_owner.claim_retirement()? else {
            return Ok(None);
        };
        #[cfg(feature = "certification-test-authority")]
        self.wait_retirement_intent_gate();
        let intent = encode_retirement(
            RETIREMENT_INTENT,
            displaced.source_root,
            displaced.segment_id,
            displaced.generation,
            displaced.bytes,
        );
        if let Err(denial) = self.append_retirement(&intent) {
            if denial != PhysicalRetirementDenial::Waiting {
                self.root_owner
                    .revert_displaced_claim(displaced.segment_id, displaced.generation);
            }
            return Err(denial);
        }
        Ok(Some(displaced))
    }

    pub(in crate::physical_runtime) fn revert_retirement_claim(
        &self,
        segment_id: u64,
        generation: u64,
    ) {
        self.root_owner
            .revert_displaced_claim(segment_id, generation);
    }

    pub(in crate::physical_runtime) fn finish_retirement(
        &self,
        displaced: crate::physical_runtime::durability::DisplacedSegment,
    ) -> Result<(), PhysicalRetirementDenial> {
        if let Some(denial) = self.root_owner.blocked_retirement(&displaced) {
            self.root_owner
                .revert_displaced_claim(displaced.segment_id, displaced.generation);
            return Err(denial);
        }
        #[cfg(feature = "certification-test-authority")]
        if self
            .stop_before_retirement_delete
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            return Err(PhysicalRetirementDenial::Delete);
        }
        self.delete_displaced_segment(displaced.segment_id, displaced.generation)?;
        #[cfg(feature = "certification-test-authority")]
        if self
            .stop_after_retirement_delete
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            return Err(PhysicalRetirementDenial::Delete);
        }
        let completion = encode_retirement(
            RETIREMENT_COMPLETION,
            displaced.source_root,
            displaced.segment_id,
            displaced.generation,
            displaced.bytes,
        );
        self.append_retirement(&completion)?;
        self.root_owner
            .complete_displaced(displaced.segment_id, displaced.generation);
        Ok(())
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn arm_retirement_intent_gate(&self) {
        self.retirement_intent_gate
            .release
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.retirement_intent_gate
            .arrived
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.retirement_intent_gate
            .armed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn retirement_intent_arrived(&self) -> bool {
        self.retirement_intent_gate
            .arrived
            .load(std::sync::atomic::Ordering::Acquire)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn release_retirement_intent_gate(&self) {
        self.retirement_intent_gate
            .release
            .store(true, std::sync::atomic::Ordering::Release);
    }

    #[cfg(feature = "certification-test-authority")]
    fn wait_retirement_intent_gate(&self) {
        if !self
            .retirement_intent_gate
            .armed
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }
        self.retirement_intent_gate
            .arrived
            .store(true, std::sync::atomic::Ordering::Release);
        while !self
            .retirement_intent_gate
            .release
            .load(std::sync::atomic::Ordering::Acquire)
        {
            std::thread::yield_now();
        }
    }

    fn append_retirement(&self, payload: &[u8]) -> Result<(), PhysicalRetirementDenial> {
        match self.wal.append_scheduled_maintenance(payload) {
            Ok(()) => Ok(()),
            Err(ScheduledMaintenanceDenial::NotStarted(cause)) if scheduler_waiting(&cause) => {
                Err(PhysicalRetirementDenial::Waiting)
            }
            Err(ScheduledMaintenanceDenial::NotStarted(_)) => Err(PhysicalRetirementDenial::WalPlan),
            Err(ScheduledMaintenanceDenial::Write) => Err(PhysicalRetirementDenial::WalWrite),
            Err(ScheduledMaintenanceDenial::Sync) => Err(PhysicalRetirementDenial::WalSync),
            Err(ScheduledMaintenanceDenial::Finish) => Err(PhysicalRetirementDenial::WalFinish),
        }
    }

    fn delete_displaced_segment(
        &self,
        segment_id: u64,
        generation: u64,
    ) -> Result<(), PhysicalRetirementDenial> {
        let artifact = RecordArtifactFile::Segment {
            segment: segment_id,
            generation,
        };
        let permit = self
            .root_owner
            .removal_permit(segment_id, generation)
            .ok_or(PhysicalRetirementDenial::Delete)?;
        self.execute_record_effect(
            artifact,
            PhysicalPublicationEffect::RemoveArtifact,
            Some(permit),
        )?;
        self.execute_record_effect(
            RecordArtifactFile::BootstrapCatalog,
            PhysicalPublicationEffect::SynchronizeRecordFamily,
            None,
        )
    }
}

impl RecordPublicationDirector {
    fn execute_record_effect(
        &self,
        artifact: RecordArtifactFile,
        effect: PhysicalPublicationEffect,
        permit: Option<crate::physical_runtime::durability::RetirementRemovalPermit>,
    ) -> Result<(), PhysicalRetirementDenial> {
        let settled = self
            .root_work
            .execute_record_effect(artifact, effect, 1, permit)
            .map_err(retirement_work_denial)?;
        match settled.evidence() {
            PhysicalWorkSettlementEvidence::PublicationEffect { .. }
                if settled.evidence().fate() == PhysicalWorkEffectFate::PublicationCompleted =>
            {
                Ok(())
            }
            _ => Err(PhysicalRetirementDenial::Delete),
        }
    }
}

fn retirement_work_denial(failure: PhysicalRootPublicationWorkFailure) -> PhysicalRetirementDenial {
    match failure {
        PhysicalRootPublicationWorkFailure::Scheduler(
            PhysicalSchedulerDenial::OwedBackgroundTurn
            | PhysicalSchedulerDenial::EffectConflict
            | PhysicalSchedulerDenial::EffectSlotsExhausted,
        )
        | PhysicalRootPublicationWorkFailure::SchedulerReservation(
            crate::physical_runtime::RecordSchedulerReservationDenial::OwedBackgroundTurn,
        ) => PhysicalRetirementDenial::Waiting,
        _ => PhysicalRetirementDenial::Delete,
    }
}

fn scheduler_waiting(cause: &PhysicalWalAppendFailureCause) -> bool {
    matches!(
        cause,
        PhysicalWalAppendFailureCause::Scheduler(
            PhysicalSchedulerDenial::OwedBackgroundTurn
                | PhysicalSchedulerDenial::EffectConflict
                | PhysicalSchedulerDenial::EffectSlotsExhausted
        )
    )
}
