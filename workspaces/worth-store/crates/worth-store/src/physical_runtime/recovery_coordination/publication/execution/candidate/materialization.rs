use worth_store_io_scheduler::execute_ready_queue_plan;
use worth_store_physical_backend::{RecoveryStagingWriteDisposition, RecoveryStagingWriteOutcome};

use crate::physical_runtime::recovery_coordination::settlement::{
    settle, signal_completion_is_terminal,
};
#[cfg(feature = "certification-test-authority")]
use crate::physical_runtime::recovery_coordination::settlement::{
    settle_with_certification, PhysicalRecoverySettlementCertificationStage,
};
use crate::physical_runtime::recovery_coordination::{
    CompletedPhysicalRecoveryPublicationCandidate, PerformedRecoveryPhysicalEffect,
    PhysicalRecoveryCoordination, PhysicalRecoveryPublicationCandidate,
    PhysicalRecoveryPublicationCandidateMaterialization,
    RecoveryPublicationCandidateMaterializationOccurrence,
};
use crate::physical_runtime::work::{
    PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch, PhysicalExecutorOutcome,
    PhysicalRetryPayload,
};
use crate::physical_runtime::{PhysicalWorkSchedulerPosture, PhysicalWorkScope};
use outcome::{attach_completed, denied};

use super::super::super::{
    PhysicalRecoveryPublicationCommand, PhysicalRecoveryPublicationCommandDenial,
    PhysicalRecoveryPublicationCommandDenialKind, PhysicalRecoveryPublicationCommandIndeterminate,
    PhysicalRecoveryPublicationCommandOutcome, PhysicalRecoveryPublicationCommandStage,
    PhysicalRecoveryPublicationSettlementFailure,
};

mod outcome;

pub(super) struct MaterializedCandidate {
    pub(super) completed: Vec<CompletedPhysicalRecoveryPublicationCandidate>,
    pub(super) materialization: PhysicalRecoveryPublicationCandidateMaterialization,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn execute(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: &PhysicalRecoveryPublicationCommand,
    candidate: &PhysicalRecoveryPublicationCandidate,
    ordinal: u64,
    completed: Vec<CompletedPhysicalRecoveryPublicationCandidate>,
) -> Result<MaterializedCandidate, PhysicalRecoveryPublicationCommandOutcome> {
    let stage = PhysicalRecoveryPublicationCommandStage::CandidateMaterialization;
    let Some(coordinate) = u32::try_from(candidate.bytes().len())
        .ok()
        .and_then(|length| {
            worth_store_physical_format::RecordFrameCoordinate::new(candidate.artifact(), 0, length)
        })
    else {
        return Err(denied(stage, completed, None));
    };
    let work = match admission::admit(coordination, stage, PhysicalWorkScope::one(coordinate)) {
        Ok(work) => work,
        Err(outcome) => return Err(attach_completed(outcome, completed)),
    };
    let work_identity = work.intent().identity();
    let (dispatched, plan) = match work.into_execution_parts(Some(candidate.payload_digest())) {
        Ok(parts) => parts,
        Err(_) => return Err(denied(stage, completed, None)),
    };
    match media.stage_recovery_artifact_scheduled(
        candidate.artifact(),
        candidate.bytes(),
        plan.backend_completion_binding()
            .backend_execution_binding(),
    ) {
        RecoveryStagingWriteOutcome::Completed(outcome) => {
            let physical = outcome.physical().clone();
            let queue = outcome.queue();
            #[cfg(feature = "certification-test-authority")]
            let queue = if coordination.take_certification_publication_scheduler_failure(stage) {
                queue.with_foreign_plan_binding_for_certification()
            } else {
                queue
            };
            let scheduler = execute_ready_queue_plan(plan, queue);
            let posture = super::scheduler_posture(&scheduler);
            let dispatch = PhysicalExecutorDispatch::new(
                dispatched,
                PhysicalExecutorOutcome::RecoveryStagingCompleted {
                    physical: physical.clone(),
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
            if posture != PhysicalWorkSchedulerPosture::Executed
                || !signal_completion_is_terminal(signal)
            {
                return Err(PhysicalRecoveryPublicationCommandOutcome::Indeterminate(
                    PhysicalRecoveryPublicationCommandIndeterminate::CandidateMaterializationSettlement {
                        artifact: candidate.artifact(),
                        physical,
                        completed: completed.into_boxed_slice(),
                        failure: if posture != PhysicalWorkSchedulerPosture::Executed {
                            PhysicalRecoveryPublicationSettlementFailure::Scheduler(posture)
                        } else {
                            PhysicalRecoveryPublicationSettlementFailure::Signal(signal)
                        },
                    },
                ));
            }
            let wait = coordination.pause_at(
                crate::physical_runtime::PhysicalRecoveryYieldpointStage::CandidateMaterialization,
            );
            if wait.is_interrupted() {
                return Err(PhysicalRecoveryPublicationCommandOutcome::Indeterminate(
                    PhysicalRecoveryPublicationCommandIndeterminate::CandidateMaterializationYieldpoint {
                        artifact: candidate.artifact(),
                        physical,
                        completed: completed.into_boxed_slice(),
                        wait,
                    },
                ));
            }
            let materialization = match physical.disposition() {
                RecoveryStagingWriteDisposition::Created
                | RecoveryStagingWriteDisposition::CompletedFromExactPrefix => {
                    let completed_from_prefix = physical.disposition()
                        == RecoveryStagingWriteDisposition::CompletedFromExactPrefix;
                    let performed =
                        PerformedRecoveryPhysicalEffect::record_candidate_materialization(
                            RecoveryPublicationCandidateMaterializationOccurrence::new(
                                super::occurrence(
                                    coordination,
                                    command,
                                    candidate,
                                    ordinal,
                                    work_identity,
                                    posture,
                                    signal,
                                ),
                                physical,
                            ),
                        );
                    if completed_from_prefix {
                        PhysicalRecoveryPublicationCandidateMaterialization::CompletedFromExactPrefix(
                            performed,
                        )
                    } else {
                        PhysicalRecoveryPublicationCandidateMaterialization::Created(performed)
                    }
                }
                RecoveryStagingWriteDisposition::AlreadyMaterialized => {
                    PhysicalRecoveryPublicationCandidateMaterialization::AlreadyMaterialized(
                        physical,
                    )
                }
            };
            Ok(MaterializedCandidate {
                completed,
                materialization,
            })
        }
        RecoveryStagingWriteOutcome::DeniedBeforeEffect(failure) => {
            let physical_failure = failure.failure();
            let scheduler = failure
                .queue()
                .map(|queue| execute_ready_queue_plan(plan, queue));
            let _ = settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::DeniedBeforeEffect {
                        failure: physical_failure,
                        retry: PhysicalRetryPayload::NewArtifact(candidate.bytes().into()),
                    },
                    PhysicalEffectRecoveryObligation::Cleared,
                ),
            );
            Err(
                PhysicalRecoveryPublicationCommandOutcome::DeniedBeforeEffect(
                    PhysicalRecoveryPublicationCommandDenial::new(
                        stage,
                        PhysicalRecoveryPublicationCommandDenialKind::Media(physical_failure),
                        completed.into_boxed_slice(),
                        None,
                        None,
                        scheduler.as_ref().map(super::scheduler_posture),
                    ),
                ),
            )
        }
        RecoveryStagingWriteOutcome::Indeterminate(outcome) => {
            let scheduler = execute_ready_queue_plan(plan, outcome.queue());
            let posture = super::scheduler_posture(&scheduler);
            let physical = outcome.physical().clone();
            let _ = settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::RecoveryStagingIndeterminate(physical.clone()),
                    PhysicalEffectRecoveryObligation::Retained,
                ),
            );
            Err(PhysicalRecoveryPublicationCommandOutcome::Indeterminate(
                PhysicalRecoveryPublicationCommandIndeterminate::CandidateMaterialization {
                    artifact: candidate.artifact(),
                    physical,
                    completed: completed.into_boxed_slice(),
                    scheduler: Some(posture),
                },
            ))
        }
    }
}

use super::super::admission;
