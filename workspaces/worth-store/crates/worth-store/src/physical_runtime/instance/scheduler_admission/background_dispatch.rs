use super::PhysicalSchedulerAdmissionOwner;
use worth_store_io_scheduler::IoSchedulerBackendCapabilityAdmission;

impl PhysicalSchedulerAdmissionOwner {
    pub(in crate::physical_runtime) fn reserve_background_dispatch(
        &self,
        lane: worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration,
        security: &worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    ) -> Result<
        (
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation,
            IoSchedulerBackendCapabilityAdmission,
        ),
        super::RecordSchedulerReservationDenial,
    > {
        let reservation = self
            .foreground
            .reserve_background(lane, &self.buffered_file, security)
            .map_err(super::RecordSchedulerReservationDenial::Admission)?;
        Ok((reservation, self.buffered_file))
    }
}
