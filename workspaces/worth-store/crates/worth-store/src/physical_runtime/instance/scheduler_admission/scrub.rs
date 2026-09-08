use super::PhysicalSchedulerAdmissionOwner;
use worth_store_io_scheduler::foreground_reservation::*;
use worth_store_io_scheduler::*;

#[derive(Debug, Clone, Copy)]
pub(in crate::physical_runtime) enum PhysicalScrubSchedulerAdmissionDenial {
    Capacity(PhysicalInstanceForegroundAdmissionDenial),
    Pacing(BackgroundPacingDenial),
    Deferred,
}

impl PhysicalSchedulerAdmissionOwner {
    pub(in crate::physical_runtime) fn scrub_background(
        &self,
        security: &IoSchedulerSecurityScopeAdmission,
        bytes: u64,
    ) -> Result<
        (
            BackgroundIdleCapacityLease,
            PhysicalInstanceForegroundCapacityLease,
            IoSchedulerBackendCapabilityAdmission,
            worth_foundational::FoundationalPolicyAdmissionReceipt,
        ),
        PhysicalScrubSchedulerAdmissionDenial,
    > {
        let lane = ForegroundLaneDeclaration::buffered_file_internal_foreground_read()
            .expect("Store-owned buffered inspection")
            .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
                "physical-integrity-scrub",
                1,
            ))
            .with_budget(super::read_budget(bytes));
        let reservation = self
            .foreground
            .reserve_background(lane, &self.buffered_file, security)
            .map_err(PhysicalScrubSchedulerAdmissionDenial::Capacity)?;
        let (receipt, capacity) = reservation.into_parts();
        let budget = BackgroundResourceBudget::new()
            .with_queue_slots(QueueSlot::new(1).expect("one window"))
            .with_worker_permits(WorkerPermit::new(1).expect("one window"))
            .with_bandwidth(BandwidthToken::bytes(bytes).expect("nonempty target"));
        let policy = crate::physical_runtime::record_serving::admit_scrub_background_policy(budget);
        let admission = admit_background_capacity(
            BackgroundCapacityAdmissionRequest::new(
                BackgroundIoPressureShape::buffered_file_scrub_scan().requesting(budget),
                &receipt,
                &self.buffered_file,
                policy.clone(),
            )
            .with_idle_available(budget)
            .with_policy_admitted(budget),
        )
        .map_err(PhysicalScrubSchedulerAdmissionDenial::Pacing)?;
        match admit_background_pacing(BackgroundIdleCapacityLeaseRequest::new(admission)) {
            BackgroundPacingOutcome::AdmittedWithDebt(admitted) => {
                Ok((admitted.into_lease(), capacity, self.buffered_file, policy))
            }
            _ => Err(PhysicalScrubSchedulerAdmissionDenial::Deferred),
        }
    }
}
