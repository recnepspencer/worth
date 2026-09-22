use worth_store_io_scheduler::{
    IoSchedulerBackendCapabilityAdmission, IoSchedulerBackendCapabilityDenial,
    IoSchedulerBackendCapabilityRequirement,
};
#[cfg(feature = "recovery-runtime-owner")]
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_backend::QualifiedFilesystemMedia;

mod background_head;
mod capacity;
mod checkpoint;
mod scrub;
pub(in crate::physical_runtime) use scrub::PhysicalScrubSchedulerAdmissionDenial;
mod reclamation;
#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use reclamation::PhysicalWalReclamationSchedulerAdmissionDenial;
mod root_publication;
#[cfg(feature = "certification-test-authority")] mod certification;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordSchedulerReservationDenial {
    Admission(
        worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial,
    ),
    OwedBackgroundTurn,
}

/// Store-owned admission from qualified media evidence. It owns no media.
#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalSchedulerAdmissionOwner {
    buffered_file: IoSchedulerBackendCapabilityAdmission,
    fsync: IoSchedulerBackendCapabilityAdmission,
    directory_sync: IoSchedulerBackendCapabilityAdmission,
    durable_rename: IoSchedulerBackendCapabilityAdmission,
    foreground:
        worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundCapacity,
    dispatch: worth_store_io_scheduler::PhysicalDispatchSelection,
    effects: crate::physical_runtime::work::PhysicalEffectAdmission,
    heads: std::sync::Arc<background_head::RetainedBackgroundHeads>,
}

impl PhysicalSchedulerAdmissionOwner {
    pub(super) fn new(
        media: &QualifiedFilesystemMedia,
        capacity: crate::physical_runtime::PhysicalWorkCapacity,
    ) -> Result<Self, IoSchedulerBackendCapabilityDenial> {
        let owner = Self {
            buffered_file: admit(media, IoSchedulerBackendCapabilityRequirement::BufferedFile)?,
            fsync: admit(
                media,
                IoSchedulerBackendCapabilityRequirement::FilesystemAdmittedFsync,
            )?,
            directory_sync: admit(
                media,
                IoSchedulerBackendCapabilityRequirement::FilesystemAdmittedDirectorySync,
            )?,
            durable_rename: admit(
                media,
                IoSchedulerBackendCapabilityRequirement::FilesystemAdmittedDurableRename,
            )?,
            foreground: worth_store_io_scheduler::foreground_reservation::
                PhysicalInstanceForegroundCapacity::new(capacity::foreground_capacity(capacity))
                .expect("an admitted physical-work profile has nonzero scheduler capacity"),
            dispatch: worth_store_io_scheduler::PhysicalDispatchSelection::new(),
            effects: crate::physical_runtime::work::PhysicalEffectAdmission::new(
                capacity.commands(),
            ),
            heads: std::sync::Arc::new(background_head::RetainedBackgroundHeads::default()),
        };
        Ok(owner)
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn new_recovery(
        media: &AdmittedRecoveryFilesystemMedia,
        capacity: crate::physical_runtime::PhysicalWorkCapacity,
    ) -> Result<Self, IoSchedulerBackendCapabilityDenial> {
        Ok(Self {
            buffered_file: admit_recovery(
                media,
                IoSchedulerBackendCapabilityRequirement::BufferedFile,
            )?,
            fsync: admit_recovery(
                media,
                IoSchedulerBackendCapabilityRequirement::FilesystemAdmittedFsync,
            )?,
            directory_sync: admit_recovery(
                media,
                IoSchedulerBackendCapabilityRequirement::FilesystemAdmittedDirectorySync,
            )?,
            durable_rename: admit_recovery(
                media,
                IoSchedulerBackendCapabilityRequirement::FilesystemAdmittedDurableRename,
            )?,
            foreground: worth_store_io_scheduler::foreground_reservation::
                PhysicalInstanceForegroundCapacity::new(capacity::foreground_capacity(capacity))
                .expect("an admitted recovery profile has nonzero scheduler capacity"),
            dispatch: worth_store_io_scheduler::PhysicalDispatchSelection::new(),
            effects: crate::physical_runtime::work::PhysicalEffectAdmission::new(
                capacity.commands(),
            ),
            heads: std::sync::Arc::new(background_head::RetainedBackgroundHeads::default()),
        })
    }

    pub(in crate::physical_runtime) fn wal_append(
        &self,
        security: &worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
        bytes: u64,
    ) -> Result<
        (
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation,
            IoSchedulerBackendCapabilityAdmission,
        ),
        RecordSchedulerReservationDenial,
    > {
        let lane = worth_store_io_scheduler::foreground_reservation::
            ForegroundLaneDeclaration::commit_critical_wal_append()
            .with_latency_envelope(
                worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::
                    bounded_interference("physical-wal-append", 2),
            )
            .with_budget(wal_append_budget(bytes));
        let reservation =
            self.reserve_selected_foreground(lane, &self.buffered_file, security)?;
        Ok((reservation, self.buffered_file))
    }

    pub(in crate::physical_runtime) fn wal_durability_barrier(
        &self,
        security: &worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    ) -> Result<
        (
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation,
            IoSchedulerBackendCapabilityAdmission,
        ),
        RecordSchedulerReservationDenial,
    > {
        let lane = worth_store_io_scheduler::foreground_reservation::
            ForegroundLaneDeclaration::filesystem_admitted_wal_barrier()
            .expect("filesystem-admitted WAL barrier is a Store-owned lane")
            .with_latency_envelope(
                worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::
                    bounded_interference("physical-wal-durability-barrier", 2),
            )
            .with_budget(wal_barrier_budget());
        let reservation = self.reserve_selected_foreground(lane, &self.fsync, security)?;
        Ok((reservation, self.fsync))
    }

    pub(in crate::physical_runtime) fn admit(
        &self,
        media: &QualifiedFilesystemMedia,
        requirement: IoSchedulerBackendCapabilityRequirement,
    ) -> Result<IoSchedulerBackendCapabilityAdmission, IoSchedulerBackendCapabilityDenial> {
        admit(media, requirement)
    }

    pub(in crate::physical_runtime) fn record_read(
        &self,
        security: &worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
        bytes: u64,
    ) -> Result<
        (
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation,
            IoSchedulerBackendCapabilityAdmission,
        ),
        RecordSchedulerReservationDenial,
    > {
        let lane = worth_store_io_scheduler::foreground_reservation::
            ForegroundLaneDeclaration::buffered_file_internal_foreground_read()
            .expect("buffered record reads are a Store-owned lane")
            .with_latency_envelope(
                worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::
                    bounded_interference("physical-record-read", 8),
            )
            .with_budget(read_budget(bytes));
        self.reserve_record_lane(lane, security)
    }

    pub(in crate::physical_runtime) fn record_metadata(
        &self,
        security: &worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    ) -> Result<
        (
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation,
            IoSchedulerBackendCapabilityAdmission,
        ),
        RecordSchedulerReservationDenial,
    > {
        let lane = worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::
            artifact_metadata_read()
            .with_latency_envelope(
                worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::
                    bounded_interference("physical-record-metadata", 8),
            )
            .with_budget(metadata_budget());
        self.reserve_record_lane(lane, security)
    }

    pub(in crate::physical_runtime) fn record_write(
        &self,
        security: &worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
        bytes: u64,
        synchronization: bool,
        publication: bool,
    ) -> Result<
        (
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation,
            IoSchedulerBackendCapabilityAdmission,
        ),
        RecordSchedulerReservationDenial,
    > {
        let lane = worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::
            ordinary_page_write()
            .with_latency_envelope(
                worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::
                    bounded_interference("physical-record-publication", 8),
            )
            .with_budget(write_budget(bytes, synchronization, publication));
        self.reserve_record_lane(lane, security)
    }

    pub(in crate::physical_runtime) fn reserve_record_lane(
        &self,
        lane: worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration,
        security: &worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    ) -> Result<
        (
            worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation,
            IoSchedulerBackendCapabilityAdmission,
        ),
        RecordSchedulerReservationDenial,
    > {
        let reservation =
            self.reserve_selected_foreground(lane, &self.buffered_file, security)?;
        Ok((reservation, self.buffered_file))
    }

    pub(in crate::physical_runtime) fn note_ready_background(&self) {
        self.dispatch.note_ready_background();
    }

    pub(in crate::physical_runtime) fn release_ready_background(&self) {
        self.dispatch.release_ready_background();
    }

    pub(in crate::physical_runtime) fn note_wal_reclamation_background_head(&self) { self.heads.note(background_head::BackgroundHeadKind::Reclamation, &self.dispatch); }

    pub(in crate::physical_runtime) fn note_checkpoint_background_head(&self) {
        self.heads.note(background_head::BackgroundHeadKind::Checkpoint, &self.dispatch);
    }

    pub(in crate::physical_runtime) fn cancel_checkpoint_background_head(&self) {
        self.heads.cancel(background_head::BackgroundHeadKind::Checkpoint, &self.dispatch);
    }

    pub(in crate::physical_runtime) fn cancel_wal_reclamation_background_head(&self) {
        self.heads.cancel(background_head::BackgroundHeadKind::Reclamation, &self.dispatch);
    }

    pub(in crate::physical_runtime) fn effects(
        &self,
    ) -> &crate::physical_runtime::work::PhysicalEffectAdmission {
        &self.effects
    }

    fn reserve_selected_foreground(
        &self,
        lane: worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration,
        backend: &IoSchedulerBackendCapabilityAdmission,
        security: &worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    ) -> Result<
        worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation,
        RecordSchedulerReservationDenial,
    > {
        let turn = self
            .dispatch
            .begin_foreground()
            .map_err(|_| RecordSchedulerReservationDenial::OwedBackgroundTurn)?;
        capacity::deny_queue_headroom(self.foreground.snapshot(), lane.requested_budget())?;
        let reservation = self.foreground.reserve(lane, backend, security).map_err(
            RecordSchedulerReservationDenial::Admission,
        )?;
        turn.commit();
        Ok(reservation)
    }

    pub(in crate::physical_runtime) fn capacity_snapshot(
        &self,
    ) -> worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundCapacitySnapshot
    {
        self.foreground.snapshot()
    }
}

fn admit(
    media: &QualifiedFilesystemMedia,
    requirement: IoSchedulerBackendCapabilityRequirement,
) -> Result<IoSchedulerBackendCapabilityAdmission, IoSchedulerBackendCapabilityDenial> {
    let claim = media
        .scheduler_capability_claim(
            requirement.capability_kind(),
            requirement.required_evidence(),
        )
        .map_err(IoSchedulerBackendCapabilityDenial::BackendCapabilityDenied)?;
    worth_store_io_scheduler::admit_backend_capability_for_scheduler_qualified_claim(
        claim,
        requirement,
    )
}

#[cfg(feature = "recovery-runtime-owner")]
fn admit_recovery(
    media: &AdmittedRecoveryFilesystemMedia,
    requirement: IoSchedulerBackendCapabilityRequirement,
) -> Result<IoSchedulerBackendCapabilityAdmission, IoSchedulerBackendCapabilityDenial> {
    let claim = media
        .scheduler_capability_claim(
            requirement.capability_kind(),
            requirement.required_evidence(),
        )
        .map_err(IoSchedulerBackendCapabilityDenial::BackendCapabilityDenied)?;
    worth_store_io_scheduler::admit_backend_capability_for_scheduler_qualified_claim(
        claim,
        requirement,
    )
}

fn read_budget(
    bytes: u64,
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::{
        BandwidthToken, CacheResidencyHint, QueueSlot, ReadAheadWindow, WorkerPermit,
    };
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one queue slot is nonzero"))
        .with_bandwidth(BandwidthToken::bytes(bytes).expect("record coordinates are nonempty"))
        .with_read_ahead(ReadAheadWindow::pages(1).expect("one read-ahead page is nonzero"))
        .with_worker_permits(WorkerPermit::new(1).expect("one worker permit is nonzero"))
        .with_cache_residency(CacheResidencyHint::frames(1).expect("one frame hint is nonzero"))
}

fn metadata_budget() -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::{QueueSlot, WorkerPermit};
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one queue slot is nonzero"))
        .with_worker_permits(WorkerPermit::new(1).expect("one worker permit is nonzero"))
}

fn write_budget(
    bytes: u64,
    synchronization: bool,
    publication: bool,
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::{
        BandwidthToken, DirtyPageBudget, FlushPermit, QueueSlot, SyncDebt, WorkerPermit,
        WriteBackWindow,
    };
    let budget = worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one queue slot is nonzero"))
        .with_bandwidth(BandwidthToken::bytes(bytes).expect("record coordinates are nonempty"))
        .with_write_back(WriteBackWindow::pages(1).expect("one writeback page is nonzero"))
        .with_dirty_pages(DirtyPageBudget::pages(1).expect("one dirty page is nonzero"))
        .with_worker_permits(WorkerPermit::new(1).expect("one worker permit is nonzero"));
    let budget = if synchronization {
        budget.with_flush_permits(FlushPermit::new(1).expect("one flush permit is nonzero"))
    } else {
        budget
    };
    if publication {
        budget.with_sync_debt(SyncDebt::units(1).expect("one sync-debt unit is nonzero"))
    } else {
        budget
    }
}

fn wal_append_budget(
    bytes: u64,
) -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget {
    use worth_store_io_scheduler::{BandwidthToken, QueueSlot, WorkerPermit};
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one WAL append is nonzero"))
        .with_bandwidth(BandwidthToken::bytes(bytes).expect("an admitted WAL frame is nonempty"))
        .with_worker_permits(WorkerPermit::new(1).expect("one WAL append is nonzero"))
}

fn wal_barrier_budget() -> worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget
{
    use worth_store_io_scheduler::{
        BandwidthToken, FlushPermit, QueueSlot, SyncDebt, WorkerPermit,
    };
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
        .with_queue_slots(QueueSlot::new(1).expect("one WAL barrier is nonzero"))
        .with_bandwidth(BandwidthToken::bytes(1).expect("one barrier accounting unit is nonzero"))
        .with_flush_permits(FlushPermit::new(1).expect("one WAL barrier is nonzero"))
        .with_sync_debt(SyncDebt::units(1).expect("one WAL barrier is nonzero"))
        .with_worker_permits(WorkerPermit::new(1).expect("one WAL barrier is nonzero"))
}

