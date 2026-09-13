use std::sync::{Arc, Weak};

use worth_proof::TransitionOutcome;
use worth_store_physical_backend::ArtifactTreeFailure;
use worth_store_physical_format::RecordFrameCoordinate;

use crate::physical_runtime::{
    instance::{
        PhysicalSchedulerAdmissionOwner, PhysicalStoreWorkRuntime, RecordSchedulerReservationDenial,
    },
    work::PhysicalWorkAdmissionAuthority,
    PhysicalExecutorCommand, PhysicalExecutorCommandDenial, PhysicalMetadataReadWorkRequest,
    PhysicalReadSubmission, PhysicalReadWorkRequest, PhysicalSchedulerDenial,
    PhysicalWorkExecution, PhysicalWorkIdentity, PhysicalWorkPreEffectDenial, PhysicalWorkScope,
};

use super::{
    PreparedCanonicalMetadataRead, PreparedCanonicalRecordRead, RecordReadPartition,
    RecordWorkAdmission,
};

mod inspection;
mod scheduler_preparation;
pub use inspection::PhysicalIntegrityScrubReadDeferral;

use scheduler_preparation::{admit_ready, require_projection_failure, RangeSchedulerRoute};

#[derive(Clone)]
pub(in crate::physical_runtime) struct CanonicalRecordReadPort {
    runtime: Weak<PhysicalStoreWorkRuntime>,
    execution: PhysicalWorkExecution,
    submission: PhysicalReadSubmission,
    physical: PhysicalWorkAdmissionAuthority,
    scheduler: PhysicalSchedulerAdmissionOwner,
    record: Arc<RecordWorkAdmission>,
}

struct PreparedPhysicalCommand {
    command: PhysicalExecutorCommand,
    projection_failure: Option<crate::physical_runtime::PhysicalWorkAspectDelta>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum CanonicalRecordReadFailure {
    RuntimeReleased,
    InvalidCoordinate,
    SubmissionRejected,
    PreEffect(PhysicalWorkPreEffectDenial),
    DependencyBlocked,
    SchedulerReservation(RecordSchedulerReservationDenial),
    #[cfg(feature = "certification-test-authority")]
    SecureIo(worth_store_io_scheduler::SecureIoPreservationDenial),
    Scheduler(PhysicalSchedulerDenial),
    Command(PhysicalExecutorCommandDenial),
    Backend(ArtifactTreeFailure),
    Terminal(crate::physical_runtime::PhysicalWorkTerminalCause),
    SchedulerSettlementRejected,
    SettlementMismatch,
    ProjectionFailureUnavailable,
}

impl CanonicalRecordReadFailure {
    pub(in crate::physical_runtime::record_serving) const fn work_denial(
        self,
    ) -> Option<super::RecordReadWorkDenial> {
        let denial = match self {
            Self::RuntimeReleased => super::RecordReadWorkDenial::RuntimeReleased,
            Self::InvalidCoordinate => super::RecordReadWorkDenial::InvalidCoordinate,
            Self::SubmissionRejected => super::RecordReadWorkDenial::SubmissionRejected,
            Self::PreEffect(_) => super::RecordReadWorkDenial::AdmissionRejected,
            Self::DependencyBlocked => super::RecordReadWorkDenial::DependencyBlocked,
            Self::SchedulerReservation(_) => {
                super::RecordReadWorkDenial::SchedulerReservationRejected
            }
            #[cfg(feature = "certification-test-authority")]
            Self::SecureIo(_) => super::RecordReadWorkDenial::SchedulerRejected,
            Self::Scheduler(_) => super::RecordReadWorkDenial::SchedulerRejected,
            Self::Command(_) => super::RecordReadWorkDenial::CommandRejected,
            Self::Backend(_) | Self::Terminal(_) => return None,
            Self::SchedulerSettlementRejected => {
                super::RecordReadWorkDenial::SchedulerSettlementRejected
            }
            Self::SettlementMismatch => super::RecordReadWorkDenial::SettlementMismatch,
            Self::ProjectionFailureUnavailable => super::RecordReadWorkDenial::AdmissionRejected,
        };
        Some(denial)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct CanonicalRecordReadFailureEvidence {
    failure: CanonicalRecordReadFailure,
    identity: Option<PhysicalWorkIdentity>,
}

impl CanonicalRecordReadFailureEvidence {
    const fn before_work(failure: CanonicalRecordReadFailure) -> Self {
        Self {
            failure,
            identity: None,
        }
    }

    pub(super) const fn during_work(
        failure: CanonicalRecordReadFailure,
        identity: PhysicalWorkIdentity,
    ) -> Self {
        Self {
            failure,
            identity: Some(identity),
        }
    }

    pub(in crate::physical_runtime) const fn failure(self) -> CanonicalRecordReadFailure {
        self.failure
    }

    pub(in crate::physical_runtime) const fn identity(self) -> Option<PhysicalWorkIdentity> {
        self.identity
    }
}

impl CanonicalRecordReadPort {
    pub(in crate::physical_runtime) fn new(
        runtime: &Arc<PhysicalStoreWorkRuntime>,
        generation: crate::physical_runtime::LifecycleGeneration,
        physical: PhysicalWorkAdmissionAuthority,
        scheduler: PhysicalSchedulerAdmissionOwner,
        record: Arc<RecordWorkAdmission>,
    ) -> Self {
        Self {
            runtime: Arc::downgrade(runtime),
            execution: PhysicalStoreWorkRuntime::execution(runtime, generation),
            submission: runtime.submission.read_submission(),
            physical,
            scheduler,
            record,
        }
    }

    pub(in crate::physical_runtime) fn prepare(
        &self,
        coordinate: RecordFrameCoordinate,
        partition: RecordReadPartition,
    ) -> Result<PreparedCanonicalRecordRead, CanonicalRecordReadFailureEvidence> {
        self.prepare_range(coordinate, partition, RangeSchedulerRoute::ordinary())
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn prepare_prefetch(
        &self,
        grant: &worth_store_buffer_pool::PrefetchResidencyGrant,
        partition: RecordReadPartition,
    ) -> Result<PreparedCanonicalRecordRead, CanonicalRecordReadFailureEvidence> {
        self.prepare_range(
            grant.frame().coordinate(),
            partition,
            RangeSchedulerRoute::prefetch(grant),
        )
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn prepare_read_ahead(
        &self,
        grant: &worth_store_buffer_pool::ReadAheadFrameGrant<'_, '_>,
        partition: RecordReadPartition,
    ) -> Result<PreparedCanonicalRecordRead, CanonicalRecordReadFailureEvidence> {
        self.prepare_range(
            grant.frame().coordinate(),
            partition,
            RangeSchedulerRoute::read_ahead(grant),
        )
    }

    fn prepare_range<'route, 'grant>(
        &self,
        coordinate: RecordFrameCoordinate,
        partition: RecordReadPartition,
        route: RangeSchedulerRoute<'route, 'grant>,
    ) -> Result<PreparedCanonicalRecordRead, CanonicalRecordReadFailureEvidence> {
        let runtime = self.runtime.upgrade().ok_or_else(|| {
            CanonicalRecordReadFailureEvidence::before_work(
                CanonicalRecordReadFailure::RuntimeReleased,
            )
        })?;
        let request = PhysicalReadWorkRequest::new(
            PhysicalWorkScope::one(coordinate),
            self.record.read_basis(partition),
            self.record.security(),
        )
        .expect("record coordinates, read basis, and security are admitted together");
        let receipt = match self.submission.submit(request).into_raw() {
            TransitionOutcome::Success(receipt) => receipt,
            _ => {
                return Err(CanonicalRecordReadFailureEvidence::before_work(
                    CanonicalRecordReadFailure::SubmissionRejected,
                ))
            }
        };
        let (ready, identity) = admit_ready(&runtime, receipt, &self.physical)?;
        let prepared = self.prepare_range_command(&runtime, ready, identity, coordinate, route)?;
        let (command, projection_failure) = require_projection_failure(prepared, identity)?;
        drop(runtime);
        Ok(PreparedCanonicalRecordRead::new(
            self.execution.clone(),
            command,
            identity,
            self.execution.bind_projection_failure(projection_failure),
        ))
    }

    pub(in crate::physical_runtime) fn file_length(
        &self,
        artifact: worth_store_physical_format::RecordArtifactFile,
        partition: RecordReadPartition,
    ) -> Result<
        (
            u64,
            PhysicalWorkIdentity,
            crate::physical_runtime::instance::PhysicalProjectionFailureCapability,
        ),
        CanonicalRecordReadFailureEvidence,
    > {
        self.prepare_metadata(artifact, partition)?.execute()
    }

    fn prepare_metadata(
        &self,
        artifact: worth_store_physical_format::RecordArtifactFile,
        partition: RecordReadPartition,
    ) -> Result<PreparedCanonicalMetadataRead, CanonicalRecordReadFailureEvidence> {
        let runtime = self.runtime.upgrade().ok_or_else(|| {
            CanonicalRecordReadFailureEvidence::before_work(
                CanonicalRecordReadFailure::RuntimeReleased,
            )
        })?;
        let request = PhysicalMetadataReadWorkRequest::new(
            artifact,
            self.record.read_basis(partition),
            self.record.security(),
        )
        .expect("record artifact metadata, read basis, and security are admitted together");
        let receipt = match self.submission.submit_metadata(request).into_raw() {
            TransitionOutcome::Success(receipt) => receipt,
            _ => {
                return Err(CanonicalRecordReadFailureEvidence::before_work(
                    CanonicalRecordReadFailure::SubmissionRejected,
                ))
            }
        };
        let (ready, identity) = admit_ready(&runtime, receipt, &self.physical)?;
        let prepared = self.prepare_metadata_command(&runtime, ready, identity)?;
        let (command, projection_failure) = require_projection_failure(prepared, identity)?;
        Ok(PreparedCanonicalMetadataRead::new(
            self.execution.clone(),
            command,
            identity,
            self.execution.bind_projection_failure(projection_failure),
        ))
    }
}
