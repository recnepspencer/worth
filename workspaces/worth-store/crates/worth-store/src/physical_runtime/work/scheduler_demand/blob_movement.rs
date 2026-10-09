use super::*;

impl PhysicalSchedulerDemand {
    pub(in crate::physical_runtime) fn blob_movement_background(
        ready: ReadyPhysicalWork,
        lease: BackgroundIdleCapacityLease,
        capacity: PhysicalInstanceForegroundCapacityLease,
    ) -> Result<Self, PhysicalSchedulerDenial> {
        ready
            .require_consumer_active()
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        let operation = ready.intent().operation();
        if operation != PhysicalWorkOperationFamily::ArtifactPublication
            || lease.class()
                != worth_store_io_scheduler::BackgroundIoPressureClass::MigrationPressure
        {
            return Err(PhysicalSchedulerDenial::BackgroundOperationMismatch(
                operation,
            ));
        }
        ready
            .admit_scheduler_pressure(
                super::super::PhysicalWorkPressureClass::BackgroundBlobMovement,
            )
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        Ok(Self {
            ready,
            work: lower_background_queue_lease(lease),
            capacity: Some(capacity),
        })
    }
}
