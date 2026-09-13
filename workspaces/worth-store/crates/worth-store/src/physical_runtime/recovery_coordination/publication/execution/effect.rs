use worth_store_io_scheduler::{execute_ready_queue_plan, QueueExecutionOutcome};
use worth_store_physical_backend::ScheduledArtifactTreePublicationEffectOutcome;

use crate::physical_runtime::recovery_coordination::settlement::{
    settle, signal_completion_is_terminal,
};
#[cfg(feature = "certification-test-authority")]
use crate::physical_runtime::recovery_coordination::settlement::{
    settle_with_certification, PhysicalRecoverySettlementCertificationStage,
};
use crate::physical_runtime::work::{
    IndeterminatePhysicalPublicationEffect, PhysicalEffectRecoveryObligation,
    PhysicalExecutorDispatch, PhysicalExecutorOutcome, PhysicalRetryPayload,
};
use crate::physical_runtime::{PhysicalPublicationEffect, PhysicalWorkSchedulerPosture};

use super::super::{
    CompletedPhysicalRecoveryPublicationCandidate, PhysicalRecoveryPublicationCommandDenial,
    PhysicalRecoveryPublicationCommandDenialKind, PhysicalRecoveryPublicationCommandIndeterminate,
    PhysicalRecoveryPublicationCommandOutcome, PhysicalRecoveryPublicationCommandStage,
};
use crate::physical_runtime::recovery_coordination::{
    PerformedRecoveryPhysicalEffect, PhysicalRecoveryCoordination,
    RecoveryRootProtocolReplacementAction,
};

pub(super) struct CompletedPublicationEffect {
    pub(super) physical: worth_store_physical_backend::CompletedArtifactTreePublicationEffect,
    pub(super) posture: PhysicalWorkSchedulerPosture,
    pub(super) signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
    pub(super) candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
    pub(super) root_protocol:
        Option<PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn complete(
    coordination: &PhysicalRecoveryCoordination,
    dispatched: crate::physical_runtime::DispatchedPhysicalWork,
    plan: worth_store_io_scheduler::QueueExecutionReadyPlan,
    physical: ScheduledArtifactTreePublicationEffectOutcome,
    stage: PhysicalRecoveryPublicationCommandStage,
    artifact: worth_store_physical_format::RecordArtifactFile,
    effect: PhysicalPublicationEffect,
    candidates: Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
    root_protocol: Option<PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>>,
) -> Result<CompletedPublicationEffect, PhysicalRecoveryPublicationCommandOutcome> {
    match physical {
        ScheduledArtifactTreePublicationEffectOutcome::Completed(completed) => {
            let physical = completed.physical().clone();
            let queue = completed.queue();
            #[cfg(feature = "certification-test-authority")]
            let queue = if coordination.take_certification_publication_scheduler_failure(stage) {
                queue.with_foreign_plan_binding_for_certification()
            } else {
                queue
            };
            let scheduler = execute_ready_queue_plan(plan, queue);
            let posture = scheduler_posture(&scheduler);
            let dispatch = PhysicalExecutorDispatch::new(
                dispatched,
                PhysicalExecutorOutcome::PublicationEffectCompleted {
                    physical: crate::physical_runtime::CompletedPhysicalPublicationEffect::new(
                        physical.clone(),
                        artifact,
                        effect,
                    ),
                    scheduler,
                },
                PhysicalEffectRecoveryObligation::Cleared,
            );
            #[cfg(feature = "certification-test-authority")]
            let signal = settle_with_certification(
                coordination,
                dispatch,
                PhysicalRecoverySettlementCertificationStage::Publication(stage),
            );
            #[cfg(not(feature = "certification-test-authority"))]
            let signal = settle(coordination, dispatch);
            if posture != PhysicalWorkSchedulerPosture::Executed {
                return Err(PhysicalRecoveryPublicationCommandOutcome::Indeterminate(
                    PhysicalRecoveryPublicationCommandIndeterminate::Scheduler {
                        stage,
                        physical,
                        candidates,
                        root_protocol,
                        posture,
                    },
                ));
            }
            if !signal_completion_is_terminal(signal) {
                return Err(PhysicalRecoveryPublicationCommandOutcome::Indeterminate(
                    PhysicalRecoveryPublicationCommandIndeterminate::Signal {
                        stage,
                        physical,
                        candidates,
                        root_protocol,
                        outcome: signal,
                    },
                ));
            }
            Ok(CompletedPublicationEffect {
                physical,
                posture,
                signal,
                candidates,
                root_protocol,
            })
        }
        ScheduledArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(failure) => {
            let _ = settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::DeniedBeforeEffect {
                        failure,
                        retry: PhysicalRetryPayload::RootPublicationEffect,
                    },
                    PhysicalEffectRecoveryObligation::Cleared,
                ),
            );
            Err(
                PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(
                    PhysicalRecoveryPublicationCommandDenial::new(
                        stage,
                        PhysicalRecoveryPublicationCommandDenialKind::Media(failure),
                        candidates,
                        None,
                        root_protocol,
                        None,
                    ),
                ),
            )
        }
        ScheduledArtifactTreePublicationEffectOutcome::Indeterminate(physical) => {
            let retained = physical.clone();
            let _ = settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::PublicationEffectIndeterminate(
                        IndeterminatePhysicalPublicationEffect::new(physical, artifact, effect),
                    ),
                    PhysicalEffectRecoveryObligation::Retained,
                ),
            );
            Err(PhysicalRecoveryPublicationCommandOutcome::Indeterminate(
                PhysicalRecoveryPublicationCommandIndeterminate::Media {
                    stage,
                    physical: retained,
                    candidates,
                    root_protocol,
                },
            ))
        }
    }
}

fn scheduler_posture(outcome: &QueueExecutionOutcome) -> PhysicalWorkSchedulerPosture {
    if matches!(outcome, QueueExecutionOutcome::Executed(_)) {
        PhysicalWorkSchedulerPosture::Executed
    } else {
        PhysicalWorkSchedulerPosture::RejectedAfterEffect
    }
}
