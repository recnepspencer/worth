use worth_store_io_scheduler::{execute_ready_queue_plan, QueueExecutionOutcome};
use worth_store_physical_backend::{
    CompletedScheduledRecoveryReopenRead, RecoveryReopenReadOutcome,
};

use crate::physical_runtime::recovery_coordination::settlement::{
    settle, signal_completion_is_terminal,
};
#[cfg(feature = "certification-test-authority")]
use crate::physical_runtime::recovery_coordination::settlement::{
    settle_with_certification, PhysicalRecoverySettlementCertificationStage,
};
use crate::physical_runtime::work::{
    PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch, PhysicalExecutorOutcome,
    PhysicalRetryPayload,
};
use crate::physical_runtime::PhysicalWorkSchedulerPosture;

use super::super::{
    PhysicalRecoveryFreshReopenDenial, PhysicalRecoveryFreshReopenDenialKind,
    PhysicalRecoveryFreshReopenStage,
};
use crate::physical_runtime::recovery_coordination::PhysicalRecoveryCoordination;

pub(super) struct CompletedRead {
    pub(super) physical: CompletedScheduledRecoveryReopenRead,
    pub(super) work: crate::physical_runtime::PhysicalWorkIdentity,
    pub(super) signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
}

pub(super) fn execute(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    stage: PhysicalRecoveryFreshReopenStage,
    generation: u64,
    maximum_bytes: u64,
) -> Result<CompletedRead, PhysicalRecoveryFreshReopenDenial> {
    let artifact = super::super::artifact(stage, generation);
    let work = super::super::admission::admit(
        coordination,
        crate::physical_runtime::PhysicalWorkScope::artifact(artifact),
        maximum_bytes,
    )
    .map_err(|kind| PhysicalRecoveryFreshReopenDenial::new(stage, kind, None, None, None))?;
    let work_identity = work.intent().identity();
    let (dispatched, plan) = work.into_execution_parts(None).map_err(|denial| {
        PhysicalRecoveryFreshReopenDenial::new(
            stage,
            PhysicalRecoveryFreshReopenDenialKind::PreEffect(denial),
            None,
            None,
            None,
        )
    })?;
    match media.read_recovery_artifact_scheduled(
        artifact,
        maximum_bytes,
        plan.backend_completion_binding()
            .backend_execution_binding(),
    ) {
        RecoveryReopenReadOutcome::Completed(completed) => {
            let queue = completed.queue();
            #[cfg(feature = "certification-test-authority")]
            let queue = if coordination.take_certification_reopen_scheduler_failure(stage) {
                queue.with_foreign_plan_binding_for_certification()
            } else {
                queue
            };
            let scheduler = execute_ready_queue_plan(plan, queue);
            let posture = scheduler_posture(&scheduler);
            let dispatch = PhysicalExecutorDispatch::new(
                dispatched,
                PhysicalExecutorOutcome::ReadCompleted {
                    physical: completed.physical(),
                    bytes: completed.bytes().to_vec().into_boxed_slice(),
                    scheduler,
                },
                PhysicalEffectRecoveryObligation::Cleared,
            );
            #[cfg(feature = "certification-test-authority")]
            let signal = settle_with_certification(
                coordination,
                dispatch,
                PhysicalRecoverySettlementCertificationStage::FreshReopen(stage),
            );
            #[cfg(not(feature = "certification-test-authority"))]
            let signal = settle(coordination, dispatch);
            if posture != PhysicalWorkSchedulerPosture::Executed {
                return Err(completed_settlement_denial(
                    stage,
                    PhysicalRecoveryFreshReopenDenialKind::SchedulerSettlement(posture),
                    completed,
                ));
            }
            if !signal_completion_is_terminal(signal) {
                return Err(completed_settlement_denial(
                    stage,
                    PhysicalRecoveryFreshReopenDenialKind::SignalSettlement(signal),
                    completed,
                ));
            }
            let wait = coordination.pause_at(match stage {
                PhysicalRecoveryFreshReopenStage::CurrentSelector => {
                    crate::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenCurrentSelector
                }
                PhysicalRecoveryFreshReopenStage::RootManifest => {
                    crate::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenRootManifest
                }
                PhysicalRecoveryFreshReopenStage::ExactBinding => unreachable!(
                    "exact binding is checked after selector and root reads"
                ),
            });
            if wait.is_interrupted() {
                return Err(completed_settlement_denial(
                    stage,
                    PhysicalRecoveryFreshReopenDenialKind::Yieldpoint(wait),
                    completed.clone(),
                ));
            }
            Ok(CompletedRead {
                physical: completed,
                work: work_identity,
                signal,
            })
        }
        RecoveryReopenReadOutcome::Denied(denied) => {
            let _scheduler = denied
                .queue()
                .map(|queue| execute_ready_queue_plan(plan, queue));
            let _ = settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::DeniedBeforeEffect {
                        failure: denied.failure(),
                        retry: PhysicalRetryPayload::Read,
                    },
                    PhysicalEffectRecoveryObligation::Cleared,
                ),
            );
            Err(PhysicalRecoveryFreshReopenDenial::new(
                stage,
                PhysicalRecoveryFreshReopenDenialKind::Media(denied.failure()),
                None,
                None,
                Some(denied),
            ))
        }
    }
}

fn completed_settlement_denial(
    stage: PhysicalRecoveryFreshReopenStage,
    kind: PhysicalRecoveryFreshReopenDenialKind,
    completed: CompletedScheduledRecoveryReopenRead,
) -> PhysicalRecoveryFreshReopenDenial {
    let (selector, root) = match stage {
        PhysicalRecoveryFreshReopenStage::CurrentSelector => (Some(completed), None),
        PhysicalRecoveryFreshReopenStage::RootManifest => (None, Some(completed)),
        PhysicalRecoveryFreshReopenStage::ExactBinding => {
            unreachable!("exact binding is not a physical read stage")
        }
    };
    PhysicalRecoveryFreshReopenDenial::new(stage, kind, selector, root, None)
}

fn scheduler_posture(outcome: &QueueExecutionOutcome) -> PhysicalWorkSchedulerPosture {
    if matches!(outcome, QueueExecutionOutcome::Executed(_)) {
        PhysicalWorkSchedulerPosture::Executed
    } else {
        PhysicalWorkSchedulerPosture::RejectedAfterEffect
    }
}
