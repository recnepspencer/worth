use worth_store_io_scheduler::{execute_ready_queue_plan, QueueExecutionReadyPlan};
use worth_store_physical_backend::{
    ArtifactAppendRange, ArtifactNewWriteRange, ArtifactTreeDirectory, ArtifactTreeFile,
    ArtifactTreeMedia, BackendQueueExecutionAdaptation, MediaOperationRole,
    ScheduledArtifactAppendOutcome, ScheduledArtifactNewWriteOutcome,
    ScheduledArtifactTreePublicationEffectOutcome,
};

use super::{recovery_obligation::scheduler_recovery, PhysicalWorkExecutor};
use crate::physical_runtime::work::{
    CompletedPhysicalCheckpointAction, IndeterminatePhysicalCheckpointAction,
    PhysicalCheckpointCommandPayload, PhysicalCheckpointExecutorCommand,
    PhysicalCheckpointRecoveryAction, PhysicalCheckpointWorkAction, PhysicalRetryPayload,
};
use crate::physical_runtime::{
    PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch, PhysicalExecutorOutcome,
    PhysicalWorkRecoveryTarget,
};

impl PhysicalWorkExecutor {
    pub(super) fn dispatch_checkpoint(
        &self,
        command: PhysicalCheckpointExecutorCommand,
    ) -> Result<PhysicalExecutorDispatch, crate::physical_runtime::PhysicalWorkPreEffectDenial>
    {
        let PhysicalCheckpointExecutorCommand {
            work,
            payload,
            payload_digest,
        } = command;
        let scope = work
            .intent()
            .scope()
            .checkpoint_target()
            .expect("checkpoint commands carry exact checkpoint scope");
        let action = scope.action();
        let recovery_action = PhysicalCheckpointRecoveryAction::from(action);
        let target = PhysicalWorkRecoveryTarget::Checkpoint {
            sequence: scope.checkpoint().sequence().get(),
            action: recovery_action,
        };
        let (dispatched, plan) = work.into_execution_parts(payload_digest)?;
        let prepared = self.prepare_effect_recovery(&dispatched, target, payload_digest)?;
        let (candidate, published, namespace) = checkpoint_artifacts(scope.checkpoint());
        let tree = self.media.artifact_tree();
        let (outcome, physical_recovery) = match action {
            PhysicalCheckpointWorkAction::CreateCandidate { byte_count } => create_candidate(
                tree,
                plan,
                candidate,
                byte_count,
                payload.expect("checkpoint creation carries exact bytes"),
                recovery_action,
            ),
            PhysicalCheckpointWorkAction::AppendCandidate { offset, byte_count } => {
                append_candidate(
                    tree,
                    plan,
                    candidate,
                    offset,
                    byte_count,
                    payload.expect("checkpoint append carries exact bytes"),
                    recovery_action,
                )
            }
            PhysicalCheckpointWorkAction::SynchronizeCandidate => {
                synchronize_candidate(tree, plan, candidate, recovery_action)
            }
            PhysicalCheckpointWorkAction::RemoveCandidate => {
                remove_candidate(tree, plan, candidate, recovery_action)
            }
            PhysicalCheckpointWorkAction::PublishCandidate => {
                publish_candidate(tree, plan, candidate, published, recovery_action)
            }
            PhysicalCheckpointWorkAction::SynchronizeNamespace => {
                synchronize_namespace(tree, plan, namespace, recovery_action)
            }
        };
        let recovery = self.finish_effect_recovery(prepared, physical_recovery);
        Ok(PhysicalExecutorDispatch::new(dispatched, outcome, recovery))
    }
}

fn create_candidate(
    tree: ArtifactTreeMedia<'_>,
    plan: QueueExecutionReadyPlan,
    candidate: ArtifactTreeFile,
    byte_count: u64,
    payload: PhysicalCheckpointCommandPayload,
    action: PhysicalCheckpointRecoveryAction,
) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
    let range = ArtifactNewWriteRange::new(byte_count)
        .expect("checkpoint command validation proved nonzero creation bytes");
    match tree.write_scheduled_new_exact(
        &candidate,
        range,
        &payload,
        plan.backend_completion_binding()
            .backend_execution_binding(),
        BackendQueueExecutionAdaptation::None,
    ) {
        ScheduledArtifactNewWriteOutcome::Completed(completed) => {
            let physical = completed.physical();
            completed_checkpoint(
                plan,
                completed.queue(),
                action,
                physical.write_operation(),
                MediaOperationRole::PositionedWrite,
                physical.completed_bytes(),
            )
        }
        ScheduledArtifactNewWriteOutcome::DeniedBeforeEffect(failure) => {
            denied_checkpoint(failure, Some(payload))
        }
        ScheduledArtifactNewWriteOutcome::Indeterminate(physical) => {
            let role = if physical.write_operation().is_some() {
                MediaOperationRole::PositionedWrite
            } else {
                MediaOperationRole::CreateNew
            };
            let operation = physical
                .write_operation()
                .unwrap_or_else(|| physical.create_operation());
            indeterminate_checkpoint(
                action,
                operation,
                role,
                physical.completed_bytes(),
                physical.failure(),
            )
        }
    }
}

fn append_candidate(
    tree: ArtifactTreeMedia<'_>,
    plan: QueueExecutionReadyPlan,
    candidate: ArtifactTreeFile,
    offset: u64,
    byte_count: u64,
    payload: PhysicalCheckpointCommandPayload,
    action: PhysicalCheckpointRecoveryAction,
) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
    let range = ArtifactAppendRange::new(offset, byte_count)
        .expect("checkpoint command validation proved a nonempty append interval");
    match tree.append_scheduled_artifact_exact_at(
        &candidate,
        range,
        &payload,
        plan.backend_completion_binding()
            .backend_execution_binding(),
        BackendQueueExecutionAdaptation::None,
    ) {
        ScheduledArtifactAppendOutcome::Completed(completed) => {
            let physical = completed.physical();
            completed_checkpoint(
                plan,
                completed.queue(),
                action,
                physical.operation(),
                MediaOperationRole::PositionedWrite,
                physical.range().byte_count(),
            )
        }
        ScheduledArtifactAppendOutcome::DeniedBeforeEffect(failure) => {
            denied_checkpoint(failure, Some(payload))
        }
        ScheduledArtifactAppendOutcome::Indeterminate(physical) => indeterminate_checkpoint(
            action,
            physical.operation(),
            MediaOperationRole::PositionedWrite,
            physical.completed_bytes(),
            physical.failure(),
        ),
    }
}

fn synchronize_candidate(
    tree: ArtifactTreeMedia<'_>,
    plan: QueueExecutionReadyPlan,
    candidate: ArtifactTreeFile,
    action: PhysicalCheckpointRecoveryAction,
) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
    let physical = tree.synchronize_scheduled_file(
        &candidate,
        plan.backend_completion_binding()
            .backend_execution_binding(),
        BackendQueueExecutionAdaptation::None,
    );
    publication_effect(
        plan,
        physical,
        action,
        MediaOperationRole::SynchronizeFileState,
    )
}

fn remove_candidate(
    tree: ArtifactTreeMedia<'_>,
    plan: QueueExecutionReadyPlan,
    candidate: ArtifactTreeFile,
    action: PhysicalCheckpointRecoveryAction,
) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
    let physical = tree.remove_scheduled_file_durably(
        &candidate,
        plan.backend_completion_binding()
            .backend_execution_binding(),
        BackendQueueExecutionAdaptation::None,
    );
    publication_effect(plan, physical, action, MediaOperationRole::Delete)
}

fn publish_candidate(
    tree: ArtifactTreeMedia<'_>,
    plan: QueueExecutionReadyPlan,
    candidate: ArtifactTreeFile,
    published: ArtifactTreeFile,
    action: PhysicalCheckpointRecoveryAction,
) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
    let physical = tree.replace_scheduled(
        &candidate,
        &published,
        plan.backend_completion_binding()
            .backend_execution_binding(),
        BackendQueueExecutionAdaptation::None,
    );
    publication_effect(plan, physical, action, MediaOperationRole::AtomicReplace)
}

fn synchronize_namespace(
    tree: ArtifactTreeMedia<'_>,
    plan: QueueExecutionReadyPlan,
    namespace: ArtifactTreeDirectory,
    action: PhysicalCheckpointRecoveryAction,
) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
    let physical = tree.synchronize_scheduled_directory(
        &namespace,
        plan.backend_completion_binding()
            .backend_execution_binding(),
        BackendQueueExecutionAdaptation::None,
    );
    publication_effect(
        plan,
        physical,
        action,
        MediaOperationRole::SynchronizeDirectoryPublication,
    )
}

fn publication_effect(
    plan: QueueExecutionReadyPlan,
    physical: ScheduledArtifactTreePublicationEffectOutcome,
    action: PhysicalCheckpointRecoveryAction,
    role: MediaOperationRole,
) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
    match physical {
        ScheduledArtifactTreePublicationEffectOutcome::Completed(completed) => {
            let operation = completed.physical().operation();
            completed_checkpoint(plan, completed.queue(), action, operation, role, 0)
        }
        ScheduledArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(failure) => {
            denied_checkpoint(failure, None)
        }
        ScheduledArtifactTreePublicationEffectOutcome::Indeterminate(physical) => {
            indeterminate_checkpoint(action, physical.operation(), role, 0, physical.failure())
        }
    }
}

fn completed_checkpoint(
    plan: QueueExecutionReadyPlan,
    queue: worth_store_physical_backend::BackendQueueExecutionCompletion,
    action: PhysicalCheckpointRecoveryAction,
    operation: worth_store_physical_backend::MediaOperationIdentity,
    role: MediaOperationRole,
    completed_bytes: u64,
) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
    let scheduler = execute_ready_queue_plan(plan, queue);
    let recovery = scheduler_recovery(&scheduler);
    (
        PhysicalExecutorOutcome::CheckpointCompleted {
            physical: CompletedPhysicalCheckpointAction::new(
                action,
                operation,
                role,
                completed_bytes,
            ),
            scheduler,
        },
        recovery,
    )
}

fn denied_checkpoint(
    failure: worth_store_physical_backend::ArtifactTreeFailure,
    payload: Option<PhysicalCheckpointCommandPayload>,
) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
    (
        PhysicalExecutorOutcome::DeniedBeforeEffect {
            failure,
            retry: PhysicalRetryPayload::Checkpoint { payload },
        },
        PhysicalEffectRecoveryObligation::Cleared,
    )
}

fn indeterminate_checkpoint(
    action: PhysicalCheckpointRecoveryAction,
    operation: worth_store_physical_backend::MediaOperationIdentity,
    role: MediaOperationRole,
    completed_bytes: u64,
    failure: worth_store_physical_backend::ArtifactTreeFailure,
) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
    (
        PhysicalExecutorOutcome::CheckpointIndeterminate(
            IndeterminatePhysicalCheckpointAction::new(
                action,
                operation,
                role,
                completed_bytes,
                failure,
            ),
        ),
        PhysicalEffectRecoveryObligation::Retained,
    )
}

fn checkpoint_artifacts(
    identity: worth_store_physical_format::PhysicalCheckpointIdentity,
) -> (ArtifactTreeFile, ArtifactTreeFile, ArtifactTreeDirectory) {
    let namespace = ArtifactTreeDirectory::families();
    let candidate = ArtifactTreeDirectory::staging()
        .file(&format!(
            "checkpoint-{:016x}.candidate",
            identity.sequence().get()
        ))
        .expect("canonical checkpoint candidate name is portable");
    let published = namespace
        .file("checkpoint.current")
        .expect("canonical checkpoint publication name is portable");
    (candidate, published, namespace)
}
