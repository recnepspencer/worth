use worth_store_buffer_pool::{
    BufferPoolQueueGroupingScope, BufferPoolWritebackQueueExecutionDeclaration,
};
use worth_store_contracts::QueueProducerResourceShape;
use worth_store_io_scheduler::{
    foreground_reservation::PhysicalInstanceForegroundReservation,
    IoSchedulerBackendCapabilityAdmission, SecureIoOperation, SecureIoPreservationReceipt,
    SecureIoPreservationRequest,
};
use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;
use worth_store_physical_format::RecordFrameCoordinate;

use crate::physical_runtime::{
    instance::RecordSchedulerReservationDenial, PhysicalRetryCommand, PhysicalSchedulerDemand,
    PhysicalWorkAdmission, PhysicalWorkScheduler, ReadyPhysicalWork, ResourceAdmittedPhysicalWork,
};

use super::{
    super::failure::{PhysicalWritebackFailureCause, PhysicalWritebackTransitionFailure},
    AdmittedPhysicalWriteback, FrameWritebackPort, ReadyPhysicalWriteback,
};
use crate::physical_runtime::record_serving::record_queue_policy::admit_record_queue_policy;
use crate::physical_runtime::record_serving::residency::scheduled_writeback::PhysicalScheduledWritebackAdmissionDenial;

#[derive(Clone, Copy)]
struct WritebackSchedulerBasis {
    coordinate: RecordFrameCoordinate,
    durability: ArtifactRangeWriteDurabilityRequirement,
    flush_epoch: u64,
    resource_shape: QueueProducerResourceShape,
}

impl FrameWritebackPort {
    pub(in crate::physical_runtime::record_serving) fn admit(
        &self,
        ready: ReadyPhysicalWriteback,
        retry: Option<PhysicalRetryCommand>,
    ) -> Result<AdmittedPhysicalWriteback, PhysicalWritebackTransitionFailure> {
        let ReadyPhysicalWriteback {
            ready,
            claim,
            dirty,
            durability,
        } = ready;
        let identity = ready.intent().identity();
        let basis = WritebackSchedulerBasis::new(&ready, dirty.coordinate(), durability);
        let work = match self.admit_ready_work(ready, &claim, basis, retry) {
            Ok(work) => work,
            Err(cause) => return Err(PhysicalWritebackTransitionFailure::new(cause, dirty)),
        };
        debug_assert_eq!(work.intent().identity(), identity);
        Ok(AdmittedPhysicalWriteback { work, claim, dirty })
    }

    fn admit_ready_work(
        &self,
        ready: ReadyPhysicalWork,
        claim: &worth_store_buffer_pool::PhysicalWritebackClaim,
        basis: WritebackSchedulerBasis,
        retry: Option<PhysicalRetryCommand>,
    ) -> Result<ResourceAdmittedPhysicalWork, PhysicalWritebackFailureCause> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(PhysicalWritebackFailureCause::RuntimeReleased)?;
        PhysicalWorkAdmission::require_ready_current(&runtime.submission, &ready, &runtime.health)
            .map_err(PhysicalWritebackFailureCause::PreEffect)?;
        let (demand, backend) = self.lower_scheduler_demand(ready, claim, basis)?;
        let work = admit_scheduler_demand(self.scheduler.effects(), demand, &backend)?;
        admit_retry(retry, &work)?;
        Ok(work)
    }

    fn lower_scheduler_demand(
        &self,
        ready: ReadyPhysicalWork,
        claim: &worth_store_buffer_pool::PhysicalWritebackClaim,
        basis: WritebackSchedulerBasis,
    ) -> Result<
        (
            PhysicalSchedulerDemand,
            IoSchedulerBackendCapabilityAdmission,
        ),
        PhysicalWritebackFailureCause,
    > {
        let (reservation, backend) = self.reserve_scheduler(basis.coordinate, basis.durability)?;
        let declaration = self.writeback_declaration(claim, basis, &reservation)?;
        let secure_io = self.admit_secure_io(&backend)?;
        let demand = PhysicalSchedulerDemand::residency_writeback(
            ready,
            declaration,
            reservation,
            secure_io,
        )
        .map_err(PhysicalWritebackFailureCause::Scheduler)?;
        Ok((demand, backend))
    }

    fn writeback_declaration(
        &self,
        claim: &worth_store_buffer_pool::PhysicalWritebackClaim,
        basis: WritebackSchedulerBasis,
        reservation: &PhysicalInstanceForegroundReservation,
    ) -> Result<BufferPoolWritebackQueueExecutionDeclaration, PhysicalWritebackFailureCause> {
        let grouping =
            BufferPoolQueueGroupingScope::new(reservation.receipt().security_scope_identity());
        let context = worth_store_buffer_pool::BufferPoolQueueDeclarationContext::new(
            grouping,
            basis.flush_epoch,
            basis.resource_shape,
        );
        self.frame_ports
            .writeback_declaration(claim, context, basis.durability)
            .map_err(PhysicalWritebackFailureCause::Residency)
    }

    fn admit_secure_io(
        &self,
        backend: &IoSchedulerBackendCapabilityAdmission,
    ) -> Result<SecureIoPreservationReceipt, PhysicalWritebackFailureCause> {
        worth_store_io_scheduler::admit_secure_io_scope_for_scheduler(
            SecureIoPreservationRequest::new(
                SecureIoOperation::WriteBack,
                self.record.scheduler_security(),
                backend,
            ),
        )
        .map_err(PhysicalWritebackFailureCause::SecureIo)
    }

    fn reserve_scheduler(
        &self,
        coordinate: RecordFrameCoordinate,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> Result<
        (
            PhysicalInstanceForegroundReservation,
            IoSchedulerBackendCapabilityAdmission,
        ),
        PhysicalWritebackFailureCause,
    > {
        let synchronization = matches!(
            durability,
            ArtifactRangeWriteDurabilityRequirement::FileDataSynchronization
        );
        self.scheduler
            .record_write(
                self.record.scheduler_security(),
                u64::from(coordinate.length()),
                synchronization,
                false,
            )
            .map_err(|denial| match denial {
                RecordSchedulerReservationDenial::Admission(denial) => {
                    PhysicalWritebackFailureCause::SchedulerReservation(denial)
                }
                RecordSchedulerReservationDenial::OwedBackgroundTurn => {
                    PhysicalWritebackFailureCause::Scheduler(
                        crate::physical_runtime::PhysicalSchedulerDenial::OwedBackgroundTurn,
                    )
                }
            })
    }
}

impl WritebackSchedulerBasis {
    fn new(
        ready: &ReadyPhysicalWork,
        coordinate: RecordFrameCoordinate,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> Self {
        let resources = ready.intent().resources();
        Self {
            coordinate,
            durability,
            flush_epoch: resources.flush_epoch(),
            resource_shape: resources.queue_shape(),
        }
    }
}

fn admit_scheduler_demand(
    effects: &crate::physical_runtime::work::PhysicalEffectAdmission,
    demand: PhysicalSchedulerDemand,
    backend: &IoSchedulerBackendCapabilityAdmission,
) -> Result<ResourceAdmittedPhysicalWork, PhysicalWritebackFailureCause> {
    let policy = admit_record_queue_policy(demand.queue_work());
    PhysicalWorkScheduler::admit(effects, demand, backend, policy)
        .map_err(PhysicalWritebackFailureCause::Scheduler)
}

fn admit_retry(
    retry: Option<PhysicalRetryCommand>,
    work: &ResourceAdmittedPhysicalWork,
) -> Result<(), PhysicalWritebackFailureCause> {
    let Some(retry) = retry else {
        return Ok(());
    };
    retry
        .admit_residency_retry(work)
        .map_err(|denial| {
            PhysicalWritebackFailureCause::WritebackAdmission(
                PhysicalScheduledWritebackAdmissionDenial::Retry(denial),
            )
        })
        .map(|_| ())
}
