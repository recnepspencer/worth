use super::*;

impl PhysicalSchedulerDemand {
    pub(in crate::physical_runtime) fn scrub_background(
        ready: ReadyPhysicalWork,
        lease: BackgroundIdleCapacityLease,
        capacity: PhysicalInstanceForegroundCapacityLease,
    ) -> Result<Self, PhysicalSchedulerDenial> {
        ready
            .require_consumer_active()
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        let operation = ready.intent().operation();
        if operation != PhysicalWorkOperationFamily::ArtifactRangeRead
            || ready.intent().scope().inspection_target().is_none()
            || lease.class() != worth_store_io_scheduler::BackgroundIoPressureClass::ScrubScan
        {
            return Err(PhysicalSchedulerDenial::BackgroundOperationMismatch(
                operation,
            ));
        }
        ready
            .admit_scheduler_pressure(super::super::PhysicalWorkPressureClass::BackgroundScrub)
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        Ok(Self {
            ready,
            work: lower_background_queue_lease(lease),
            capacity: Some(capacity),
        })
    }
}
