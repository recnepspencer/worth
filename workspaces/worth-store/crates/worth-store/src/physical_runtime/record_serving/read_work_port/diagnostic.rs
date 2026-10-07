use worth_proof::TransitionOutcome;
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::{
    scheduler_preparation::{admit_ready, prepare_admitted_command, require_projection_failure},
    CanonicalRecordReadFailure as Failure, CanonicalRecordReadFailureEvidence as Evidence,
    CanonicalRecordReadPort, PreparedCanonicalMetadataRead, PreparedCanonicalRecordRead,
    RecordReadPartition,
};
use crate::physical_runtime::{
    instance::PhysicalScrubSchedulerAdmissionDenial, PhysicalExecutorCommand,
    PhysicalMetadataReadWorkRequest, PhysicalReadWorkRequest, PhysicalSchedulerDemand,
    PhysicalSchedulerDenial, PhysicalWorkScope,
};

impl CanonicalRecordReadPort {
    pub(in crate::physical_runtime::record_serving) fn for_diagnostic_scrub(mut self) -> Self {
        self.rebuild = None;
        self.diagnostic_scrub = true;
        self
    }

    pub(super) fn prepare_diagnostic_range(
        &self,
        coordinate: RecordFrameCoordinate,
        partition: RecordReadPartition,
    ) -> Result<PreparedCanonicalRecordRead, Evidence> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or_else(|| Evidence::before_work(Failure::RuntimeReleased))?;
        let request = PhysicalReadWorkRequest::new(
            PhysicalWorkScope::one(coordinate),
            self.record.read_basis(partition),
            self.record.security(),
        )
        .expect("protected C5 coordinate carries admitted read and security bases");
        let admission = self
            .scheduler
            .scrub_background(
                self.record.scheduler_security(),
                std::num::NonZeroU64::new(u64::from(coordinate.length())),
            )
            .map_err(|denial| {
                Evidence::before_work(Failure::Scheduler(admission_denial(denial)))
            })?;
        let receipt = match self.submission.submit(request).into_raw() {
            TransitionOutcome::Success(receipt) => receipt,
            _ => return Err(Evidence::before_work(Failure::SubmissionRejected)),
        };
        let (ready, identity) = admit_ready(&runtime, receipt, &self.physical)?;
        let (lease, capacity, backend, policy) = admission;
        let demand =
            PhysicalSchedulerDemand::scrub_selected_record_background(ready, lease, capacity)
                .map_err(|denial| Evidence::during_work(Failure::Scheduler(denial), identity))?;
        let prepared = prepare_admitted_command(
            &runtime,
            self.scheduler.effects(),
            identity,
            (demand, backend, policy),
            PhysicalExecutorCommand::read,
        )?;
        let (command, projection_failure) = require_projection_failure(prepared, identity)?;
        Ok(PreparedCanonicalRecordRead::new(
            self.execution.clone(),
            command,
            identity,
            self.execution.bind_projection_failure(projection_failure),
        ))
    }

    pub(super) fn prepare_diagnostic_metadata(
        &self,
        artifact: RecordArtifactFile,
        partition: RecordReadPartition,
    ) -> Result<PreparedCanonicalMetadataRead, Evidence> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or_else(|| Evidence::before_work(Failure::RuntimeReleased))?;
        let request = PhysicalMetadataReadWorkRequest::new(
            artifact,
            self.record.read_basis(partition),
            self.record.security(),
        )
        .expect("protected C5 metadata carries admitted read and security bases");
        let admission = self
            .scheduler
            .scrub_background(self.record.scheduler_security(), None)
            .map_err(|denial| {
                Evidence::before_work(Failure::Scheduler(admission_denial(denial)))
            })?;
        let receipt = match self.submission.submit_metadata(request).into_raw() {
            TransitionOutcome::Success(receipt) => receipt,
            _ => return Err(Evidence::before_work(Failure::SubmissionRejected)),
        };
        let (ready, identity) = admit_ready(&runtime, receipt, &self.physical)?;
        let (lease, capacity, backend, policy) = admission;
        let demand =
            PhysicalSchedulerDemand::scrub_selected_record_background(ready, lease, capacity)
                .map_err(|denial| Evidence::during_work(Failure::Scheduler(denial), identity))?;
        let prepared = prepare_admitted_command(
            &runtime,
            self.scheduler.effects(),
            identity,
            (demand, backend, policy),
            PhysicalExecutorCommand::metadata,
        )?;
        let (command, projection_failure) = require_projection_failure(prepared, identity)?;
        Ok(PreparedCanonicalMetadataRead::new(
            self.execution.clone(),
            command,
            identity,
            self.execution.bind_projection_failure(projection_failure),
        ))
    }
}

fn admission_denial(denial: PhysicalScrubSchedulerAdmissionDenial) -> PhysicalSchedulerDenial {
    match denial {
        PhysicalScrubSchedulerAdmissionDenial::Capacity(cause) => {
            PhysicalSchedulerDenial::BackgroundCapacity(cause)
        }
        PhysicalScrubSchedulerAdmissionDenial::Pacing(cause) => {
            PhysicalSchedulerDenial::BackgroundPacing(cause)
        }
        PhysicalScrubSchedulerAdmissionDenial::Deferred => {
            PhysicalSchedulerDenial::BackgroundPacingDeferred
        }
    }
}
