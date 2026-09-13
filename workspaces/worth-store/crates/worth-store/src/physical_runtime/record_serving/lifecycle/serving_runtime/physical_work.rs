use super::ServingPhysicalRuntime;

mod batch_execution;
#[cfg(feature = "certification-test-authority")]
mod certification_cross_settlement;
mod execution;
mod settlement_revocation;

impl ServingPhysicalRuntime {
    fn physical_work_lifecycle(&self) -> crate::physical_runtime::instance::PhysicalWorkLifecycle {
        crate::physical_runtime::instance::PhysicalWorkLifecycle::new(
            &self.parts.work_runtime,
            self.parts.work_admission,
        )
    }

    pub fn physical_signal_runtime_identity(
        &self,
    ) -> crate::physical_runtime::PhysicalSignalRuntimeIdentity {
        self.parts.work_runtime.signal.runtime_identity()
    }

    pub fn physical_signal_profile_identity(
        &self,
    ) -> crate::physical_runtime::PhysicalSignalProfileIdentity {
        self.parts.work_runtime.signal.profile()
    }

    pub fn physical_signal_aspect_binding_observations(
        &self,
    ) -> Box<[crate::physical_runtime::PhysicalSignalAspectBindingObservation]> {
        self.parts.work_runtime.signal.binding_observations()
    }

    pub fn physical_signal_clock_observation(
        &self,
    ) -> Result<
        crate::physical_runtime::PhysicalSignalClockObservation,
        crate::physical_runtime::PhysicalSignalClockObservationFailure,
    > {
        self.parts.work_runtime.signal.clock_observation()
    }

    pub fn physical_signal_observation(
        &self,
    ) -> Result<
        crate::physical_runtime::PhysicalSignalObservation,
        crate::physical_runtime::PhysicalSignalClockObservationFailure,
    > {
        self.parts.work_runtime.signal.observation()
    }

    pub fn apply_physical_aspect_delta(
        &self,
        delta: crate::physical_runtime::PhysicalWorkAspectDelta,
    ) -> Result<(), crate::physical_runtime::PhysicalSignalDeltaApplicationFailure> {
        self.parts.work_runtime.signal.apply_delta(delta)
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_apply_physical_aspect_delta(
        &self,
        delta: crate::physical_runtime::PhysicalWorkAspectDelta,
    ) -> Result<(), crate::physical_runtime::PhysicalSignalDeltaApplicationFailure> {
        self.parts
            .work_runtime
            .signal
            .apply_delta_for_certification(delta)
    }

    pub fn physical_read_submission(&self) -> crate::physical_runtime::PhysicalReadSubmission {
        self.physical_work_lifecycle().read_submission()
    }

    pub fn physical_mutation_submission(
        &self,
    ) -> crate::physical_runtime::PhysicalMutationSubmission {
        self.physical_work_lifecycle().mutation_submission()
    }

    pub fn physical_work_observer(&self) -> crate::physical_runtime::PhysicalWorkObservation {
        self.physical_work_lifecycle().observation()
    }

    pub fn physical_work_counters(&self) -> crate::physical_runtime::PhysicalWorkCounterSnapshot {
        self.parts.work_runtime.submission.counters()
    }

    pub fn admit_physical_work(
        &self,
        receipt: crate::physical_runtime::PhysicalWorkSubmissionReceipt,
    ) -> Result<
        crate::physical_runtime::AdmittedPhysicalWork,
        crate::physical_runtime::PhysicalWorkPreEffectDenial,
    > {
        self.physical_work_lifecycle().admit(receipt)
    }

    pub fn request_physical_work(
        &self,
        admitted: crate::physical_runtime::AdmittedPhysicalWork,
    ) -> Result<
        crate::physical_runtime::PhysicalWorkReadiness,
        crate::physical_runtime::PhysicalWorkPreEffectDenial,
    > {
        self.physical_work_lifecycle().request(admitted)
    }

    pub fn revalidate_physical_work(
        &self,
        ready: crate::physical_runtime::ReadyPhysicalWork,
    ) -> Result<
        crate::physical_runtime::PhysicalWorkReadiness,
        crate::physical_runtime::PhysicalWorkPreEffectDenial,
    > {
        self.physical_work_lifecycle().revalidate_ready(ready)
    }

    pub fn revalidate_blocked_physical_work(
        &self,
        blocked: crate::physical_runtime::BlockedPhysicalWork,
    ) -> Result<
        crate::physical_runtime::PhysicalWorkReadiness,
        crate::physical_runtime::PhysicalWorkPreEffectDenial,
    > {
        self.physical_work_lifecycle().revalidate_blocked(blocked)
    }

    pub fn admit_physical_scheduler_capability(
        &self,
        requirement: worth_store_io_scheduler::IoSchedulerBackendCapabilityRequirement,
    ) -> Result<
        worth_store_io_scheduler::IoSchedulerBackendCapabilityAdmission,
        worth_store_io_scheduler::IoSchedulerBackendCapabilityDenial,
    > {
        self.parts.scheduler_admission.admit(
            self.parts.work_runtime.executor.record_serving_media(),
            requirement,
        )
    }

    pub fn reserve_physical_scheduler_foreground(
        &self,
        lane: worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration,
    ) -> Result<
        (
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation,
            worth_store_io_scheduler::IoSchedulerBackendCapabilityAdmission,
        ),
        worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial,
    > {
        self.parts
            .scheduler_admission
            .reserve_record_lane(lane, self.parts.record_work.scheduler_security())
    }

    pub fn physical_scheduler_capacity(
        &self,
    ) -> worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundCapacitySnapshot
    {
        self.parts.scheduler_admission.capacity_snapshot()
    }

    pub fn admit_physical_scheduler_demand(
        &self,
        demand: crate::physical_runtime::PhysicalSchedulerDemand,
        backend: &worth_store_io_scheduler::IoSchedulerBackendCapabilityAdmission,
        policy: worth_foundational::FoundationalPolicyAdmissionReceipt,
    ) -> Result<
        crate::physical_runtime::ResourceAdmittedPhysicalWork,
        crate::physical_runtime::PhysicalSchedulerDenial,
    > {
        crate::physical_runtime::PhysicalWorkAdmission::require_current(
            &self.parts.work_runtime.submission,
            demand.intent(),
            &self.parts.work_runtime.health,
        )
        .map_err(crate::physical_runtime::PhysicalSchedulerDenial::PreEffect)?;
        crate::physical_runtime::PhysicalWorkScheduler::admit(demand, backend, policy)
    }

    pub fn execute_physical_work(
        &self,
        command: crate::physical_runtime::PhysicalExecutorCommand,
    ) -> Result<
        crate::physical_runtime::PhysicalWorkExecutionOutcome,
        crate::physical_runtime::PhysicalWorkPreEffectDenial,
    > {
        self.physical_work_execution()
            .execute_physical_work(command)
    }

    pub fn physical_work_execution(&self) -> crate::physical_runtime::PhysicalWorkExecution {
        crate::physical_runtime::instance::PhysicalStoreWorkRuntime::execution(
            &self.parts.work_runtime,
            self.parts.core.lifecycle_generation(),
        )
    }

    pub fn physical_recovery_obligations(
        &self,
    ) -> &[crate::physical_runtime::PhysicalWorkRecoveryLocator] {
        self.parts.work_runtime.recovery.obligations()
    }

    pub fn physical_recovery_evidence_damaged(&self) -> bool {
        self.parts.work_runtime.recovery.evidence_damaged()
    }

    pub fn physical_recovery_admission_observations(
        &self,
    ) -> &[crate::physical_runtime::PhysicalWorkRecoveryAdmissionObservation] {
        self.parts.work_runtime.recovery.admission_observations()
    }

    pub fn physical_recovery_admission_counters(
        &self,
    ) -> crate::physical_runtime::PhysicalWorkRecoveryAdmissionCounters {
        self.parts.work_runtime.recovery.admission_counters()
    }

    pub fn cancel_physical_work(
        &self,
        consumer: crate::physical_runtime::PhysicalWorkConsumerHandle,
    ) -> Result<
        crate::physical_runtime::PhysicalWorkCancellationJoin,
        crate::physical_runtime::PhysicalWorkCancellationFailure,
    > {
        self.physical_work_lifecycle().cancel(consumer)
    }

    pub fn schedule_physical_work_retry(
        &self,
        settled: &crate::physical_runtime::SettledPhysicalWork,
    ) -> Result<
        crate::physical_runtime::PhysicalWorkRetryScheduleOutcome,
        crate::physical_runtime::PhysicalWorkRetryFailure,
    > {
        self.physical_work_lifecycle().schedule_retry(settled)
    }

    pub fn advance_physical_signal_clock(
        &self,
        consumer: crate::physical_runtime::PhysicalWorkConsumerHandle,
        request: worth_signal::facade::ClockAdvanceRequest,
    ) -> Result<
        worth_signal::facade::ValidatedClockAdvance,
        crate::physical_runtime::PhysicalWorkRetryFailure,
    > {
        self.physical_work_lifecycle()
            .advance_clock(consumer, request)
    }

    pub fn admit_physical_work_retry(
        &self,
        retry: &crate::physical_runtime::PhysicalWorkRetrySchedule,
        settled: crate::physical_runtime::SettledPhysicalWork,
    ) -> Result<
        crate::physical_runtime::PhysicalWorkRetryAdmission,
        crate::physical_runtime::PhysicalWorkRetryFailure,
    > {
        self.physical_work_lifecycle().admit_retry(retry, settled)
    }

    pub fn timeout_physical_work(
        &self,
        consumer: crate::physical_runtime::PhysicalWorkConsumerHandle,
    ) -> Result<
        crate::physical_runtime::PhysicalWorkTimeoutJoin,
        crate::physical_runtime::PhysicalWorkCancellationFailure,
    > {
        self.physical_work_lifecycle().timeout(consumer)
    }
}
