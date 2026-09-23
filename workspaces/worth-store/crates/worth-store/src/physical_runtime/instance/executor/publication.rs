use worth_store_io_scheduler::execute_ready_queue_plan;
use worth_store_physical_backend::{
    BackendQueueExecutionAdaptation, ScheduledArtifactNewWriteOutcome,
    ScheduledArtifactTreePublicationEffectOutcome,
};

use super::{recovery_obligation::scheduler_recovery, PhysicalWorkExecutor};
use crate::physical_runtime::{
    record_serving::residency::artifact_tree::PhysicalRecordArtifactTree,
    PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch, PhysicalExecutorOutcome,
    PhysicalPublicationEffect, PhysicalPublicationExecutorCommand, PhysicalWriteExecutorCommand,
};

impl PhysicalWorkExecutor {
    pub(super) fn dispatch_new_artifact(
        &self,
        command: PhysicalWriteExecutorCommand,
    ) -> Result<PhysicalExecutorDispatch, crate::physical_runtime::PhysicalWorkPreEffectDenial>
    {
        let PhysicalWriteExecutorCommand {
            work,
            coordinate,
            payload,
            payload_digest,
        } = command;
        let (dispatched, plan) = work.into_execution_parts(Some(payload_digest))?;
        let prepared = self.prepare_effect_recovery(
            &dispatched,
            crate::physical_runtime::PhysicalWorkRecoveryTarget::Range(coordinate),
            Some(payload_digest),
        )?;
        let physical = PhysicalRecordArtifactTree::new(&self.media).write_scheduled_new_exact(
            coordinate,
            &payload,
            plan.backend_completion_binding()
                .backend_execution_binding(),
            BackendQueueExecutionAdaptation::None,
        );
        let (outcome, physical_recovery) = match physical {
            ScheduledArtifactNewWriteOutcome::Completed(completed) => {
                let physical = completed.physical().clone();
                let scheduler = execute_ready_queue_plan(plan, completed.queue());
                let recovery = scheduler_recovery(&scheduler);
                (
                    PhysicalExecutorOutcome::NewArtifactCompleted {
                        physical,
                        coordinate,
                        scheduler,
                    },
                    recovery,
                )
            }
            ScheduledArtifactNewWriteOutcome::DeniedBeforeEffect(failure) => (
                PhysicalExecutorOutcome::DeniedBeforeEffect {
                    failure,
                    retry: crate::physical_runtime::work::PhysicalRetryPayload::NewArtifact(
                        payload,
                    ),
                },
                PhysicalEffectRecoveryObligation::Cleared,
            ),
            ScheduledArtifactNewWriteOutcome::Indeterminate(physical) => (
                PhysicalExecutorOutcome::NewArtifactIndeterminate {
                    physical,
                    coordinate,
                },
                PhysicalEffectRecoveryObligation::Retained,
            ),
        };
        let recovery = self.finish_effect_recovery(prepared, physical_recovery);
        Ok(PhysicalExecutorDispatch::new(dispatched, outcome, recovery))
    }

    pub(super) fn dispatch_publication_effect(
        &self,
        command: PhysicalPublicationExecutorCommand,
        root_publication: bool,
    ) -> Result<PhysicalExecutorDispatch, crate::physical_runtime::PhysicalWorkPreEffectDenial>
    {
        let PhysicalPublicationExecutorCommand {
            work,
            artifact,
            effect,
        } = command;
        let (dispatched, plan) = work.into_execution_parts(None)?;
        let target = publication_recovery_target(effect, artifact);
        let prepared = self.prepare_effect_recovery(&dispatched, target, None)?;
        let tree = PhysicalRecordArtifactTree::new(&self.media);
        let binding = plan
            .backend_completion_binding()
            .backend_execution_binding();
        let physical = match effect {
            PhysicalPublicationEffect::SynchronizeArtifact => tree.synchronize_scheduled_artifact(
                artifact,
                binding,
                BackendQueueExecutionAdaptation::None,
            ),
            PhysicalPublicationEffect::SynchronizeArtifactParent => tree
                .synchronize_scheduled_artifact_parent(
                    artifact,
                    binding,
                    BackendQueueExecutionAdaptation::None,
                ),
            PhysicalPublicationEffect::ReplaceCatalog => tree.replace_scheduled_catalog(
                artifact,
                binding,
                BackendQueueExecutionAdaptation::None,
            ),
            PhysicalPublicationEffect::SynchronizeRecordFamily => tree
                .synchronize_scheduled_record_family(
                    binding,
                    BackendQueueExecutionAdaptation::None,
                ),
            PhysicalPublicationEffect::RemoveArtifact => tree.remove_scheduled_artifact(
                artifact,
                binding,
                BackendQueueExecutionAdaptation::None,
            ),
        };
        #[cfg(feature = "certification-test-authority")]
        if matches!(effect, PhysicalPublicationEffect::ReplaceCatalog)
            && matches!(
                &physical,
                ScheduledArtifactTreePublicationEffectOutcome::Completed(_)
            )
        {
            self.certification_yieldpoints.pause(
                crate::physical_runtime::certification::
                    CertificationPhysicalExecutionCheckpoint::
                        AfterCatalogReplacementBeforeSchedulerSettlement,
            );
        }
        let (outcome, physical_recovery) =
            classify_publication_effect(plan, physical, artifact, effect, root_publication);
        let recovery = self.finish_effect_recovery(prepared, physical_recovery);
        Ok(PhysicalExecutorDispatch::new(dispatched, outcome, recovery))
    }
}

fn classify_publication_effect(
    plan: worth_store_io_scheduler::QueueExecutionReadyPlan,
    physical: ScheduledArtifactTreePublicationEffectOutcome,
    artifact: worth_store_physical_format::RecordArtifactFile,
    effect: PhysicalPublicationEffect,
    root_publication: bool,
) -> (PhysicalExecutorOutcome, PhysicalEffectRecoveryObligation) {
    match physical {
        ScheduledArtifactTreePublicationEffectOutcome::Completed(completed) => {
            let physical = completed.physical().clone();
            let scheduler = execute_ready_queue_plan(plan, completed.queue());
            let recovery = scheduler_recovery(&scheduler);
            (
                PhysicalExecutorOutcome::PublicationEffectCompleted {
                    physical:
                        crate::physical_runtime::work::CompletedPhysicalPublicationEffect::new(
                            physical, artifact, effect,
                        ),
                    scheduler,
                },
                recovery,
            )
        }
        ScheduledArtifactTreePublicationEffectOutcome::DeniedBeforeEffect(failure) => (
            PhysicalExecutorOutcome::DeniedBeforeEffect {
                failure,
                retry: if root_publication {
                    crate::physical_runtime::work::PhysicalRetryPayload::RootPublicationEffect
                } else {
                    crate::physical_runtime::work::PhysicalRetryPayload::PublicationEffect(effect)
                },
            },
            PhysicalEffectRecoveryObligation::Cleared,
        ),
        ScheduledArtifactTreePublicationEffectOutcome::Indeterminate(failure) => (
            PhysicalExecutorOutcome::PublicationEffectIndeterminate(
                crate::physical_runtime::work::IndeterminatePhysicalPublicationEffect::new(
                    failure, artifact, effect,
                ),
            ),
            PhysicalEffectRecoveryObligation::Retained,
        ),
    }
}

fn publication_recovery_target(
    effect: PhysicalPublicationEffect,
    artifact: worth_store_physical_format::RecordArtifactFile,
) -> crate::physical_runtime::PhysicalWorkRecoveryTarget {
    match effect {
        PhysicalPublicationEffect::SynchronizeArtifact => {
            crate::physical_runtime::PhysicalWorkRecoveryTarget::ArtifactFileSynchronization(
                artifact,
            )
        }
        PhysicalPublicationEffect::SynchronizeArtifactParent => {
            crate::physical_runtime::PhysicalWorkRecoveryTarget::ArtifactParentSynchronization(
                artifact,
            )
        }
        PhysicalPublicationEffect::ReplaceCatalog => {
            crate::physical_runtime::PhysicalWorkRecoveryTarget::CatalogReplacement(artifact)
        }
        PhysicalPublicationEffect::SynchronizeRecordFamily => {
            crate::physical_runtime::PhysicalWorkRecoveryTarget::RecordNamespaceSynchronization
        }
        PhysicalPublicationEffect::RemoveArtifact => {
            crate::physical_runtime::PhysicalWorkRecoveryTarget::ArtifactRemoval(artifact)
        }
    }
}
