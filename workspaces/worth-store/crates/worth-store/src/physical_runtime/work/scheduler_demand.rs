use worth_store_io_scheduler::foreground_reservation::{
    ForegroundIoLaneKind, ForegroundReservationReceipt, PhysicalInstanceForegroundCapacityLease,
    PhysicalInstanceForegroundReservation,
};
use worth_store_io_scheduler::{
    admit_queue_execution_plan, admit_queue_policy_receipt, lower_background_queue_lease,
    lower_physical_foreground_work, BackgroundIdleCapacityLease,
    IoSchedulerBackendCapabilityAdmission, PhysicalForegroundWorkDeclaration,
    QueueExecutionAdmissionDenial, QueueExecutionAdmissionRequest, QueueLocalityIdentity,
    QueueWorkDeclaration, SecureIoPreservationReceipt,
};
use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;

use super::{
    PhysicalWorkDurabilityRequirement, PhysicalWorkOperationFamily, ReadyPhysicalWork,
    ResourceAdmittedPhysicalWork,
};

mod locality;
mod residency;
mod scrub;

use locality::physical_locality;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalSchedulerDenial {
    PreEffect(super::PhysicalWorkPreEffectDenial),
    ForegroundLaneMismatch {
        operation: PhysicalWorkOperationFamily,
        lane: ForegroundIoLaneKind,
    },
    BackgroundOperationMismatch(PhysicalWorkOperationFamily),
    Queue(QueueExecutionAdmissionDenial),
    ResidencyWorkMismatch,
    Residency(worth_store_buffer_pool::PhysicalResidencyDenial),
}

pub struct PhysicalSchedulerDemand {
    ready: ReadyPhysicalWork,
    work: QueueWorkDeclaration,
    capacity: Option<PhysicalInstanceForegroundCapacityLease>,
}

pub struct PhysicalWorkScheduler;

impl PhysicalSchedulerDemand {
    pub fn foreground(
        ready: ReadyPhysicalWork,
        reservation: PhysicalInstanceForegroundReservation,
        secure_io: Option<SecureIoPreservationReceipt>,
    ) -> Result<Self, PhysicalSchedulerDenial> {
        ready
            .require_consumer_active()
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        let intent = ready.intent();
        require_lane(intent.operation(), reservation.receipt().lane())?;
        ready
            .admit_scheduler_pressure(pressure_class(reservation.receipt().lane()))
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        let (receipt, capacity) = reservation.into_parts();
        let declaration = physical_foreground_declaration(
            intent,
            receipt,
            physical_locality(intent.identity().store(), intent.scope()),
        );
        let declaration = match secure_io {
            Some(secure_io) => declaration.with_secure_io_scope(secure_io),
            None => declaration,
        };
        let work =
            lower_physical_foreground_work(declaration).map_err(PhysicalSchedulerDenial::Queue)?;
        Ok(Self {
            ready,
            work,
            capacity: Some(capacity),
        })
    }

    pub(in crate::physical_runtime) fn checkpoint_background(
        ready: ReadyPhysicalWork,
        lease: BackgroundIdleCapacityLease,
    ) -> Result<Self, PhysicalSchedulerDenial> {
        ready
            .require_consumer_active()
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        let operation = ready.intent().operation();
        if operation != PhysicalWorkOperationFamily::CheckpointCapture {
            return Err(PhysicalSchedulerDenial::BackgroundOperationMismatch(
                operation,
            ));
        }
        ready
            .admit_scheduler_pressure(super::PhysicalWorkPressureClass::BackgroundCheckpoint)
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        Ok(Self {
            ready,
            work: lower_background_queue_lease(lease),
            capacity: None,
        })
    }

    pub(in crate::physical_runtime) fn wal_reclamation_background(
        ready: ReadyPhysicalWork,
        lease: BackgroundIdleCapacityLease,
    ) -> Result<Self, PhysicalSchedulerDenial> {
        ready
            .require_consumer_active()
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        let operation = ready.intent().operation();
        if operation != PhysicalWorkOperationFamily::WalReclamation {
            return Err(PhysicalSchedulerDenial::BackgroundOperationMismatch(
                operation,
            ));
        }
        ready
            .admit_scheduler_pressure(super::PhysicalWorkPressureClass::BackgroundCheckpoint)
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        Ok(Self {
            ready,
            work: lower_background_queue_lease(lease),
            capacity: None,
        })
    }

    pub const fn intent(&self) -> &super::PhysicalWorkIntent {
        self.ready.intent()
    }

    pub const fn queue_work(&self) -> &QueueWorkDeclaration {
        &self.work
    }

    pub fn with_secure_io(mut self, secure_io: SecureIoPreservationReceipt) -> Self {
        self.work = self.work.with_secure_io_scope(secure_io);
        self
    }
}

const fn pressure_class(lane: ForegroundIoLaneKind) -> super::PhysicalWorkPressureClass {
    match lane {
        ForegroundIoLaneKind::PointRead => super::PhysicalWorkPressureClass::ForegroundPointRead,
        ForegroundIoLaneKind::RangeRead => super::PhysicalWorkPressureClass::ForegroundRangeRead,
        ForegroundIoLaneKind::InteractiveRead => {
            super::PhysicalWorkPressureClass::ForegroundInteractiveRead
        }
        ForegroundIoLaneKind::InternalForegroundRead => {
            super::PhysicalWorkPressureClass::ForegroundInternalRead
        }
        ForegroundIoLaneKind::ArtifactMetadataRead => {
            super::PhysicalWorkPressureClass::ForegroundInternalRead
        }
        ForegroundIoLaneKind::OrdinaryPageWrite => {
            super::PhysicalWorkPressureClass::ForegroundMutation
        }
        ForegroundIoLaneKind::CommitCriticalWalAppend
        | ForegroundIoLaneKind::CommitCriticalWalWrite
        | ForegroundIoLaneKind::RootPublication => {
            super::PhysicalWorkPressureClass::ForegroundMutation
        }
    }
}

fn require_lane(
    operation: PhysicalWorkOperationFamily,
    lane: ForegroundIoLaneKind,
) -> Result<(), PhysicalSchedulerDenial> {
    let compatible = match operation {
        PhysicalWorkOperationFamily::ArtifactMetadataRead => {
            lane == ForegroundIoLaneKind::ArtifactMetadataRead
        }
        PhysicalWorkOperationFamily::ArtifactRangeRead => matches!(
            lane,
            ForegroundIoLaneKind::PointRead
                | ForegroundIoLaneKind::RangeRead
                | ForegroundIoLaneKind::InteractiveRead
                | ForegroundIoLaneKind::InternalForegroundRead
        ),
        PhysicalWorkOperationFamily::ArtifactRangeWrite
        | PhysicalWorkOperationFamily::ArtifactPublication => {
            lane == ForegroundIoLaneKind::OrdinaryPageWrite
        }
        PhysicalWorkOperationFamily::WalAppend => {
            lane == ForegroundIoLaneKind::CommitCriticalWalAppend
        }
        PhysicalWorkOperationFamily::DurabilityBarrier => {
            lane == ForegroundIoLaneKind::CommitCriticalWalWrite
        }
        PhysicalWorkOperationFamily::RootPublication => {
            lane == ForegroundIoLaneKind::RootPublication
        }
        PhysicalWorkOperationFamily::CheckpointCapture
        | PhysicalWorkOperationFamily::WalReclamation => false,
    };
    compatible
        .then_some(())
        .ok_or(PhysicalSchedulerDenial::ForegroundLaneMismatch { operation, lane })
}

impl PhysicalWorkScheduler {
    pub fn admit(
        demand: PhysicalSchedulerDemand,
        backend: &IoSchedulerBackendCapabilityAdmission,
        policy: worth_foundational::FoundationalPolicyAdmissionReceipt,
    ) -> Result<ResourceAdmittedPhysicalWork, PhysicalSchedulerDenial> {
        let PhysicalSchedulerDemand {
            ready,
            work,
            capacity,
        } = demand;
        let policy =
            admit_queue_policy_receipt(work, policy).map_err(PhysicalSchedulerDenial::Queue)?;
        let plan = admit_queue_execution_plan(QueueExecutionAdmissionRequest::new(policy, backend))
            .map_err(PhysicalSchedulerDenial::Queue)?;
        Ok(ResourceAdmittedPhysicalWork::new(ready, plan, capacity))
    }
}

fn physical_foreground_declaration(
    intent: &super::PhysicalWorkIntent,
    reservation: ForegroundReservationReceipt,
    locality: QueueLocalityIdentity,
) -> PhysicalForegroundWorkDeclaration {
    let resources = intent.resources();
    match (intent.operation(), intent.durability()) {
        (PhysicalWorkOperationFamily::WalAppend, PhysicalWorkDurabilityRequirement::WalAppend) => {
            PhysicalForegroundWorkDeclaration::wal_append(
                reservation,
                locality,
                resources.queue_shape(),
                resources.flush_epoch(),
            )
        }
        (
            PhysicalWorkOperationFamily::DurabilityBarrier,
            PhysicalWorkDurabilityRequirement::WalDurabilityBarrier,
        ) => PhysicalForegroundWorkDeclaration::durable_write(
            reservation,
            locality,
            resources.queue_shape(),
            resources.flush_epoch(),
        ),
        (
            PhysicalWorkOperationFamily::RootPublication,
            PhysicalWorkDurabilityRequirement::RootPublication,
        ) => PhysicalForegroundWorkDeclaration::durable_write(
            reservation,
            locality,
            resources.queue_shape(),
            resources.flush_epoch(),
        ),
        (
            PhysicalWorkOperationFamily::ArtifactMetadataRead
            | PhysicalWorkOperationFamily::ArtifactRangeRead,
            PhysicalWorkDurabilityRequirement::ReadOnly,
        ) => PhysicalForegroundWorkDeclaration::read(
            reservation,
            locality,
            resources.queue_shape(),
            resources.flush_epoch(),
        ),
        (
            PhysicalWorkOperationFamily::ArtifactRangeWrite,
            PhysicalWorkDurabilityRequirement::ArtifactRangeWrite(
                ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
            ),
        ) => PhysicalForegroundWorkDeclaration::buffered_write(
            reservation,
            locality,
            resources.queue_shape(),
            resources.flush_epoch(),
        ),
        (
            PhysicalWorkOperationFamily::ArtifactRangeWrite,
            PhysicalWorkDurabilityRequirement::ArtifactRangeWrite(
                ArtifactRangeWriteDurabilityRequirement::FileDataSynchronization,
            ),
        )
        | (
            PhysicalWorkOperationFamily::ArtifactPublication,
            PhysicalWorkDurabilityRequirement::ArtifactRangeWrite(_),
        ) => PhysicalForegroundWorkDeclaration::durable_write(
            reservation,
            locality,
            resources.queue_shape(),
            resources.flush_epoch(),
        ),
        _ => unreachable!("physical declaration admission already proved operation durability"),
    }
}
