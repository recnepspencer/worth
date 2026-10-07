use worth_proof::TransitionOutcome;
use worth_store_io_scheduler::{execute_ready_queue_plan, BackgroundPacingOutcome};
use worth_store_physical_backend::{
    execute_recovery_cleanup_removal, AdmittedRecoveryFilesystemMedia,
    BackendRecoveryCleanupRemovalOutcome, MediaOperationRole,
};
use worth_store_physical_format::PhysicalCheckpointIdentity;

use crate::physical_runtime::recovery_coordination::settlement::{
    scheduler_posture, settle, signal_completion_is_terminal,
};
use crate::physical_runtime::work::{
    CompletedPhysicalCheckpointAction, IndeterminatePhysicalCheckpointAction,
    PhysicalCheckpointRecoveryAction, PhysicalCheckpointWorkAction, PhysicalCheckpointWorkScope,
    PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch, PhysicalExecutorOutcome,
    PhysicalMutationWorkRequest, PhysicalRetryPayload,
};
use crate::physical_runtime::{
    PhysicalSchedulerDemand, PhysicalWorkAdmission, PhysicalWorkReadiness, PhysicalWorkScheduler,
    PhysicalWorkSchedulerPosture,
};

use super::{
    AdmittedCheckpointResidue, PhysicalRecoveryCoordination, RecoveryCheckpointResidueDenial,
    RecoveryCheckpointResidueOutcome,
};

pub(super) fn remove(
    coordination: &PhysicalRecoveryCoordination,
    media: &AdmittedRecoveryFilesystemMedia,
    admitted: AdmittedCheckpointResidue,
) -> RecoveryCheckpointResidueOutcome {
    let execution = match admit_execution(coordination, admitted.checkpoint) {
        Ok(execution) => execution,
        Err(denial) => return RecoveryCheckpointResidueOutcome::DeniedBeforeEffect(denial),
    };
    let (dispatched, plan) = match execution.into_execution_parts(None) {
        Ok(parts) => parts,
        Err(_) => {
            return RecoveryCheckpointResidueOutcome::DeniedBeforeEffect(
                RecoveryCheckpointResidueDenial::Admission,
            )
        }
    };
    let physical = execute_recovery_cleanup_removal(
        media,
        admitted.request,
        plan.backend_completion_binding()
            .backend_execution_binding(),
    );
    match physical {
        BackendRecoveryCleanupRemovalOutcome::Completed(completed) => {
            let scheduler = execute_ready_queue_plan(plan, completed.queue());
            let posture = scheduler_posture(&scheduler);
            let signal = settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::CheckpointCompleted {
                        physical: CompletedPhysicalCheckpointAction::new(
                            PhysicalCheckpointRecoveryAction::RemoveCandidate,
                            completed.operation(),
                            MediaOperationRole::Delete,
                            0,
                        ),
                        scheduler,
                    },
                    PhysicalEffectRecoveryObligation::Cleared,
                ),
            );
            if posture == PhysicalWorkSchedulerPosture::Executed
                && signal_completion_is_terminal(signal)
            {
                RecoveryCheckpointResidueOutcome::Removed {
                    checkpoint: admitted.checkpoint,
                    candidate_digest: admitted.candidate_digest,
                }
            } else {
                RecoveryCheckpointResidueOutcome::Indeterminate
            }
        }
        BackendRecoveryCleanupRemovalOutcome::DeniedBeforeEffect(denied) => {
            let scheduler = denied
                .queue()
                .map(|queue| scheduler_posture(&execute_ready_queue_plan(plan, queue)));
            let signal = settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::DeniedBeforeEffect {
                        failure: denied.failure(),
                        retry: PhysicalRetryPayload::Checkpoint { payload: None },
                    },
                    PhysicalEffectRecoveryObligation::Cleared,
                ),
            );
            if scheduler.is_some_and(|posture| posture != PhysicalWorkSchedulerPosture::Executed)
                || !signal_completion_is_terminal(signal)
            {
                RecoveryCheckpointResidueOutcome::Indeterminate
            } else {
                RecoveryCheckpointResidueOutcome::DeniedBeforeEffect(
                    RecoveryCheckpointResidueDenial::Backend(denied.failure()),
                )
            }
        }
        BackendRecoveryCleanupRemovalOutcome::Indeterminate(indeterminate) => {
            let _scheduler = execute_ready_queue_plan(plan, indeterminate.queue());
            let _signal = settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::CheckpointIndeterminate(
                        IndeterminatePhysicalCheckpointAction::new(
                            PhysicalCheckpointRecoveryAction::RemoveCandidate,
                            indeterminate.operation(),
                            MediaOperationRole::Delete,
                            0,
                            indeterminate.failure(),
                        ),
                    ),
                    PhysicalEffectRecoveryObligation::Cleared,
                ),
            );
            RecoveryCheckpointResidueOutcome::Indeterminate
        }
    }
}

fn admit_execution(
    coordination: &PhysicalRecoveryCoordination,
    selected: PhysicalCheckpointIdentity,
) -> Result<crate::physical_runtime::ResourceAdmittedPhysicalWork, RecoveryCheckpointResidueDenial>
{
    let next = std::num::NonZeroU64::new(selected.sequence().get() + 1)
        .ok_or(RecoveryCheckpointResidueDenial::Admission)?;
    let candidate = PhysicalCheckpointIdentity::new(selected.store_identity(), next);
    let scope =
        PhysicalCheckpointWorkScope::new(candidate, PhysicalCheckpointWorkAction::RemoveCandidate)
            .ok_or(RecoveryCheckpointResidueDenial::Admission)?;
    let request = PhysicalMutationWorkRequest::checkpoint_capture(
        scope,
        coordination.bases[4].clone(),
        coordination.work_security,
    )
    .map_err(|_| RecoveryCheckpointResidueDenial::Admission)?;
    let receipt = match coordination
        .submission
        .mutation_submission()
        .submit(request)
        .into_raw()
    {
        TransitionOutcome::Success(receipt) => receipt,
        _ => return Err(RecoveryCheckpointResidueDenial::Admission),
    };
    let receipt_identity = receipt.identity();
    let admitted = PhysicalWorkAdmission::admit_recovery(
        &coordination.submission,
        receipt,
        &coordination.admission,
    )
    .map_err(|_| {
        let _ = coordination
            .submission
            .cancel_before_dispatch(receipt_identity);
        RecoveryCheckpointResidueDenial::Admission
    })?;
    let ready = match coordination.signal.request(admitted) {
        Ok(PhysicalWorkReadiness::Ready(ready)) => ready,
        Ok(PhysicalWorkReadiness::Blocked(blocked)) => {
            let identity = blocked.intent().identity();
            let route = blocked.authority().binding();
            if let Some((_, request)) = blocked.into_revalidation_parts() {
                cancel(
                    coordination,
                    crate::physical_runtime::PhysicalWorkConsumerHandle::new(
                        identity, request, route,
                    ),
                );
            } else {
                let _ = coordination.submission.cancel_before_dispatch(identity);
            }
            return Err(RecoveryCheckpointResidueDenial::Admission);
        }
        Err(_) => {
            let _ = coordination
                .submission
                .cancel_before_dispatch(receipt_identity);
            return Err(RecoveryCheckpointResidueDenial::Admission);
        }
    };
    let consumer = ready.consumer_handle();
    let (pacing, backend, policy, capacity) =
        match coordination
            .scheduler
            .checkpoint_background(&coordination.scheduler_security, 1, 0)
        {
            Ok(parts) => parts,
            Err(_) => {
                cancel(coordination, consumer);
                return Err(RecoveryCheckpointResidueDenial::Scheduler);
            }
        };
    let lease = match pacing {
        BackgroundPacingOutcome::AdmittedWithDebt(admitted) => admitted.into_lease(),
        _ => {
            cancel(coordination, consumer);
            return Err(RecoveryCheckpointResidueDenial::Scheduler);
        }
    };
    let demand = match PhysicalSchedulerDemand::checkpoint_background(ready, lease, capacity) {
        Ok(demand) => demand,
        Err(_) => {
            cancel(coordination, consumer);
            return Err(RecoveryCheckpointResidueDenial::Scheduler);
        }
    };
    if PhysicalWorkAdmission::require_current_recovery(&coordination.submission, demand.intent())
        .is_err()
    {
        cancel(coordination, consumer);
        return Err(RecoveryCheckpointResidueDenial::Admission);
    }
    match PhysicalWorkScheduler::admit(coordination.scheduler.effects(), demand, &backend, policy) {
        Ok(work) => Ok(work),
        Err(_) => {
            cancel(coordination, consumer);
            Err(RecoveryCheckpointResidueDenial::Scheduler)
        }
    }
}

fn cancel(
    coordination: &PhysicalRecoveryCoordination,
    consumer: crate::physical_runtime::PhysicalWorkConsumerHandle,
) {
    let _ = coordination
        .submission
        .cancel_before_dispatch(consumer.identity());
    let _ = coordination.signal.cancel(consumer);
}
