use worth_store_io_scheduler::execute_ready_queue_plan;
use worth_store_physical_backend::RecoveryStagingSynchronizationOutcome;

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
    RecoveryPublicationCandidateSynchronizationAction,
    RecoveryPublicationCandidateSynchronizationOccurrence,
};
use crate::physical_runtime::work::{
    IndeterminatePhysicalPublicationEffect, PhysicalEffectRecoveryObligation,
    PhysicalExecutorDispatch, PhysicalExecutorOutcome, PhysicalRetryPayload,
};
use crate::physical_runtime::{
    PhysicalPublicationEffect, PhysicalWorkSchedulerPosture, PhysicalWorkScope,
};

use super::super::super::{
    PhysicalRecoveryPublicationCommand, PhysicalRecoveryPublicationCommandIndeterminate,
    PhysicalRecoveryPublicationCommandOutcome, PhysicalRecoveryPublicationCommandStage,
    PhysicalRecoveryPublicationSettlementFailure,
};
use super::materialization::MaterializedCandidate;
use outcome::{attach_materialization, denied};

mod outcome;

#[allow(clippy::too_many_arguments)]
pub(super) fn execute(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: &PhysicalRecoveryPublicationCommand,
    candidate: &PhysicalRecoveryPublicationCandidate,
    ordinal: u64,
    materialized: MaterializedCandidate,
) -> Result<
    (
        Vec<CompletedPhysicalRecoveryPublicationCandidate>,
        CompletedPhysicalRecoveryPublicationCandidate,
    ),
    PhysicalRecoveryPublicationCommandOutcome,
> {
    let stage = PhysicalRecoveryPublicationCommandStage::CandidateSynchronization;
    let MaterializedCandidate {
        completed,
        materialization,
    } = materialized;
    let work = match admission::admit(
        coordination,
        stage,
        PhysicalWorkScope::artifact(candidate.artifact()),
    ) {
        Ok(work) => work,
        Err(outcome) => return Err(attach_materialization(outcome, completed, materialization)),
    };
    let work_identity = work.intent().identity();
    let (dispatched, plan) = match work.into_execution_parts(None) {
        Ok(parts) => parts,
        Err(_) => return Err(denied(stage, completed, materialization, None)),
    };
    match media.synchronize_recovery_artifact_scheduled(
        candidate.artifact(),
        plan.backend_completion_binding()
            .backend_execution_binding(),
    ) {
        RecoveryStagingSynchronizationOutcome::Completed(outcome) => {
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
                PhysicalExecutorOutcome::PublicationEffectCompleted {
                    physical: crate::physical_runtime::CompletedPhysicalPublicationEffect::new(
                        physical.clone(),
                        candidate.artifact(),
                        PhysicalPublicationEffect::SynchronizeArtifact,
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
            if posture != PhysicalWorkSchedulerPosture::Executed
                || !signal_completion_is_terminal(signal)
            {
                return Err(PhysicalRecoveryPublicationCommandOutcome::Indeterminate(
                    PhysicalRecoveryPublicationCommandIndeterminate::CandidateSynchronizationSettlement {
                        artifact: candidate.artifact(),
                        physical,
                        materialization,
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
                crate::physical_runtime::PhysicalRecoveryYieldpointStage::CandidateSynchronization,
            );
            if wait.is_interrupted() {
                return Err(PhysicalRecoveryPublicationCommandOutcome::Indeterminate(
                    PhysicalRecoveryPublicationCommandIndeterminate::CandidateSynchronizationYieldpoint {
                        artifact: candidate.artifact(),
                        physical,
                        materialization,
                        completed: completed.into_boxed_slice(),
                        wait,
                    },
                ));
            }
            let synchronization: PerformedRecoveryPhysicalEffect<
                RecoveryPublicationCandidateSynchronizationAction,
            > = PerformedRecoveryPhysicalEffect::record_candidate_synchronization(
                RecoveryPublicationCandidateSynchronizationOccurrence::new(
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
            Ok((
                completed,
                CompletedPhysicalRecoveryPublicationCandidate::new(
                    materialization,
                    synchronization,
                ),
            ))
        }
        RecoveryStagingSynchronizationOutcome::DeniedBeforeEffect(failure) => {
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
                        retry: PhysicalRetryPayload::PublicationEffect(
                            PhysicalPublicationEffect::SynchronizeArtifact,
                        ),
                    },
                    PhysicalEffectRecoveryObligation::Cleared,
                ),
            );
            Err(denied(
                stage,
                completed,
                materialization,
                scheduler.as_ref().map(super::scheduler_posture),
            ))
        }
        RecoveryStagingSynchronizationOutcome::Indeterminate(outcome) => {
            let scheduler = execute_ready_queue_plan(plan, outcome.queue());
            let posture = super::scheduler_posture(&scheduler);
            let physical = outcome.physical().clone();
            let _ = settle(
                coordination,
                PhysicalExecutorDispatch::new(
                    dispatched,
                    PhysicalExecutorOutcome::PublicationEffectIndeterminate(
                        IndeterminatePhysicalPublicationEffect::new(
                            physical.clone(),
                            candidate.artifact(),
                            PhysicalPublicationEffect::SynchronizeArtifact,
                        ),
                    ),
                    PhysicalEffectRecoveryObligation::Retained,
                ),
            );
            Err(PhysicalRecoveryPublicationCommandOutcome::Indeterminate(
                PhysicalRecoveryPublicationCommandIndeterminate::CandidateSynchronization {
                    artifact: candidate.artifact(),
                    physical,
                    materialization,
                    completed: completed.into_boxed_slice(),
                    scheduler: Some(posture),
                },
            ))
        }
    }
}

use super::super::admission;
