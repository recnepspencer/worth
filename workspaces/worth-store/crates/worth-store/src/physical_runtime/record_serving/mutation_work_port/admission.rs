use worth_proof::TransitionOutcome;
use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;
use worth_store_physical_format::RecordFrameCoordinate;

use crate::physical_runtime::{
    PhysicalExecutorCommand, PhysicalMutationWorkRequest, PhysicalSchedulerDemand,
    PhysicalWorkAdmission, PhysicalWorkReadiness, PhysicalWorkScope, RecordPublicationStage,
};

use super::{
    CanonicalRecordMutationFailure, CanonicalRecordMutationPort, PreparedCanonicalRecordMutation,
};

impl CanonicalRecordMutationPort {
    pub(in crate::physical_runtime) fn prepare_new_artifact(
        &self,
        stage: RecordPublicationStage,
        coordinate: RecordFrameCoordinate,
        payload: impl Into<Box<[u8]>>,
    ) -> Result<PreparedCanonicalRecordMutation, CanonicalRecordMutationFailure> {
        let work = self.admit_range(stage, coordinate)?;
        let identity = work.intent().identity();
        let command = PhysicalExecutorCommand::new_artifact(work, payload)
            .map_err(|failure| CanonicalRecordMutationFailure::command(identity, failure))?;
        Ok(self.prepared(
            command,
            crate::physical_runtime::PhysicalWorkRecoveryTarget::Range(coordinate),
        ))
    }

    fn admit_range(
        &self,
        stage: RecordPublicationStage,
        coordinate: RecordFrameCoordinate,
    ) -> Result<crate::physical_runtime::ResourceAdmittedPhysicalWork, CanonicalRecordMutationFailure>
    {
        let runtime = self.runtime()?;
        let ready = self.request_ready(
            &runtime,
            stage,
            PhysicalWorkScope::one(coordinate),
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )?;
        let identity = ready.intent().identity();
        let (reservation, backend) = self
            .scheduler
            .record_write(
                self.record.scheduler_security(),
                u64::from(coordinate.length()),
                false,
                true,
            )
            .map_err(|failure| {
                CanonicalRecordMutationFailure::scheduler_reservation(identity, failure)
            })?;
        self.admit_scheduler(&runtime, ready, reservation, backend)
    }

    fn runtime(
        &self,
    ) -> Result<
        std::sync::Arc<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
        CanonicalRecordMutationFailure,
    > {
        self.runtime
            .upgrade()
            .ok_or_else(CanonicalRecordMutationFailure::runtime_released)
    }

    fn request_ready(
        &self,
        runtime: &crate::physical_runtime::instance::PhysicalStoreWorkRuntime,
        stage: RecordPublicationStage,
        scope: PhysicalWorkScope,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> Result<crate::physical_runtime::ReadyPhysicalWork, CanonicalRecordMutationFailure> {
        let request = PhysicalMutationWorkRequest::publication(
            scope,
            self.record.mutation_basis(stage),
            self.record.security(),
            durability,
        )
        .map_err(|_| CanonicalRecordMutationFailure::submission_rejected())?;
        let receipt = match self.submission.submit(request).into_raw() {
            TransitionOutcome::Success(receipt) => receipt,
            _ => return Err(CanonicalRecordMutationFailure::submission_rejected()),
        };
        let identity = receipt.identity();
        let admitted = PhysicalWorkAdmission::admit(
            &runtime.submission,
            receipt,
            &self.physical,
            &runtime.health,
        )
        .map_err(|failure| CanonicalRecordMutationFailure::pre_effect(identity, failure))?;
        match runtime
            .signal
            .request(admitted)
            .map_err(|failure| CanonicalRecordMutationFailure::pre_effect(identity, failure))?
        {
            PhysicalWorkReadiness::Ready(ready) => Ok(ready),
            PhysicalWorkReadiness::Blocked(_) => {
                Err(CanonicalRecordMutationFailure::dependency_blocked(identity))
            }
        }
    }

    fn admit_scheduler(
        &self,
        runtime: &crate::physical_runtime::instance::PhysicalStoreWorkRuntime,
        ready: crate::physical_runtime::ReadyPhysicalWork,
        reservation: worth_store_io_scheduler::foreground_reservation::
            PhysicalInstanceForegroundReservation,
        backend: worth_store_io_scheduler::IoSchedulerBackendCapabilityAdmission,
    ) -> Result<crate::physical_runtime::ResourceAdmittedPhysicalWork, CanonicalRecordMutationFailure>
    {
        let identity = ready.intent().identity();
        let demand = PhysicalSchedulerDemand::foreground(ready, reservation, None)
            .map_err(|failure| CanonicalRecordMutationFailure::scheduler(identity, failure))?;
        PhysicalWorkAdmission::require_current(
            &runtime.submission,
            demand.intent(),
            &runtime.health,
        )
        .map_err(|failure| CanonicalRecordMutationFailure::pre_effect(identity, failure))?;
        let policy =
            super::super::record_queue_policy::admit_record_queue_policy(demand.queue_work());
        crate::physical_runtime::PhysicalWorkScheduler::admit(
            self.scheduler.effects(),
            demand,
            &backend,
            policy,
        )
            .map_err(|failure| CanonicalRecordMutationFailure::scheduler(identity, failure))
    }

    fn prepared(
        &self,
        command: PhysicalExecutorCommand,
        target: crate::physical_runtime::PhysicalWorkRecoveryTarget,
    ) -> PreparedCanonicalRecordMutation {
        PreparedCanonicalRecordMutation {
            identity: command.identity(),
            execution: self.execution.clone(),
            command,
            target,
        }
    }
}
