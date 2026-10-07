use worth_store_io_scheduler::execute_ready_queue_plan;
use worth_store_physical_backend::{
    BackendQueueExecutionAdaptation, ScheduledArtifactRangeWriteOutcome,
};

use super::recovery_obligation::scheduler_recovery;
use super::PhysicalWorkExecutor;
use crate::physical_runtime::{
    record_serving::residency::artifact_tree::PhysicalRecordArtifactTree,
    work::PhysicalRetryPayload, PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch,
    PhysicalExecutorOutcome, PhysicalWriteExecutorCommand,
};

#[derive(Clone, Copy)]
enum PhysicalRangeWriteRole {
    ExactWrite,
    Publication,
}

impl PhysicalWorkExecutor {
    pub(super) fn dispatch_exact_write(
        &self,
        command: PhysicalWriteExecutorCommand,
    ) -> Result<PhysicalExecutorDispatch, crate::physical_runtime::PhysicalWorkPreEffectDenial>
    {
        self.dispatch_range_write(command, PhysicalRangeWriteRole::ExactWrite)
    }

    pub(super) fn dispatch_publication_write(
        &self,
        command: PhysicalWriteExecutorCommand,
    ) -> Result<PhysicalExecutorDispatch, crate::physical_runtime::PhysicalWorkPreEffectDenial>
    {
        self.dispatch_range_write(command, PhysicalRangeWriteRole::Publication)
    }

    fn dispatch_range_write(
        &self,
        command: PhysicalWriteExecutorCommand,
        role: PhysicalRangeWriteRole,
    ) -> Result<PhysicalExecutorDispatch, crate::physical_runtime::PhysicalWorkPreEffectDenial>
    {
        let PhysicalWriteExecutorCommand {
            work,
            coordinate,
            payload,
            payload_digest,
            retirement_retry,
        } = command;
        let durability = write_durability(work.intent().durability());
        let (dispatched, plan) = work.into_execution_parts(Some(payload_digest))?;
        let prepared = self.prepare_effect_recovery(
            &dispatched,
            crate::physical_runtime::PhysicalWorkRecoveryTarget::Range(coordinate),
            Some(payload_digest),
        )?;
        let tree = PhysicalRecordArtifactTree::new(&self.media);
        let physical = if retirement_retry.is_some() {
            tree.write_retirement_candidate_retry(
                coordinate,
                &payload,
                plan.backend_completion_binding()
                    .backend_execution_binding(),
                BackendQueueExecutionAdaptation::None,
                durability,
            )
        } else {
            tree.write_scheduled_foreground_exact_at(
                coordinate,
                &payload,
                plan.backend_completion_binding()
                    .backend_execution_binding(),
                BackendQueueExecutionAdaptation::None,
                durability,
            )
        };
        let (outcome, physical_recovery) = self.classify_range_write(plan, physical, payload, role);
        let recovery = self.finish_effect_recovery(prepared, physical_recovery);
        Ok(PhysicalExecutorDispatch::new(dispatched, outcome, recovery))
    }

    fn classify_range_write(
        &self,
        plan: worth_store_io_scheduler::QueueExecutionReadyPlan,
        physical: ScheduledArtifactRangeWriteOutcome,
        payload: Box<[u8]>,
        role: PhysicalRangeWriteRole,
    ) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
        match physical {
            ScheduledArtifactRangeWriteOutcome::Completed(completed) => {
                let physical = completed.physical().clone();
                #[cfg(feature = "certification-test-authority")]
                if matches!(role, PhysicalRangeWriteRole::ExactWrite) {
                    self.certification_yieldpoints.pause(
                        super::CertificationPhysicalExecutionCheckpoint::
                            AfterExactWriteBeforeSchedulerSettlement,
                    );
                }
                let scheduler = execute_ready_queue_plan(plan, completed.queue());
                let recovery = scheduler_recovery(&scheduler);
                let outcome = match role {
                    PhysicalRangeWriteRole::ExactWrite => PhysicalExecutorOutcome::WriteCompleted {
                        physical,
                        scheduler,
                    },
                    PhysicalRangeWriteRole::Publication => {
                        PhysicalExecutorOutcome::PublicationCompleted {
                            physical,
                            scheduler,
                        }
                    }
                };
                (outcome, recovery)
            }
            ScheduledArtifactRangeWriteOutcome::DeniedBeforeEffect(failure) => (
                PhysicalExecutorOutcome::DeniedBeforeEffect {
                    failure,
                    retry: retry_payload(role, payload),
                },
                PhysicalEffectRecoveryObligation::Cleared,
            ),
            ScheduledArtifactRangeWriteOutcome::Indeterminate(failure) => (
                PhysicalExecutorOutcome::Indeterminate(failure),
                PhysicalEffectRecoveryObligation::Retained,
            ),
        }
    }
}

fn write_durability(
    requirement: crate::physical_runtime::PhysicalWorkDurabilityRequirement,
) -> worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement {
    match requirement {
        crate::physical_runtime::PhysicalWorkDurabilityRequirement::ArtifactRangeWrite(
            durability,
        ) => durability,
        crate::physical_runtime::PhysicalWorkDurabilityRequirement::ReadOnly => {
            unreachable!("write commands admit mutation durability")
        }
        crate::physical_runtime::PhysicalWorkDurabilityRequirement::WalAppend => {
            unreachable!("WAL append commands use the dedicated WAL executor")
        }
        crate::physical_runtime::PhysicalWorkDurabilityRequirement::WalDurabilityBarrier => {
            unreachable!("WAL barrier commands use the dedicated WAL barrier executor")
        }
        crate::physical_runtime::PhysicalWorkDurabilityRequirement::CheckpointCapture => {
            unreachable!("checkpoint commands use the dedicated checkpoint executor")
        }
        crate::physical_runtime::PhysicalWorkDurabilityRequirement::WalReclamation => {
            unreachable!("WAL reclamation commands use the dedicated reclamation executor")
        }
        crate::physical_runtime::PhysicalWorkDurabilityRequirement::RootPublication => {
            unreachable!("root publication commands use the dedicated root executor")
        }
    }
}

fn retry_payload(role: PhysicalRangeWriteRole, payload: Box<[u8]>) -> PhysicalRetryPayload {
    match role {
        PhysicalRangeWriteRole::ExactWrite => PhysicalRetryPayload::ExactWrite(payload),
        PhysicalRangeWriteRole::Publication => PhysicalRetryPayload::Publication(payload),
    }
}
