pub(crate) use {std::fs, std::num::NonZeroU64, std::path::Path};

pub(crate) use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::BlobAppendFailure, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimFailure, worth_store::physical_runtime::BlobReclaimPublicationStage, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIndeterminateStage, worth_store::physical_runtime::PhysicalWorkCounterSnapshot, worth_store::physical_runtime::PhysicalWorkCounterStage, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkPressureClass, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration, worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope, worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget, worth_store_io_scheduler::foreground_reservation::QueueSlot, worth_store_io_scheduler::foreground_reservation::WorkerPermit};
pub(crate) use {worth_store_physical_backend::MediaCounterSnapshot};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordKind, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::PersistedRecordIdentity};

pub(crate) use {crate::physical_blob_journeys::blob_expiry::wal_oracle};

pub(crate) use {crate::physical_blob_journeys::blob_crash::establish_recovery_frontier, crate::blob_crash::recovery::recover_closed_store, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, crate::physical_blob_journeys::blob_reclaim::abandoned_prefix, crate::physical_blob_journeys::blob_reclaim::request, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};

pub(crate) const CHUNK: usize = 64 << 10;

pub(crate) fn hold_background_share(
    serving: &ServingPhysicalRuntime,
) -> worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation {
    let slots = serving
        .physical_scheduler_capacity()
        .configured()
        .queue_slots();
    let lane = worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
            "blob-producer-interference",
            1,
        ))
        .with_budget(
            worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
                .with_queue_slots(worth_store_io_scheduler::foreground_reservation::QueueSlot::new(slots / 2 + 1).unwrap())
                .with_worker_permits(worth_store_io_scheduler::foreground_reservation::WorkerPermit::new(1).unwrap()),
        );
    serving
        .reserve_physical_scheduler_foreground(lane)
        .expect("foreground can reserve the background share")
        .0
}

pub(crate) fn hold_one_foreground_slot(
    serving: &ServingPhysicalRuntime,
) -> worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation {
    let lane = worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
            "blob-producer-success",
            1,
        ))
        .with_budget(
            worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
                .with_queue_slots(worth_store_io_scheduler::foreground_reservation::QueueSlot::new(1).unwrap())
                .with_worker_permits(worth_store_io_scheduler::foreground_reservation::WorkerPermit::new(1).unwrap()),
        );
    serving
        .reserve_physical_scheduler_foreground(lane)
        .unwrap()
        .0
}

pub(crate) fn foreground_floor_remains_available(serving: &ServingPhysicalRuntime) {
    let lane = worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
            "blob-producer-foreground-floor",
            1,
        ))
        .with_budget(
            worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
                .with_queue_slots(worth_store_io_scheduler::foreground_reservation::QueueSlot::new(1).unwrap())
                .with_worker_permits(worth_store_io_scheduler::foreground_reservation::WorkerPermit::new(1).unwrap()),
        );
    for _ in 0..4 {
        serving
            .reserve_physical_scheduler_foreground(lane)
            .expect("a denied synchronous producer releases its dispatch head");
    }
}

pub(crate) fn terminal_count(serving: &ServingPhysicalRuntime, class: PhysicalWorkPressureClass) -> u64 {
    serving.physical_work_counters().count_under_pressure(
        worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactPublication,
        class,
        worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
    )
}

pub(crate) fn assert_no_producer_work(
    before: PhysicalWorkCounterSnapshot,
    after: PhysicalWorkCounterSnapshot,
    class: PhysicalWorkPressureClass,
) {
    for stage in [
        worth_store::physical_runtime::PhysicalWorkCounterStage::Declared,
        worth_store::physical_runtime::PhysicalWorkCounterStage::Ready,
        worth_store::physical_runtime::PhysicalWorkCounterStage::Queued,
        worth_store::physical_runtime::PhysicalWorkCounterStage::Dispatched,
        worth_store::physical_runtime::PhysicalWorkCounterStage::Settling,
        worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
    ] {
        assert_eq!(
            after.count_under_pressure(
                worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactPublication,
                class,
                stage
            ),
            before.count_under_pressure(
                worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactPublication,
                class,
                stage
            ),
            "denial must precede {class:?} work at {stage:?}"
        );
    }
}

pub(crate) fn assert_wal_prelude_without_root_replacement(
    before: MediaCounterSnapshot,
    after: MediaCounterSnapshot,
) {
    // Data dispatch is denied after its durable WAL prelude. Aggregate append,
    // sync, and cleanup counters also include WAL namespace bookkeeping.
    assert_eq!(
        after.positioned_write_attempts(),
        before.positioned_write_attempts() + 1
    );
    assert!(after.append_attempts() > before.append_attempts());
    assert!(after.file_syncs() > before.file_syncs());
    assert_eq!(after.replacements(), before.replacements());
}

pub(crate) fn arena_bytes(root: &Path) -> Vec<u8> {
    std::fs::read(root.join("families/records/arenas/arena-0000000000000001.data"))
        .expect("the selected blob record arena exists")
}

pub(crate) fn wal_end(root: &Path) -> u64 {
    crate::physical_blob_journeys::blob_expiry::wal_oracle::inspect_wal_inventory(root)
        .expect("independent retained WAL inventory")
        .lsn_range()
        .expect("nonempty retained WAL")
        .1
}

pub(crate) fn assert_wal_only_indeterminate(cause: &BlobAppendFailure) {
    let worth_store::physical_runtime::BlobAppendFailure::Indeterminate(fate) = cause else {
        panic!("durable WAL plus denied frame must be indeterminate: {cause:?}");
    };
    assert_eq!(
        fate.stage(),
        worth_store::physical_runtime::PhysicalMutationIndeterminateStage::DataDispatch
    );
    assert_eq!(fate.completed_effect_count(), 1);
}

pub(crate) fn assert_no_capacity_leak(
    held: worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundCapacitySnapshot,
    current: worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundCapacitySnapshot,
) {
    assert_eq!(current.available(), held.available());
    assert_eq!(current.active_reservations(), held.active_reservations());
    assert_eq!(
        current.denied_reservations(),
        held.denied_reservations() + 1
    );
    assert_eq!(
        current.admitted_reservations() - held.admitted_reservations(),
        current.released_reservations() - held.released_reservations(),
        "WAL prelude admissions and the denied producer must leave no lease"
    );
}

pub(crate) fn begin_one_chunk<'a>(
    serving: &'a ServingPhysicalRuntime,
    scope: &'a AdmittedBlobScope,
) -> worth_store::physical_runtime::BlobIngestSession<'a> {
    let blobs = serving.blobs().unwrap();
    let limits = worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
        object,
        worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        CHUNK as u64,
        scope,
        worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(32).unwrap(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    blobs
        .begin_ingest(declaration, placement(), (CHUNK / 2) as u64, limits)
        .unwrap()
}