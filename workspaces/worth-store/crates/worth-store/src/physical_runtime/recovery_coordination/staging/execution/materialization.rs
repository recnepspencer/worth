use worth_store_io_scheduler::execute_ready_queue_plan;
use worth_store_physical_backend::{
    ArtifactRangeWriteDurabilityRequirement, RecoveryStagingWriteDisposition,
    RecoveryStagingWriteOutcome,
};

use crate::physical_runtime::recovery_coordination::{
    PhysicalRecoveryCoordination, RecoveryStagingWriteOccurrence,
};
use crate::physical_runtime::work::{
    PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch, PhysicalExecutorOutcome,
    PhysicalRetryPayload,
};
use crate::physical_runtime::{PhysicalWorkSchedulerPosture, PhysicalWorkScope};

use super::super::{
    PhysicalRecoveryStagingCommand, PhysicalRecoveryStagingCommandDenialKind,
    PhysicalRecoveryStagingCommandIndeterminate, PhysicalRecoveryStagingCommandOutcome,
    PhysicalRecoveryStagingCommandStage, PhysicalRecoveryStagingMaterialization,
    PhysicalRecoveryStagingMaterializationEvidence,
};
use super::outcome::{denied, pre_effect};

pub(super) fn execute(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: &PhysicalRecoveryStagingCommand<'_>,
) -> Result<PhysicalRecoveryStagingMaterialization, PhysicalRecoveryStagingCommandOutcome> {
    let coordinate = command_coordinate(command).ok_or_else(|| {
        denied(
            PhysicalRecoveryStagingCommandStage::Materialization,
            PhysicalRecoveryStagingCommandDenialKind::Submission,
            None,
            None,
        )
    })?;
    let work = super::admission::admit(
        coordination,
        PhysicalRecoveryStagingCommandStage::Materialization,
        PhysicalWorkScope::one(coordinate),
        ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        command.bytes.len() as u64,
        false,
    )?;
    let work_identity = work.intent().identity();
    let (dispatched, plan) = work
        .into_execution_parts(Some(command.payload_digest))
        .map_err(|denial| {
            pre_effect(PhysicalRecoveryStagingCommandStage::Materialization, denial)
        })?;
    match media.stage_recovery_artifact_scheduled(
        command.artifact,
        command.bytes,
        plan.backend_completion_binding()
            .backend_execution_binding(),
    ) {
        RecoveryStagingWriteOutcome::Completed(completed) => {
            let physical = completed.physical().clone();
            let scheduler = execute_ready_queue_plan(plan, completed.queue());
            let posture = scheduler_posture(&scheduler);
            let dispatch = PhysicalExecutorDispatch::new(
                dispatched,
                PhysicalExecutorOutcome::RecoveryStagingCompleted {
                    physical: physical.clone(),
                    scheduler,
                },
                PhysicalEffectRecoveryObligation::Cleared,
            );
            #[cfg(feature = "certification-test-authority")]
            let signal = crate::physical_runtime::recovery_coordination::settlement::settle_with_certification(
                coordination,
                dispatch,
                crate::physical_runtime::recovery_coordination::settlement::PhysicalRecoverySettlementCertificationStage::Staging(
                    PhysicalRecoveryStagingCommandStage::Materialization,
                ),
            );
            #[cfg(not(feature = "certification-test-authority"))]
            let signal = crate::physical_runtime::recovery_coordination::settlement::settle(
                coordination,
                dispatch,
            );
            if posture != PhysicalWorkSchedulerPosture::Executed {
                return Err(PhysicalRecoveryStagingCommandOutcome::Indeterminate(
                    PhysicalRecoveryStagingCommandIndeterminate::Scheduler {
                        stage: PhysicalRecoveryStagingCommandStage::Materialization,
                        materialization: Some(
                            PhysicalRecoveryStagingMaterializationEvidence::PhysicallyCompleted(
                                physical,
                            ),
                        ),
                        synchronization: None,
                        posture,
                    },
                ));
            }
            if !crate::physical_runtime::recovery_coordination::settlement::signal_completion_is_terminal(signal) {
                return Err(PhysicalRecoveryStagingCommandOutcome::Indeterminate(
                    PhysicalRecoveryStagingCommandIndeterminate::Signal {
                        stage: PhysicalRecoveryStagingCommandStage::Materialization,
                        materialization: Some(
                            PhysicalRecoveryStagingMaterializationEvidence::PhysicallyCompleted(
                                physical,
                            ),
                        ),
                        synchronization: None,
                        outcome: signal,
                    },
                ));
            }
            let wait = coordination.pause_at(
                crate::physical_runtime::PhysicalRecoveryYieldpointStage::StagingMaterialization,
            );
            if wait.is_interrupted() {
                return Err(PhysicalRecoveryStagingCommandOutcome::Indeterminate(
                    PhysicalRecoveryStagingCommandIndeterminate::Yieldpoint {
                        stage: PhysicalRecoveryStagingCommandStage::Materialization,
                        materialization: Some(
                            PhysicalRecoveryStagingMaterializationEvidence::PhysicallyCompleted(
                                physical.clone(),
                            ),
                        ),
                        synchronization: None,
                        wait,
                    },
                ));
            }
            Ok(match physical.disposition() {
                RecoveryStagingWriteDisposition::Created
                | RecoveryStagingWriteDisposition::CompletedFromExactPrefix => {
                    let completed_from_prefix = physical.disposition()
                        == RecoveryStagingWriteDisposition::CompletedFromExactPrefix;
                    let performed = crate::physical_runtime::recovery_coordination::PerformedRecoveryPhysicalEffect::record_write(
                        RecoveryStagingWriteOccurrence::new(
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
                    if completed_from_prefix {
                        PhysicalRecoveryStagingMaterialization::CompletedFromExactPrefix(performed)
                    } else {
                        PhysicalRecoveryStagingMaterialization::Created(performed)
                    }
                }
                RecoveryStagingWriteDisposition::AlreadyMaterialized => {
                    PhysicalRecoveryStagingMaterialization::AlreadyMaterialized(physical)
                }
            })
        }
        RecoveryStagingWriteOutcome::DeniedBeforeEffect(failure) => {
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
                        retry: PhysicalRetryPayload::NewArtifact(command.bytes.into()),
                    },
                    PhysicalEffectRecoveryObligation::Cleared,
                ),
            );
            Err(denied(
                PhysicalRecoveryStagingCommandStage::Materialization,
                PhysicalRecoveryStagingCommandDenialKind::Media(physical_failure),
                None,
                posture,
            ))
        }
        RecoveryStagingWriteOutcome::Indeterminate(physical) => {
            let scheduler = execute_ready_queue_plan(plan, physical.queue());
            let posture = scheduler_posture(&scheduler);
            let retained = physical.physical().clone();
            let _ = crate::physical_runtime::recovery_coordination::settlement::settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::RecoveryStagingIndeterminate(
                        physical.physical().clone(),
                    ),
                    PhysicalEffectRecoveryObligation::Retained,
                ),
            );
            Err(PhysicalRecoveryStagingCommandOutcome::Indeterminate(
                PhysicalRecoveryStagingCommandIndeterminate::Materialization {
                    physical: retained,
                    scheduler: Some(posture),
                },
            ))
        }
    }
}

fn command_coordinate(
    command: &PhysicalRecoveryStagingCommand<'_>,
) -> Option<worth_store_physical_format::RecordFrameCoordinate> {
    u32::try_from(command.bytes.len()).ok().and_then(|length| {
        worth_store_physical_format::RecordFrameCoordinate::new(command.artifact, 0, length)
    })
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
