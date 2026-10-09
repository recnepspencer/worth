use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::{
    CanonicalRecordMutationFailure, CanonicalRecordMutationPort, PreparedCanonicalRecordMutation,
};
use crate::physical_runtime::{
    PhysicalExecutorCommand, PhysicalSchedulerDemand, PhysicalWorkAdmission, PhysicalWorkScope,
    RecordPublicationStage,
};

impl CanonicalRecordMutationPort {
    pub(in crate::physical_runtime::record_serving) fn cancel_blob_movement_background(&self) {
        self.scheduler.cancel_blob_movement_background_head();
    }

    pub(in crate::physical_runtime::record_serving) fn admit_blob_movement_frame(
        &self,
        bytes: u64,
    ) -> Result<
        crate::physical_runtime::instance::BlobMovementFrameAdmission,
        crate::physical_runtime::PhysicalSchedulerDenial,
    > {
        self.scheduler
            .blob_movement_background(self.record.scheduler_security(), bytes, 0)
    }

    pub(in crate::physical_runtime::record_serving) fn prepare_blob_movement_artifact(
        &self,
        stage: RecordPublicationStage,
        coordinate: RecordFrameCoordinate,
        payload: &[u8],
        admission: crate::physical_runtime::instance::BlobMovementFrameAdmission,
    ) -> Result<PreparedCanonicalRecordMutation, CanonicalRecordMutationFailure> {
        if !matches!(
            coordinate.artifact(),
            RecordArtifactFile::ExtentArena { .. }
        ) {
            return Err(CanonicalRecordMutationFailure::submission_rejected());
        }
        let runtime = self.runtime()?;
        let ready = self.request_ready(
            &runtime,
            stage,
            PhysicalWorkScope::one(coordinate),
            ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )?;
        let identity = ready.intent().identity();
        let admission = admission
            .require_length(u64::from(coordinate.length()))
            .map_err(|failure| CanonicalRecordMutationFailure::scheduler(identity, failure))?;
        let demand = PhysicalSchedulerDemand::blob_movement_background(
            ready,
            admission.lease,
            admission.capacity,
        )
        .map_err(|failure| CanonicalRecordMutationFailure::scheduler(identity, failure))?;
        PhysicalWorkAdmission::require_current(
            &runtime.submission,
            demand.intent(),
            &runtime.health,
        )
        .map_err(|failure| CanonicalRecordMutationFailure::pre_effect(identity, failure))?;
        let work = crate::physical_runtime::PhysicalWorkScheduler::admit(
            self.scheduler.effects(),
            demand,
            &admission.backend,
            admission.policy,
        )
        .map_err(|failure| CanonicalRecordMutationFailure::scheduler(identity, failure))?;
        let command = PhysicalExecutorCommand::new_artifact(work, payload)
            .map_err(|failure| CanonicalRecordMutationFailure::command(identity, failure))?;
        Ok(self.prepared(
            command,
            crate::physical_runtime::PhysicalWorkRecoveryTarget::Range(coordinate),
        ))
    }
}
