use worth_store_io_scheduler::execute_ready_queue_plan;
use worth_store_physical_backend::ScheduledArtifactInspectionReadOutcome;

use super::PhysicalWorkExecutor;
use crate::physical_runtime::work::PhysicalInspectionExecutorCommand;
use crate::physical_runtime::{
    PhysicalEffectRecoveryObligation, PhysicalExecutorDispatch, PhysicalExecutorOutcome,
};

impl PhysicalWorkExecutor {
    pub(super) fn dispatch_inspection(
        &self,
        command: PhysicalInspectionExecutorCommand,
    ) -> Result<PhysicalExecutorDispatch, crate::physical_runtime::PhysicalWorkPreEffectDenial>
    {
        let PhysicalInspectionExecutorCommand {
            work,
            range,
            mut destination,
        } = command;
        let (mut dispatched, plan) = work.into_execution_parts(None)?;
        let artifact = self.inspection_artifact(range.target());
        dispatched.bind_inspection_source(artifact.clone());
        let observed = self.media.artifact_tree().read_scheduled_inspection(
            &artifact,
            range,
            &mut destination,
            plan.backend_completion_binding()
                .backend_execution_binding(),
        );
        let outcome = match observed {
            ScheduledArtifactInspectionReadOutcome::Observed { physical, queue } => {
                #[cfg(feature = "certification-test-authority")]
                self.certification_yieldpoints.pause(
                    super::CertificationPhysicalExecutionCheckpoint::AfterReadBeforeSchedulerSettlement,
                );
                PhysicalExecutorOutcome::InspectionObserved {
                    physical,
                    bytes: destination,
                    scheduler: execute_ready_queue_plan(plan, queue),
                }
            }
            ScheduledArtifactInspectionReadOutcome::DeniedBeforeEffect(failure) => {
                PhysicalExecutorOutcome::InspectionDenied(failure)
            }
        };
        Ok(PhysicalExecutorDispatch::new(
            dispatched,
            outcome,
            PhysicalEffectRecoveryObligation::Cleared,
        ))
    }
}

impl PhysicalWorkExecutor {
    fn inspection_artifact(
        &self,
        target: worth_store_physical_format::PhysicalArtifactReadTarget,
    ) -> worth_store_physical_backend::ArtifactTreeFile {
        use worth_store_physical_backend::ArtifactTreeDirectory;
        use worth_store_physical_format::PhysicalArtifactReadTarget as Target;
        match target {
            Target::Record(artifact) => crate::physical_runtime::record_serving::residency::artifact_tree::PhysicalRecordArtifactTree::new(&self.media).artifact(artifact),
            Target::Wal(identity) => {
                let name = worth_store_wal::WalSegmentArtifactIdentity::new(
                    worth_store_wal::WalSegmentId::new(identity.segment().get()).expect("canonical segment"),
                    worth_store_wal::WalSegmentGeneration::new(identity.generation().get()).expect("canonical generation"),
                ).file_name();
                ArtifactTreeDirectory::families().child("wal").expect("owner WAL directory").file(&name).expect("canonical WAL filename")
            }
            Target::Checkpoint(_) => ArtifactTreeDirectory::families().file("checkpoint.current").expect("canonical checkpoint filename"),
            Target::PhysicalWork(identity) => ArtifactTreeDirectory::families().child("physical-work").expect("owner journal directory").file(
                &format!("effect-{:016x}-{:016x}-{:016x}.pending", identity.runtime().get(), identity.generation().get(), identity.operation().get())
            ).expect("canonical obligation filename"),
        }
    }
}
