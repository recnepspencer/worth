use worth_store_io_scheduler::execute_ready_queue_plan;
use worth_store_wal::WalSegmentArtifactIdentity;

use super::*;
use crate::physical_runtime::recovery_coordination::settlement::{
    scheduler_posture, settle, signal_completion_is_terminal,
};
use crate::physical_runtime::recovery_coordination::{
    RecoveryCleanupRemovalBinding, RecoveryCleanupRemovalOccurrence,
    RecoveryCleanupRemovalSettlement, RecoveryCleanupRemovalTarget,
};
use crate::physical_runtime::work::{
    CompletedPhysicalWalReclamationAction, IndeterminatePhysicalWalReclamationAction,
    PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch, PhysicalExecutorOutcome,
    PhysicalRetryPayload, PhysicalWalReclamationScope,
};
use crate::physical_runtime::{PhysicalRecoveryCoordination, PhysicalWorkSchedulerPosture};

struct RemovalExecution {
    dispatched: crate::physical_runtime::DispatchedPhysicalWork,
    plan: worth_store_io_scheduler::QueueExecutionReadyPlan,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    artifact: WalSegmentArtifactIdentity,
}

struct CompletedRemovalSettlement {
    physical: crate::physical_runtime::CompletedRecoveryCleanupPhysicalRemoval,
    admission: [u8; 32],
    revalidation: crate::physical_runtime::RecoveryCleanupArtifactRevalidationProgress,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    artifact: WalSegmentArtifactIdentity,
    posture: PhysicalWorkSchedulerPosture,
    signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
}

pub(in crate::physical_runtime::recovery_coordination::cleanup) fn execute(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: PhysicalRecoveryCleanupRemovalCommand,
) -> PhysicalRecoveryCleanupRemovalOutcome {
    let execution = match admit_execution(coordination, media, &command) {
        Ok(execution) => execution,
        Err(outcome) => return outcome,
    };
    let binding = super::binding::effect(coordination, &command, execution.work);
    let selector = match super::binding::admit_selector(&command) {
        Ok(selector) => selector,
        Err(integrity) => return deny_invalid_binding(coordination, execution, Some(integrity)),
    };
    coordination
        .root_protocol_counters
        .observe_selector(crate::physical_runtime::PhysicalRootProtocolRoute::CleanupRemoval);
    let root = match super::binding::admit_root_manifest(&command) {
        Ok(root) => root,
        Err(integrity) => return deny_invalid_binding(coordination, execution, Some(integrity)),
    };
    coordination
        .root_protocol_counters
        .observe_root(crate::physical_runtime::PhysicalRootProtocolRoute::CleanupRemoval);
    if !super::binding::matches(&command, execution.work, binding, selector, root) {
        return deny_invalid_binding(coordination, execution, None);
    }
    let Some(request) = super::request::from_command(&command, binding.identity()) else {
        return deny_invalid_binding(coordination, execution, None);
    };
    match worth_store_physical_backend::execute_recovery_cleanup_removal(
        media,
        request,
        execution
            .plan
            .backend_completion_binding()
            .backend_execution_binding(),
    ) {
        worth_store_physical_backend::BackendRecoveryCleanupRemovalOutcome::Completed(
            completed,
        ) => {
            let physical =
                crate::physical_runtime::CompletedRecoveryCleanupPhysicalRemoval::from_backend(
                    command.artifact,
                    *completed,
                );
            let wait = coordination
                .pause_at(crate::physical_runtime::PhysicalRecoveryYieldpointStage::CleanupRemoval);
            if wait.is_interrupted() {
                let settlement =
                    settle_completed_removal(coordination, &command, execution, physical);
                return PhysicalRecoveryCleanupRemovalOutcome::Indeterminate(
                    PhysicalRecoveryCleanupRemovalIndeterminate::Yieldpoint {
                        revalidation: settlement.revalidation,
                        physical: settlement.physical,
                        wait,
                    },
                );
            }
            complete_removal(coordination, command, execution, physical)
        }
        worth_store_physical_backend::BackendRecoveryCleanupRemovalOutcome::DeniedBeforeEffect(
            denied,
        ) => {
            let physical =
                crate::physical_runtime::DeniedRecoveryCleanupPhysicalRemoval::from_backend(
                    command.artifact,
                    *denied,
                );
            deny_removal(coordination, execution, Box::new(physical))
        }
        worth_store_physical_backend::BackendRecoveryCleanupRemovalOutcome::Indeterminate(
            indeterminate,
        ) => {
            let physical =
                crate::physical_runtime::IndeterminateRecoveryCleanupPhysicalRemoval::from_backend(
                    command.artifact,
                    *indeterminate,
                );
            indeterminate_removal(coordination, command, execution, Box::new(physical))
        }
    }
}

fn deny_invalid_binding(
    coordination: &PhysicalRecoveryCoordination,
    execution: RemovalExecution,
    integrity: Option<crate::physical_runtime::RootProtocolAdmissionDenial>,
) -> PhysicalRecoveryCleanupRemovalOutcome {
    let signal = settle(
        coordination,
        PhysicalExecutorDispatch::new(
            execution.dispatched,
            PhysicalExecutorOutcome::DeniedBeforeEffect {
                failure: worth_store_physical_backend::ArtifactTreeFailure::recovery_denial(),
                retry: PhysicalRetryPayload::WalReclamation,
            },
            PhysicalEffectRecoveryObligation::Cleared,
        ),
    );
    PhysicalRecoveryCleanupRemovalOutcome::DeniedBeforeEffect(
        PhysicalRecoveryCleanupRemovalDenial {
            kind: PhysicalRecoveryCleanupRemovalDenialKind::InvalidCommand,
            physical: None,
            work: Some(execution.work),
            scheduler: None,
            signal: Some(signal),
            integrity,
        },
    )
}

fn admit_execution(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: &PhysicalRecoveryCleanupRemovalCommand,
) -> Result<RemovalExecution, PhysicalRecoveryCleanupRemovalOutcome> {
    let scope =
        validated_scope(coordination, media, command).ok_or_else(invalid_command_outcome)?;
    let work =
        super::super::admission::removal(coordination, scope).map_err(admission_denial_outcome)?;
    let work_identity = work.intent().identity();
    let (dispatched, plan) = work
        .into_execution_parts(None)
        .map_err(|denial| execution_denial_outcome(denial, work_identity))?;
    Ok(RemovalExecution {
        dispatched,
        plan,
        work: work_identity,
        artifact: command.artifact,
    })
}

fn validated_scope(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: &PhysicalRecoveryCleanupRemovalCommand,
) -> Option<PhysicalWalReclamationScope> {
    if command.store != media.store_identity()
        || command.media_generation != media.media_generation()
        || command.session != coordination.session_identity()
    {
        return None;
    }
    PhysicalWalReclamationScope::new(
        command.checkpoint,
        command.compaction_generation,
        command.compaction_digest,
        command.retained_boundary,
        command.artifact,
        command.lsn_range,
        command.byte_count,
    )
}

fn invalid_command_outcome() -> PhysicalRecoveryCleanupRemovalOutcome {
    denied_without_physical(
        PhysicalRecoveryCleanupRemovalDenialKind::InvalidCommand,
        None,
    )
}

fn admission_denial_outcome(
    denial: super::super::admission::PhysicalRecoveryCleanupAdmissionDenial,
) -> PhysicalRecoveryCleanupRemovalOutcome {
    denied_without_physical(
        PhysicalRecoveryCleanupRemovalDenialKind::Admission(denial),
        None,
    )
}

fn execution_denial_outcome(
    denial: crate::physical_runtime::PhysicalWorkPreEffectDenial,
    work: crate::physical_runtime::PhysicalWorkIdentity,
) -> PhysicalRecoveryCleanupRemovalOutcome {
    denied_without_physical(
        PhysicalRecoveryCleanupRemovalDenialKind::Execution(denial),
        Some(work),
    )
}

fn denied_without_physical(
    kind: PhysicalRecoveryCleanupRemovalDenialKind,
    work: Option<crate::physical_runtime::PhysicalWorkIdentity>,
) -> PhysicalRecoveryCleanupRemovalOutcome {
    PhysicalRecoveryCleanupRemovalOutcome::DeniedBeforeEffect(
        PhysicalRecoveryCleanupRemovalDenial {
            kind,
            physical: None,
            work,
            scheduler: None,
            signal: None,
            integrity: None,
        },
    )
}

fn complete_removal(
    coordination: &PhysicalRecoveryCoordination,
    command: PhysicalRecoveryCleanupRemovalCommand,
    execution: RemovalExecution,
    completed: crate::physical_runtime::CompletedRecoveryCleanupPhysicalRemoval,
) -> PhysicalRecoveryCleanupRemovalOutcome {
    let settlement = settle_completed_removal(coordination, &command, execution, completed);
    if settlement.posture != PhysicalWorkSchedulerPosture::Executed {
        return PhysicalRecoveryCleanupRemovalOutcome::Indeterminate(
            PhysicalRecoveryCleanupRemovalIndeterminate::Scheduler {
                physical: settlement.physical,
                revalidation: settlement.revalidation,
                posture: settlement.posture,
                signal: settlement.signal,
            },
        );
    }
    if !signal_completion_is_terminal(settlement.signal) {
        return PhysicalRecoveryCleanupRemovalOutcome::Indeterminate(
            PhysicalRecoveryCleanupRemovalIndeterminate::Signal {
                physical: settlement.physical,
                revalidation: settlement.revalidation,
                posture: settlement.posture,
                outcome: settlement.signal,
            },
        );
    }
    let performed = PerformedRecoveryPhysicalEffect::record_cleanup_removal(
        RecoveryCleanupRemovalOccurrence::new(
            RecoveryCleanupRemovalBinding::new(
                coordination.session_identity(),
                command.plan,
                command.published_generation,
                command.checkpoint,
            ),
            RecoveryCleanupRemovalTarget::new(
                settlement.artifact,
                command.lsn_range,
                command.byte_count,
            ),
            RecoveryCleanupRemovalSettlement::new(
                settlement.physical,
                settlement.admission,
                settlement.work,
                settlement.posture,
                settlement.signal,
            ),
        ),
    );
    coordination
        .root_protocol_counters
        .observe_publication(crate::physical_runtime::PhysicalRootProtocolRoute::CleanupRemoval);
    PhysicalRecoveryCleanupRemovalOutcome::Completed(CompletedPhysicalRecoveryCleanupRemoval {
        performed,
        revalidation: settlement.revalidation,
    })
}

fn settle_completed_removal(
    coordination: &PhysicalRecoveryCoordination,
    command: &PhysicalRecoveryCleanupRemovalCommand,
    execution: RemovalExecution,
    completed: crate::physical_runtime::CompletedRecoveryCleanupPhysicalRemoval,
) -> CompletedRemovalSettlement {
    let physical = completed;
    let admission = physical.admission();
    let revalidation = physical.revalidation();
    let queue = physical.queue();
    #[cfg(feature = "certification-test-authority")]
    let queue = if coordination.take_certification_cleanup_scheduler_failure(
        super::super::PhysicalRecoveryCleanupCommandStage::Removal,
    ) {
        queue.with_foreign_plan_binding_for_certification()
    } else {
        queue
    };
    let scheduler = execute_ready_queue_plan(execution.plan, queue);
    let posture = scheduler_posture(&scheduler);
    let dispatch = PhysicalExecutorDispatch::new(
        execution.dispatched,
        PhysicalExecutorOutcome::WalReclamationCompleted {
            physical: CompletedPhysicalWalReclamationAction::new(
                command.checkpoint,
                command.artifact,
                command.lsn_range,
                command.byte_count,
                physical.operation(),
            ),
            scheduler,
        },
        PhysicalEffectRecoveryObligation::Cleared,
    );
    CompletedRemovalSettlement {
        physical,
        admission,
        revalidation,
        work: execution.work,
        artifact: execution.artifact,
        posture,
        signal: settle(coordination, dispatch),
    }
}

fn deny_removal(
    coordination: &PhysicalRecoveryCoordination,
    execution: RemovalExecution,
    denied: Box<crate::physical_runtime::DeniedRecoveryCleanupPhysicalRemoval>,
) -> PhysicalRecoveryCleanupRemovalOutcome {
    let cause = denied.cause();
    let scheduler = denied
        .queue()
        .map(|queue| scheduler_posture(&execute_ready_queue_plan(execution.plan, queue)));
    let signal = settle(
        coordination,
        PhysicalExecutorDispatch::new(
            execution.dispatched,
            PhysicalExecutorOutcome::DeniedBeforeEffect {
                failure: denied.failure(),
                retry: PhysicalRetryPayload::WalReclamation,
            },
            PhysicalEffectRecoveryObligation::Cleared,
        ),
    );
    PhysicalRecoveryCleanupRemovalOutcome::DeniedBeforeEffect(
        PhysicalRecoveryCleanupRemovalDenial {
            kind: PhysicalRecoveryCleanupRemovalDenialKind::Media(cause),
            physical: Some(*denied),
            work: Some(execution.work),
            scheduler,
            signal: Some(signal),
            integrity: None,
        },
    )
}

fn indeterminate_removal(
    coordination: &PhysicalRecoveryCoordination,
    command: PhysicalRecoveryCleanupRemovalCommand,
    execution: RemovalExecution,
    indeterminate: Box<crate::physical_runtime::IndeterminateRecoveryCleanupPhysicalRemoval>,
) -> PhysicalRecoveryCleanupRemovalOutcome {
    let scheduler = execute_ready_queue_plan(execution.plan, indeterminate.queue());
    let posture = scheduler_posture(&scheduler);
    let physical = &indeterminate;
    let signal = settle(
        coordination,
        PhysicalExecutorDispatch::new(
            execution.dispatched,
            PhysicalExecutorOutcome::WalReclamationIndeterminate(
                IndeterminatePhysicalWalReclamationAction::new(
                    command.checkpoint,
                    command.artifact,
                    physical.operation(),
                    physical.failure(),
                ),
            ),
            PhysicalEffectRecoveryObligation::Cleared,
        ),
    );
    PhysicalRecoveryCleanupRemovalOutcome::Indeterminate(
        PhysicalRecoveryCleanupRemovalIndeterminate::Media {
            physical: *indeterminate,
            scheduler: posture,
            signal,
        },
    )
}
