use std::num::NonZeroU64;

use worth_proof::TransitionOutcome;
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use super::{
    scheduler_preparation::{admit_ready, prepare_admitted_command, require_projection_failure},
    CanonicalRecordReadFailure as Failure, CanonicalRecordReadFailureEvidence as Evidence,
    CanonicalRecordReadPort, PreparedCanonicalMetadataRead, PreparedCanonicalRecordRead,
    RecordReadPartition,
};
use crate::physical_runtime::{
    record_serving::RebuildReadShape, PhysicalExecutorCommand, PhysicalMetadataReadWorkRequest,
    PhysicalReadWorkRequest, PhysicalSchedulerDemand, PhysicalWorkScope,
};

impl CanonicalRecordReadPort {
    pub(in crate::physical_runtime::record_serving) fn for_rebuild(
        mut self,
        shape: RebuildReadShape,
    ) -> Self {
        self.rebuild = Some(shape);
        self.diagnostic_scrub = false;
        self
    }

    pub(in crate::physical_runtime::record_serving) fn for_ordinary(mut self) -> Self {
        self.rebuild = None;
        self.diagnostic_scrub = false;
        self
    }

    pub(in crate::physical_runtime::record_serving) fn read_allocation_scope(
        &self,
    ) -> worth_store_buffer_pool::PhysicalOperationAllocationScope {
        if self.diagnostic_scrub {
            worth_store_buffer_pool::PhysicalOperationAllocationScope::Scrub
        } else if self.rebuild.is_some() {
            worth_store_buffer_pool::PhysicalOperationAllocationScope::Maintenance
        } else {
            worth_store_buffer_pool::PhysicalOperationAllocationScope::ForegroundRead
        }
    }

    pub(super) fn prepare_rebuild_range(
        &self,
        coordinate: RecordFrameCoordinate,
        partition: RecordReadPartition,
        shape: RebuildReadShape,
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
        .expect("canonical rebuild coordinates carry admitted read and security bases");
        // Capacity denial precedes submission: there is no orphaned Ready work
        // and the synchronous attempt has no retained retry/fairness owner.
        let admission = self
            .scheduler
            .rebuild_read(
                shape,
                self.record.scheduler_security(),
                NonZeroU64::new(u64::from(coordinate.length())),
                0,
            )
            .map_err(|denial| Evidence::before_work(Failure::Scheduler(denial)))?;
        let receipt = match self.submission.submit(request).into_raw() {
            TransitionOutcome::Success(receipt) => receipt,
            _ => return Err(Evidence::before_work(Failure::SubmissionRejected)),
        };
        let (ready, identity) = admit_ready(&runtime, receipt, &self.physical)?;
        let admitted = PhysicalSchedulerDemand::rebuild_read(ready, admission)
            .map_err(|denial| Evidence::during_work(Failure::Scheduler(denial), identity))?;
        let prepared = prepare_admitted_command(
            &runtime,
            self.scheduler.effects(),
            identity,
            admitted,
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

    pub(super) fn prepare_rebuild_metadata(
        &self,
        artifact: RecordArtifactFile,
        partition: RecordReadPartition,
        shape: RebuildReadShape,
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
        .expect("rebuild metadata carries admitted read and security bases");
        let admission = self
            .scheduler
            .rebuild_read(shape, self.record.scheduler_security(), None, 0)
            .map_err(|denial| Evidence::before_work(Failure::Scheduler(denial)))?;
        let receipt = match self.submission.submit_metadata(request).into_raw() {
            TransitionOutcome::Success(receipt) => receipt,
            _ => return Err(Evidence::before_work(Failure::SubmissionRejected)),
        };
        let (ready, identity) = admit_ready(&runtime, receipt, &self.physical)?;
        let admitted = PhysicalSchedulerDemand::rebuild_read(ready, admission)
            .map_err(|denial| Evidence::during_work(Failure::Scheduler(denial), identity))?;
        let prepared = prepare_admitted_command(
            &runtime,
            self.scheduler.effects(),
            identity,
            admitted,
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
