use super::*;

impl PhysicalSchedulerDemand {
    /// Selected C.5 record reads retain their protected routing while each
    /// canonical frame/metadata effect consumes a diagnostic-background turn.
    pub(in crate::physical_runtime) fn scrub_selected_record_background(
        ready: ReadyPhysicalWork,
        lease: BackgroundIdleCapacityLease,
        capacity: PhysicalInstanceForegroundCapacityLease,
    ) -> Result<Self, PhysicalSchedulerDenial> {
        ready
            .require_consumer_active()
            .map_err(PhysicalSchedulerDenial::PreEffect)?;
        let intent = ready.intent();
        let eligible = match intent.operation() {
            PhysicalWorkOperationFamily::ArtifactRangeRead => {
                intent.scope().coordinates().len() == 1
            }
            PhysicalWorkOperationFamily::ArtifactMetadataRead => {
                intent.scope().artifact_target().is_some()
            }
            _ => false,
        };
        if !eligible
            || lease.class() != worth_store_io_scheduler::BackgroundIoPressureClass::ScrubScan
        {
            return Err(PhysicalSchedulerDenial::BackgroundOperationMismatch(
                intent.operation(),
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
