use super::ServingPhysicalRuntime;

mod durable_publication;
mod root_completion;
mod wal_durable;

impl ServingPhysicalRuntime {
    pub fn certification_pause_next_read_root_capture(
        &self,
        stage: crate::physical_runtime::certification::CertificationReadRootCaptureStage,
    ) -> crate::physical_runtime::certification::CertificationReadRootCapturePauseGate {
        self.parts.publication.pause_next_root_capture(stage)
    }

    pub fn certification_record_submission(
        &self,
    ) -> crate::physical_runtime::certification::CertificationPhysicalRecordSubmission {
        crate::physical_runtime::record_serving::RecordPublicationDirector::certification_submission(
            &self.parts.publication,
        )
    }

    pub fn certification_advance_physical_signal_clock(
        &self,
        request: worth_signal::facade::ClockAdvanceRequest,
    ) -> Result<
        worth_signal::facade::ValidatedClockAdvance,
        crate::physical_runtime::PhysicalWorkRetryFailure,
    > {
        self.parts
            .work_runtime
            .signal
            .advance_clock_for_certification(request)
            .map_err(|_| crate::physical_runtime::PhysicalWorkRetryFailure::DerivedStateUnavailable)
    }

    pub fn certification_pending_publication_count(&self) -> usize {
        self.parts.publication.pending_publication_count()
    }

    /// Limits usable candidate growth so rewrite admission can be denied one-over.
    ///
    /// `usable_growth_bytes` is the allowance after progress headroom. Headroom stays
    /// at the store default (64 KiB) so one-byte-over claims remain honest.
    pub fn certification_charged_growth_bytes(&self) -> u64 {
        self.parts.publication.charged_growth_bytes()
    }

    pub fn certification_limit_candidate_growth_bytes(&self, usable_growth_bytes: u64) {
        self.parts
            .publication
            .limit_candidate_growth_bytes(usable_growth_bytes);
    }

    pub fn certification_pause_physical_mutation_at(
        &self,
        checkpoint: crate::physical_runtime::certification::CertificationPhysicalMutationCheckpoint,
    ) -> crate::physical_runtime::certification::CertificationPhysicalMutationPauseGate {
        self.parts
            .publication
            .pause_mutation_at_for_certification(checkpoint)
    }

    pub fn certification_physical_residency(
        &self,
    ) -> crate::physical_runtime::record_serving::PhysicalResidencyCertification {
        crate::physical_runtime::record_serving::PhysicalResidencyCertification::from_parts(
            &self.parts,
        )
    }

    pub fn certification_stale_physical_residency(
        &self,
    ) -> crate::physical_runtime::record_serving::PhysicalResidencyCertification {
        crate::physical_runtime::record_serving::PhysicalResidencyCertification::stale_from_parts(
            &self.parts,
        )
    }

    pub fn certification_begin_lifecycle_termination(&self) {
        self.parts.termination.begin_for_certification();
    }

    pub fn certification_stale_physical_work_execution(
        &self,
    ) -> crate::physical_runtime::PhysicalWorkExecution {
        let generation = self
            .parts
            .core
            .lifecycle_generation()
            .certification_predecessor();
        crate::physical_runtime::instance::PhysicalStoreWorkRuntime::execution(
            &self.parts.work_runtime,
            generation,
        )
    }

    pub fn certification_cross_settle_physical_writes(
        &self,
        first: crate::physical_runtime::PhysicalExecutorCommand,
        second: crate::physical_runtime::PhysicalExecutorCommand,
    ) -> Result<
        [crate::physical_runtime::PhysicalWorkEffectFate; 2],
        crate::physical_runtime::PhysicalWorkPreEffectDenial,
    > {
        self.parts
            .work_runtime
            .certification_cross_settle_physical_writes(first, second)
    }

    pub fn certification_pause_physical_command_shards_after_lock(
        &self,
    ) -> crate::physical_runtime::certification::CertificationPhysicalSubmissionPauseGate {
        self.parts
            .work_runtime
            .submission
            .pause_after_command_shard_lock_for_certification()
    }

    pub fn certification_pause_physical_signal_after_dequeue(
        &self,
    ) -> crate::physical_runtime::certification::CertificationPhysicalSignalPauseGate {
        self.parts
            .work_runtime
            .signal
            .pause_after_dequeue_for_certification()
    }

    pub fn certification_pause_physical_execution_at(
        &self,
        checkpoint: crate::physical_runtime::certification::
            CertificationPhysicalExecutionCheckpoint,
    ) -> crate::physical_runtime::certification::CertificationPhysicalExecutionPauseGate {
        self.parts
            .work_runtime
            .executor
            .pause_at_for_certification(checkpoint)
    }

    pub fn certification_fail_next_physical_signal_abandonment(&self) {
        self.parts
            .work_runtime
            .signal
            .fail_next_abandonment_for_certification();
    }

    pub fn certification_physical_signal_route_depth(
        &self,
        route: crate::physical_runtime::PhysicalSignalAspectBindingDigest,
    ) -> Option<usize> {
        self.parts
            .work_runtime
            .signal
            .route_depth_for_certification(route)
    }

    pub fn certification_require_serving_inspection(&self) {
        self.parts.work_runtime.health.revoke();
    }

    /// Drive one checkpoint attempt through the production work port.
    /// Foreground pressure yields before a media effect and keeps the owed turn.
    pub fn certification_checkpoint_work_yields_for_foreground_pressure(&self) -> bool {
        self.parts
            .checkpoint
            .certification_checkpoint_under_pressure(1)
    }

    /// Drive one reclamation attempt through the production work port.
    pub fn certification_reclamation_work_yields_for_foreground_pressure(&self) -> bool {
        self.parts.checkpoint.certification_reclamation_under_pressure(1)
    }

    /// A quiet foreground admits the reclamation quantum through the work port.
    pub fn certification_reclamation_work_admits_when_foreground_is_idle(&self) -> bool {
        !self
            .parts
            .checkpoint
            .certification_reclamation_under_pressure(0)
    }

    pub fn certification_note_reclamation_background_head(&self) {
        self.parts
            .scheduler_admission
            .note_wal_reclamation_background_head();
    }

    pub fn certification_fail_next_checkpoint_admission(&self) {
        self.parts
            .checkpoint
            .certification_fail_next_admission();
    }

    pub fn certification_foreground_is_blocked_by_background(&self) -> bool {
        use worth_store_io_scheduler::foreground_reservation::{
            BandwidthToken, DirtyPageBudget, ForegroundLaneDeclaration, ForegroundResourceBudget,
            QueueSlot, WorkerPermit, WriteBackWindow,
        };
        let lane = ForegroundLaneDeclaration::ordinary_page_write().with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(QueueSlot::new(1).expect("one foreground slot is nonzero"))
                .with_bandwidth(BandwidthToken::bytes(4_096).expect("bandwidth is nonzero"))
                .with_write_back(WriteBackWindow::pages(1).expect("one page is nonzero"))
                .with_dirty_pages(DirtyPageBudget::pages(1).expect("one dirty page is nonzero"))
                .with_worker_permits(WorkerPermit::new(1).expect("one worker is nonzero")),
        );
        matches!(
            self.parts.scheduler_admission.reserve_record_lane(
                lane,
                self.parts.record_work.scheduler_security(),
            ),
            Err(crate::physical_runtime::RecordSchedulerReservationDenial::OwedBackgroundTurn)
        )
    }

    pub fn certification_cancel_checkpoint_background_head(&self) {
        self.parts
            .scheduler_admission
            .cancel_checkpoint_background_head();
    }

    pub fn certification_cancel_wal_reclamation_background_head(&self) {
        self.parts
            .scheduler_admission
            .cancel_wal_reclamation_background_head();
    }

    pub fn certification_publication_summary(
        &self,
    ) -> Result<
        crate::physical_runtime::record_serving::PhysicalRecordPublicationSummary,
        crate::physical_runtime::record_serving::RecordCanonicalObservationDenial,
    > {
        let (root, free_space) = self.parts.publication.planning_snapshot();
        let allocation = self
            .parts
            .residency
            .ports()
            .begin_operation(
                worth_store_buffer_pool::PhysicalOperationAllocationScope::Verification,
                std::num::NonZeroU64::new(u64::from(
                    self.parts.format.declaration().page_size().bytes(),
                ))
                .expect("an admitted physical page size is nonzero"),
            )
            .map_err(|_| {
                crate::physical_runtime::record_serving::RecordCanonicalObservationDenial::ManifestUnavailable
            })?;
        let integrity_counters = self
            .parts
            .residency
            .ports()
            .resident_integrity_counter_owner();
        crate::physical_runtime::record_serving::evidence::canonical_observation::observe_runtime_topology(
            crate::physical_runtime::record_serving::evidence::canonical_observation::RuntimeTopologySource {
                allocation: &allocation,
                media: self.parts.work_runtime.executor.record_serving_media(),
                frame_load: self.parts.residency.ports().loader(),
                format: self.parts.format,
                access: self.parts.access,
                root: &root,
                free_space: &free_space,
                lifecycle: self.parts.core.lifecycle_state(),
                integrity_counters: integrity_counters.as_ref(),
            },
        )
    }

    pub fn certification_frame_port_observer(
        &self,
    ) -> crate::physical_runtime::record_serving::FramePortCounterObserver {
        self.parts.residency.ports().observer()
    }

    pub fn certification_reject_next_candidate_publication_after_physical_write(&self) {
        self.parts
            .residency
            .ports()
            .reject_next_candidate_publication();
    }

    pub fn certification_fail_next_wal_member_before_effect(&self) {
        self.parts
            .publication
            .fail_next_wal_member_before_effect();
    }

    pub fn certification_reject_next_candidate_retention_before_effect(&self) {
        self.parts
            .residency
            .ports()
            .reject_next_candidate_retention();
    }
}
