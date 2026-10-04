use super::PhysicalWorkExecutor;
use crate::physical_runtime::{
    DispatchedPhysicalWork, PhysicalEffectRecoveryObligation, PhysicalWorkPreEffectDenial,
    PhysicalWorkRecoveryTarget,
};

pub(super) fn scheduler_recovery(
    scheduler: &worth_store_io_scheduler::QueueExecutionOutcome,
) -> PhysicalEffectRecoveryObligation {
    if matches!(
        scheduler,
        worth_store_io_scheduler::QueueExecutionOutcome::Executed(_)
    ) {
        PhysicalEffectRecoveryObligation::Cleared
    } else {
        PhysicalEffectRecoveryObligation::Retained
    }
}

impl PhysicalWorkExecutor {
    pub(super) fn prepare_effect_recovery(
        &self,
        dispatched: &DispatchedPhysicalWork,
        target: PhysicalWorkRecoveryTarget,
        payload_digest: Option<[u8; 32]>,
    ) -> Result<crate::physical_runtime::work::PreparedPhysicalEffect, PhysicalWorkPreEffectDenial>
    {
        self.recovery
            .prepare(
                &self.media,
                dispatched.intent().identity(),
                dispatched.intent().operation(),
                target,
                payload_digest,
            )
            .map_err(|_| PhysicalWorkPreEffectDenial::RecoveryJournalUnavailable)
    }

    pub(super) fn finish_effect_recovery(
        &self,
        prepared: crate::physical_runtime::work::PreparedPhysicalEffect,
        physical: PhysicalEffectRecoveryObligation,
    ) -> PhysicalEffectRecoveryObligation {
        match physical {
            PhysicalEffectRecoveryObligation::Retained
            | PhysicalEffectRecoveryObligation::RetainedWithoutRecord => {
                match self.recovery.retain(&self.media, prepared) {
                    Ok(()) => PhysicalEffectRecoveryObligation::Retained,
                    Err(()) => PhysicalEffectRecoveryObligation::RetainedWithoutRecord,
                }
            }
            PhysicalEffectRecoveryObligation::Cleared => {
                if self.recovery.finish(&self.media, prepared).is_ok() {
                    PhysicalEffectRecoveryObligation::Cleared
                } else {
                    PhysicalEffectRecoveryObligation::Retained
                }
            }
        }
    }
}
