use super::settlement::{settle, signal_completion_is_terminal};
use super::{PhysicalRecoveryCoordination, PhysicalRecoveryFreshReopenDenialKind};
use crate::physical_runtime::work::{
    PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch, PhysicalExecutorOutcome,
    PhysicalRetryPayload,
};
use worth_store_io_scheduler::{execute_ready_queue_plan, QueueExecutionOutcome};
use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, CompletedScheduledRecoveryReopenRead,
    RecoveryReopenReadOutcome,
};
use worth_store_physical_format::RecordFrameCoordinate;

impl PhysicalRecoveryCoordination {
    /// The recovery plan supplies one exact source frame, admitted as read work.
    pub fn execute_source_copy_read(
        &self,
        media: &AdmittedRecoveryFilesystemMedia,
        coordinate: RecordFrameCoordinate,
        maximum_bytes: u64,
    ) -> Result<CompletedScheduledRecoveryReopenRead, PhysicalRecoveryFreshReopenDenialKind> {
        if u64::from(coordinate.length()) > maximum_bytes {
            return Err(PhysicalRecoveryFreshReopenDenialKind::Submission);
        }
        let work = super::reopen::admission::admit(
            self,
            crate::physical_runtime::PhysicalWorkScope::one(coordinate),
            u64::from(coordinate.length()),
        )?;
        let (dispatched, plan) = work
            .into_execution_parts(None)
            .map_err(PhysicalRecoveryFreshReopenDenialKind::PreEffect)?;
        match media.read_recovery_range_scheduled(
            coordinate,
            maximum_bytes,
            plan.backend_completion_binding()
                .backend_execution_binding(),
        ) {
            RecoveryReopenReadOutcome::Completed(completed) => {
                let scheduler = execute_ready_queue_plan(plan, completed.queue());
                let posture = if matches!(scheduler, QueueExecutionOutcome::Executed(_)) {
                    crate::physical_runtime::PhysicalWorkSchedulerPosture::Executed
                } else {
                    crate::physical_runtime::PhysicalWorkSchedulerPosture::RejectedAfterEffect
                };
                let signal = settle(
                    self,
                    PhysicalExecutorDispatch::new(
                        dispatched,
                        PhysicalExecutorOutcome::ReadCompleted {
                            physical: completed.physical(),
                            bytes: completed.bytes().to_vec().into_boxed_slice(),
                            scheduler,
                        },
                        PhysicalEffectRecoveryObligation::Cleared,
                    ),
                );
                if posture != crate::physical_runtime::PhysicalWorkSchedulerPosture::Executed {
                    return Err(PhysicalRecoveryFreshReopenDenialKind::SchedulerSettlement(
                        posture,
                    ));
                }
                if !signal_completion_is_terminal(signal) {
                    return Err(PhysicalRecoveryFreshReopenDenialKind::SignalSettlement(
                        signal,
                    ));
                }
                Ok(completed)
            }
            RecoveryReopenReadOutcome::Denied(denied) => {
                let _scheduler = denied
                    .queue()
                    .map(|queue| execute_ready_queue_plan(plan, queue));
                let _ = settle(
                    self,
                    PhysicalExecutorDispatch::new(
                        dispatched,
                        PhysicalExecutorOutcome::DeniedBeforeEffect {
                            failure: denied.failure(),
                            retry: PhysicalRetryPayload::Read,
                        },
                        PhysicalEffectRecoveryObligation::Cleared,
                    ),
                );
                Err(PhysicalRecoveryFreshReopenDenialKind::Media(
                    denied.failure(),
                ))
            }
        }
    }
}
