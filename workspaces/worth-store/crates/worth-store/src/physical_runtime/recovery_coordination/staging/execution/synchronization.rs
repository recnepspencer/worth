use worth_store_io_scheduler::execute_ready_queue_plan;
use worth_store_physical_backend::RecoveryStagingSynchronizationOutcome;

use crate::physical_runtime::recovery_coordination::{
    PhysicalRecoveryCoordination, RecoveryStagingSynchronizationOccurrence,
};
use crate::physical_runtime::work::{
    IndeterminatePhysicalPublicationEffect, PhysicalEffectRecoveryObligation,
    PhysicalExecutorDispatch, PhysicalExecutorOutcome, PhysicalRetryPayload,
};
use crate::physical_runtime::{
    PhysicalPublicationEffect, PhysicalWorkSchedulerPosture, PhysicalWorkScope,
};

use super::super::{
    CompletedPhysicalRecoveryStagingCommand, PhysicalRecoveryStagingCommand,
    PhysicalRecoveryStagingCommandDenialKind, PhysicalRecoveryStagingCommandIndeterminate,
    PhysicalRecoveryStagingCommandOutcome, PhysicalRecoveryStagingCommandStage,
    PhysicalRecoveryStagingMaterialization, PhysicalRecoveryStagingMaterializationEvidence,
};
use super::outcome::{attach_materialization, denied, pre_effect};

pub(super) fn execute(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: PhysicalRecoveryStagingCommand<'_>,
    materialization: PhysicalRecoveryStagingMaterialization,
) -> PhysicalRecoveryStagingCommandOutcome {
    let work = match super::admission::admit(
        coordination,
        PhysicalRecoveryStagingCommandStage::Synchronization,
        PhysicalWorkScope::artifact(command.artifact),
        worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::FileDataSynchronization,
        0,
        true,
    ) {
        Ok(work) => work,
        Err(outcome) => return attach_materialization(outcome, materialization),
    };
    let work_identity = work.intent().identity();
    let (dispatched, plan) = match work.into_execution_parts(None) {
        Ok(parts) => parts,
        Err(denial) => {
            return attach_materialization(
                pre_effect(PhysicalRecoveryStagingCommandStage::Synchronization, denial),
                materialization,
            )
        }
    };
    match media.synchronize_recovery_artifact_scheduled(
        command.artifact,
        plan.backend_completion_binding()
            .backend_execution_binding(),
    ) {
        RecoveryStagingSynchronizationOutcome::Completed(completed) => {
            let physical = completed.physical().clone();
            let scheduler = execute_ready_queue_plan(plan, completed.queue());
            let posture = scheduler_posture(&scheduler);
            let dispatch = PhysicalExecutorDispatch::new(
                dispatched,
                PhysicalExecutorOutcome::PublicationEffectCompleted {
                    physical: crate::physical_runtime::CompletedPhysicalPublicationEffect::new(
                        physical.clone(),
                        command.artifact,
                        PhysicalPublicationEffect::SynchronizeArtifact,
                    ),
                    scheduler,
                },
                PhysicalEffectRecoveryObligation::Cleared,
            );
            #[cfg(feature = "certification-test-authority")]
            let signal = crate::physical_runtime::recovery_coordination::settlement::settle_with_certification(
                coordination,
                dispatch,
                crate::physical_runtime::recovery_coordination::settlement::PhysicalRecoverySettlementCertificationStage::Staging(
                    PhysicalRecoveryStagingCommandStage::Synchronization,
                ),
            );
            #[cfg(not(feature = "certification-test-authority"))]
            let signal = crate::physical_runtime::recovery_coordination::settlement::settle(
                coordination,
                dispatch,
            );
            if posture != PhysicalWorkSchedulerPosture::Executed {
                return PhysicalRecoveryStagingCommandOutcome::Indeterminate(
                    PhysicalRecoveryStagingCommandIndeterminate::Scheduler {
                        stage: PhysicalRecoveryStagingCommandStage::Synchronization,
                        materialization: Some(
                            PhysicalRecoveryStagingMaterializationEvidence::Performed(
                                materialization,
                            ),
                        ),
                        synchronization: Some(physical),
                        posture,
                    },
                );
            }
            if !crate::physical_runtime::recovery_coordination::settlement::signal_completion_is_terminal(signal) {
                return PhysicalRecoveryStagingCommandOutcome::Indeterminate(
                    PhysicalRecoveryStagingCommandIndeterminate::Signal {
                        stage: PhysicalRecoveryStagingCommandStage::Synchronization,
                        materialization: Some(
                            PhysicalRecoveryStagingMaterializationEvidence::Performed(
                                materialization,
                            ),
                        ),
                        synchronization: Some(physical),
                        outcome: signal,
                    },
                );
            }
            let wait = coordination.pause_at(
                crate::physical_runtime::PhysicalRecoveryYieldpointStage::StagingSynchronization,
            );
            if wait.is_interrupted() {
                return PhysicalRecoveryStagingCommandOutcome::Indeterminate(
                    PhysicalRecoveryStagingCommandIndeterminate::Yieldpoint {
                        stage: PhysicalRecoveryStagingCommandStage::Synchronization,
                        materialization: Some(
                            PhysicalRecoveryStagingMaterializationEvidence::Performed(
                                materialization,
                            ),
                        ),
                        synchronization: Some(physical),
                        wait,
                    },
                );
            }
            let performed = crate::physical_runtime::recovery_coordination::PerformedRecoveryPhysicalEffect::record_synchronization(
                RecoveryStagingSynchronizationOccurrence::new(
                    coordination.session_identity(),
                    command.plan,
                    command.staging_generation,
                    command.ordinal,
                    physical,
                    work_identity,
                    posture,
                    signal,
                ),
            );
            PhysicalRecoveryStagingCommandOutcome::Completed(
                CompletedPhysicalRecoveryStagingCommand::new(materialization, performed),
            )
        }
        RecoveryStagingSynchronizationOutcome::DeniedBeforeEffect(failure) => {
            let scheduler = failure
                .queue()
                .map(|queue| execute_ready_queue_plan(plan, queue));
            let posture = scheduler.as_ref().map(scheduler_posture);
            let physical_failure = failure.failure();
            let _ = crate::physical_runtime::recovery_coordination::settlement::settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::DeniedBeforeEffect {
                        failure: physical_failure,
                        retry: PhysicalRetryPayload::PublicationEffect(
                            PhysicalPublicationEffect::SynchronizeArtifact,
                        ),
                    },
                    PhysicalEffectRecoveryObligation::Cleared,
                ),
            );
            denied(
                PhysicalRecoveryStagingCommandStage::Synchronization,
                PhysicalRecoveryStagingCommandDenialKind::Media(physical_failure),
                Some(materialization),
                posture,
            )
        }
        RecoveryStagingSynchronizationOutcome::Indeterminate(physical) => {
            let scheduler = execute_ready_queue_plan(plan, physical.queue());
            let posture = scheduler_posture(&scheduler);
            let retained = physical.physical().clone();
            let _ = crate::physical_runtime::recovery_coordination::settlement::settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::PublicationEffectIndeterminate(
                        IndeterminatePhysicalPublicationEffect::new(
                            physical.physical().clone(),
                            command.artifact,
                            PhysicalPublicationEffect::SynchronizeArtifact,
                        ),
                    ),
                    PhysicalEffectRecoveryObligation::Retained,
                ),
            );
            PhysicalRecoveryStagingCommandOutcome::Indeterminate(
                PhysicalRecoveryStagingCommandIndeterminate::Synchronization {
                    physical: retained,
                    materialization,
                    scheduler: Some(posture),
                },
            )
        }
    }
}

fn scheduler_posture(
    outcome: &worth_store_io_scheduler::QueueExecutionOutcome,
) -> PhysicalWorkSchedulerPosture {
    if matches!(
        outcome,
        worth_store_io_scheduler::QueueExecutionOutcome::Executed(_)
    ) {
        PhysicalWorkSchedulerPosture::Executed
    } else {
        PhysicalWorkSchedulerPosture::RejectedAfterEffect
    }
}
