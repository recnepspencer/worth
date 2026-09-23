use worth_store_physical_format::RecordFrameCoordinate;

use crate::physical_runtime::{
    instance::PhysicalStoreWorkRuntime, work::PhysicalWorkAdmissionAuthority,
    PhysicalExecutorCommand, PhysicalExecutorCommandDenial, PhysicalSchedulerDemand,
    PhysicalWorkAdmission, PhysicalWorkIdentity, PhysicalWorkReadiness,
    PhysicalWorkSubmissionReceipt, ReadyPhysicalWork, ResourceAdmittedPhysicalWork,
};
#[cfg(feature = "certification-test-authority")]
use worth_store_io_scheduler::{
    IoSchedulerBackendCapabilityAdmission, SecureIoOperation, SecureIoPreservationDenial,
    SecureIoPreservationReceipt, SecureIoPreservationRequest,
};

use super::{
    CanonicalRecordReadFailure, CanonicalRecordReadFailureEvidence, CanonicalRecordReadPort,
    PreparedPhysicalCommand,
};

pub(super) enum RangeSchedulerRoute<'route, 'grant> {
    Ordinary {
        lifetimes: std::marker::PhantomData<(&'route (), &'grant ())>,
    },
    #[cfg(feature = "certification-test-authority")]
    Prefetch(&'route worth_store_buffer_pool::PrefetchResidencyGrant),
    #[cfg(feature = "certification-test-authority")]
    ReadAhead(&'route worth_store_buffer_pool::ReadAheadFrameGrant<'grant, 'route>),
}

impl<'route, 'grant> RangeSchedulerRoute<'route, 'grant> {
    pub(super) const fn ordinary() -> Self {
        Self::Ordinary {
            lifetimes: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) const fn prefetch(
        grant: &'route worth_store_buffer_pool::PrefetchResidencyGrant,
    ) -> Self {
        Self::Prefetch(grant)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) const fn read_ahead(
        grant: &'route worth_store_buffer_pool::ReadAheadFrameGrant<'grant, 'route>,
    ) -> Self {
        Self::ReadAhead(grant)
    }
}

impl CanonicalRecordReadPort {
    pub(super) fn prepare_range_command<'route, 'grant>(
        &self,
        runtime: &PhysicalStoreWorkRuntime,
        ready: ReadyPhysicalWork,
        identity: PhysicalWorkIdentity,
        coordinate: RecordFrameCoordinate,
        route: RangeSchedulerRoute<'route, 'grant>,
    ) -> Result<PreparedPhysicalCommand, CanonicalRecordReadFailureEvidence> {
        let (reservation, backend) = self
            .scheduler
            .record_read(
                self.record.scheduler_security(),
                u64::from(coordinate.length()),
            )
            .map_err(CanonicalRecordReadFailure::SchedulerReservation)
            .map_err(|failure| {
                CanonicalRecordReadFailureEvidence::during_work(failure, identity)
            })?;
        let demand = match route {
            RangeSchedulerRoute::Ordinary { .. } => {
                PhysicalSchedulerDemand::foreground(ready, reservation, None)
            }
            #[cfg(feature = "certification-test-authority")]
            RangeSchedulerRoute::Prefetch(grant) => {
                let secure_io = self
                    .admit_speculative_read_secure_io(&backend)
                    .map_err(CanonicalRecordReadFailure::SecureIo)
                    .map_err(|failure| {
                        CanonicalRecordReadFailureEvidence::during_work(failure, identity)
                    })?;
                PhysicalSchedulerDemand::residency_prefetch(ready, grant, reservation, secure_io)
            }
            #[cfg(feature = "certification-test-authority")]
            RangeSchedulerRoute::ReadAhead(grant) => {
                let secure_io = self
                    .admit_speculative_read_secure_io(&backend)
                    .map_err(CanonicalRecordReadFailure::SecureIo)
                    .map_err(|failure| {
                        CanonicalRecordReadFailureEvidence::during_work(failure, identity)
                    })?;
                PhysicalSchedulerDemand::residency_read_ahead(ready, grant, reservation, secure_io)
            }
        }
        .map_err(CanonicalRecordReadFailure::Scheduler)
        .map_err(|failure| CanonicalRecordReadFailureEvidence::during_work(failure, identity))?;
        prepare_command(
            runtime,
            self.scheduler.effects(),
            demand,
            identity,
            backend,
            |work| PhysicalExecutorCommand::read(work),
        )
    }

    #[cfg(feature = "certification-test-authority")]
    fn admit_speculative_read_secure_io(
        &self,
        backend: &IoSchedulerBackendCapabilityAdmission,
    ) -> Result<SecureIoPreservationReceipt, SecureIoPreservationDenial> {
        worth_store_io_scheduler::admit_secure_io_scope_for_scheduler(
            SecureIoPreservationRequest::new(
                SecureIoOperation::ReadAhead,
                self.record.scheduler_security(),
                backend,
            ),
        )
    }

    pub(super) fn prepare_metadata_command(
        &self,
        runtime: &PhysicalStoreWorkRuntime,
        ready: ReadyPhysicalWork,
        identity: PhysicalWorkIdentity,
    ) -> Result<PreparedPhysicalCommand, CanonicalRecordReadFailureEvidence> {
        let (reservation, backend) = self
            .scheduler
            .record_metadata(self.record.scheduler_security())
            .map_err(CanonicalRecordReadFailure::SchedulerReservation)
            .map_err(|failure| {
                CanonicalRecordReadFailureEvidence::during_work(failure, identity)
            })?;
        let demand = PhysicalSchedulerDemand::foreground(ready, reservation, None)
            .map_err(CanonicalRecordReadFailure::Scheduler)
            .map_err(|failure| {
                CanonicalRecordReadFailureEvidence::during_work(failure, identity)
            })?;
        prepare_command(
            runtime,
            self.scheduler.effects(),
            demand,
            identity,
            backend,
            |work| PhysicalExecutorCommand::metadata(work),
        )
    }
}

pub(super) fn require_projection_failure(
    prepared: PreparedPhysicalCommand,
    identity: PhysicalWorkIdentity,
) -> Result<
    (
        PhysicalExecutorCommand,
        crate::physical_runtime::PhysicalWorkAspectDelta,
    ),
    CanonicalRecordReadFailureEvidence,
> {
    let projection_failure = prepared.projection_failure.ok_or_else(|| {
        CanonicalRecordReadFailureEvidence::during_work(
            CanonicalRecordReadFailure::ProjectionFailureUnavailable,
            identity,
        )
    })?;
    Ok((prepared.command, projection_failure))
}

pub(super) fn admit_ready(
    runtime: &PhysicalStoreWorkRuntime,
    receipt: PhysicalWorkSubmissionReceipt,
    physical: &PhysicalWorkAdmissionAuthority,
) -> Result<(ReadyPhysicalWork, PhysicalWorkIdentity), CanonicalRecordReadFailureEvidence> {
    let identity = receipt.identity();
    let admitted =
        PhysicalWorkAdmission::admit(&runtime.submission, receipt, physical, &runtime.health)
            .map_err(CanonicalRecordReadFailure::PreEffect)
            .map_err(|failure| {
                CanonicalRecordReadFailureEvidence::during_work(failure, identity)
            })?;
    match runtime
        .signal
        .request(admitted)
        .map_err(CanonicalRecordReadFailure::PreEffect)
        .map_err(|failure| CanonicalRecordReadFailureEvidence::during_work(failure, identity))?
    {
        PhysicalWorkReadiness::Ready(ready) => Ok((ready, identity)),
        PhysicalWorkReadiness::Blocked(_) => Err(CanonicalRecordReadFailureEvidence::during_work(
            CanonicalRecordReadFailure::DependencyBlocked,
            identity,
        )),
    }
}

fn prepare_command(
    runtime: &PhysicalStoreWorkRuntime,
    effects: &crate::physical_runtime::work::PhysicalEffectAdmission,
    demand: PhysicalSchedulerDemand,
    identity: PhysicalWorkIdentity,
    backend: worth_store_io_scheduler::IoSchedulerBackendCapabilityAdmission,
    build: impl FnOnce(
        ResourceAdmittedPhysicalWork,
    ) -> Result<PhysicalExecutorCommand, PhysicalExecutorCommandDenial>,
) -> Result<PreparedPhysicalCommand, CanonicalRecordReadFailureEvidence> {
    PhysicalWorkAdmission::require_current(&runtime.submission, demand.intent(), &runtime.health)
        .map_err(CanonicalRecordReadFailure::PreEffect)
        .map_err(|failure| CanonicalRecordReadFailureEvidence::during_work(failure, identity))?;
    let policy = super::super::record_queue_policy::admit_record_queue_policy(demand.queue_work());
    let work =
        crate::physical_runtime::PhysicalWorkScheduler::admit(effects, demand, &backend, policy)
            .map_err(CanonicalRecordReadFailure::Scheduler)
            .map_err(|failure| {
                CanonicalRecordReadFailureEvidence::during_work(failure, identity)
            })?;
    debug_assert_eq!(work.intent().identity(), identity);
    let projection_failure = runtime
        .signal
        .admit_projection_failure(&work)
        .map_err(|_| CanonicalRecordReadFailure::ProjectionFailureUnavailable)
        .map_err(|failure| CanonicalRecordReadFailureEvidence::during_work(failure, identity))?;
    let command = build(work)
        .map_err(CanonicalRecordReadFailure::Command)
        .map_err(|failure| CanonicalRecordReadFailureEvidence::during_work(failure, identity))?;
    Ok(PreparedPhysicalCommand {
        command,
        projection_failure,
    })
}
