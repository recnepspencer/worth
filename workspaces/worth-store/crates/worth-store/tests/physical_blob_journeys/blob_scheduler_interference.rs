use std::{fs, num::NonZeroU64, path::Path};

use worth_store::physical_runtime::{
    AdmittedBlobScope, BlobAppendFailure, BlobCheckpointLimit, BlobIngestDeclaration,
    BlobIngestFailure, BlobReadLimits, BlobReclaimDisposition, BlobReclaimFailure,
    BlobReclaimPublicationStage, PhysicalMutationDeadline, PhysicalMutationIndeterminateStage,
    PhysicalWorkCounterSnapshot, PhysicalWorkCounterStage, PhysicalWorkOperationFamily,
    PhysicalWorkPressureClass, ServingPhysicalRuntime,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_io_scheduler::foreground_reservation::{
    ForegroundLaneDeclaration, ForegroundLatencyEnvelope, ForegroundResourceBudget, QueueSlot,
    WorkerPermit,
};
use worth_store_physical_backend::MediaCounterSnapshot;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, PersistedRecordIdentity,
};

#[path = "../physical_record_journeys/durability_admission/independent_wal_oracle/segment_inventory.rs"]
mod wal_oracle;

use super::{
    blob_crash::{establish_recovery_frontier, recover_closed_store},
    blob_frontier::selected_blob_records,
    blob_reclaim::{abandoned_prefix, request},
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

const CHUNK: usize = 64 << 10;

fn hold_background_share(
    serving: &ServingPhysicalRuntime,
) -> worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation {
    let slots = serving
        .physical_scheduler_capacity()
        .configured()
        .queue_slots();
    let lane = ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "blob-producer-interference",
            1,
        ))
        .with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(QueueSlot::new(slots / 2 + 1).unwrap())
                .with_worker_permits(WorkerPermit::new(1).unwrap()),
        );
    serving
        .reserve_physical_scheduler_foreground(lane)
        .expect("foreground can reserve the background share")
        .0
}

fn hold_one_foreground_slot(
    serving: &ServingPhysicalRuntime,
) -> worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundReservation {
    let lane = ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "blob-producer-success",
            1,
        ))
        .with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(QueueSlot::new(1).unwrap())
                .with_worker_permits(WorkerPermit::new(1).unwrap()),
        );
    serving
        .reserve_physical_scheduler_foreground(lane)
        .unwrap()
        .0
}

fn foreground_floor_remains_available(serving: &ServingPhysicalRuntime) {
    let lane = ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "blob-producer-foreground-floor",
            1,
        ))
        .with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(QueueSlot::new(1).unwrap())
                .with_worker_permits(WorkerPermit::new(1).unwrap()),
        );
    for _ in 0..4 {
        serving
            .reserve_physical_scheduler_foreground(lane)
            .expect("a denied synchronous producer releases its dispatch head");
    }
}

fn terminal_count(serving: &ServingPhysicalRuntime, class: PhysicalWorkPressureClass) -> u64 {
    serving.physical_work_counters().count_under_pressure(
        PhysicalWorkOperationFamily::ArtifactPublication,
        class,
        PhysicalWorkCounterStage::Terminal,
    )
}

fn assert_no_producer_work(
    before: PhysicalWorkCounterSnapshot,
    after: PhysicalWorkCounterSnapshot,
    class: PhysicalWorkPressureClass,
) {
    for stage in [
        PhysicalWorkCounterStage::Declared,
        PhysicalWorkCounterStage::Ready,
        PhysicalWorkCounterStage::Queued,
        PhysicalWorkCounterStage::Dispatched,
        PhysicalWorkCounterStage::Settling,
        PhysicalWorkCounterStage::Terminal,
    ] {
        assert_eq!(
            after.count_under_pressure(
                PhysicalWorkOperationFamily::ArtifactPublication,
                class,
                stage
            ),
            before.count_under_pressure(
                PhysicalWorkOperationFamily::ArtifactPublication,
                class,
                stage
            ),
            "denial must precede {class:?} work at {stage:?}"
        );
    }
}

fn assert_wal_prelude_without_root_replacement(
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

fn arena_bytes(root: &Path) -> Vec<u8> {
    fs::read(root.join("families/records/arenas/arena-0000000000000001.data"))
        .expect("the selected blob record arena exists")
}

fn wal_end(root: &Path) -> u64 {
    wal_oracle::inspect_wal_inventory(root)
        .expect("independent retained WAL inventory")
        .lsn_range()
        .expect("nonempty retained WAL")
        .1
}

fn assert_wal_only_indeterminate(cause: &BlobAppendFailure) {
    let BlobAppendFailure::Indeterminate(fate) = cause else {
        panic!("durable WAL plus denied frame must be indeterminate: {cause:?}");
    };
    assert_eq!(
        fate.stage(),
        PhysicalMutationIndeterminateStage::DataDispatch
    );
    assert_eq!(fate.completed_effect_count(), 1);
}

fn assert_no_capacity_leak(
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

fn begin_one_chunk<'a>(
    serving: &'a ServingPhysicalRuntime,
    scope: &'a AdmittedBlobScope,
) -> worth_store::physical_runtime::BlobIngestSession<'a> {
    let blobs = serving.blobs().unwrap();
    let limits = BlobReadLimits::new(NonZeroU64::new(128).unwrap());
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(CHUNK as u64).unwrap(),
        CHUNK as u64,
        scope,
        BlobCheckpointLimit::bounded_horizon(32).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    blobs
        .begin_ingest(declaration, placement(), (CHUNK / 2) as u64, limits)
        .unwrap()
}

#[test]
fn real_ingest_frame_denial_preserves_foreground_floor_and_later_spends_ingest_class() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope("c11.blob.scheduler.ingest.scope");
    let light_foreground = hold_one_foreground_slot(&serving);
    let before_ingest = terminal_count(&serving, PhysicalWorkPressureClass::BackgroundBlobIngest);
    let mut admitted = begin_one_chunk(&serving, &scope);
    admitted.push(&[0x7b; CHUNK / 2]).unwrap();
    admitted.push(&[0x7b; CHUNK / 2]).unwrap();
    admitted.finish().unwrap();
    assert!(
        terminal_count(&serving, PhysicalWorkPressureClass::BackgroundBlobIngest) > before_ingest
    );
    drop(light_foreground);

    let mut ingest = begin_one_chunk(&serving, &scope);
    let before_records = selected_blob_records(&serving);
    let before_arena = arena_bytes(directory.path());
    let before_wal_end = wal_end(directory.path());
    let blocker = hold_background_share(&serving);
    let held_capacity = serving.physical_scheduler_capacity();
    let before_media = serving.media_counters();
    let before_work = serving.physical_work_counters();

    ingest.push(&[0x5a; CHUNK / 2]).unwrap();
    let denied = ingest.push(&[0x5a; CHUNK / 2]);
    let Err(BlobIngestFailure::Append {
        kind: BlobRecordKind::Chunk,
        cause,
        ..
    }) = &denied
    else {
        panic!("the chunk must reach its real append producer: {denied:?}");
    };
    assert_wal_only_indeterminate(cause);
    assert_eq!(wal_end(directory.path()), before_wal_end + 1);
    assert_wal_prelude_without_root_replacement(before_media, serving.media_counters());
    assert_no_producer_work(
        before_work,
        serving.physical_work_counters(),
        PhysicalWorkPressureClass::BackgroundBlobIngest,
    );
    assert_eq!(selected_blob_records(&serving), before_records);
    assert_eq!(arena_bytes(directory.path()), before_arena);
    assert_no_capacity_leak(held_capacity, serving.physical_scheduler_capacity());
    foreground_floor_remains_available(&serving);
    drop(ingest);
    drop(blocker);
    assert_eq!(
        serving.physical_scheduler_capacity().active_reservations(),
        0
    );
    drop(serving);
    recover_closed_store(directory.path());
    let reopened = serving_from_open(directory.path());
    let recovered = selected_blob_records(&reopened);
    assert_eq!(recovered.len(), before_records.len() + 1);
    for prior in &before_records {
        assert!(
            recovered.contains(prior),
            "C.8 must retain the prior selected route"
        );
    }
    let added = recovered
        .iter()
        .filter(|(record, _)| !before_records.iter().any(|(prior, _)| prior == record))
        .collect::<Vec<_>>();
    let [(_, bytes)] = added.as_slice() else {
        panic!("C.8 must replay exactly one WAL-bound chunk: {added:?}");
    };
    let BlobRecordV1::Chunk(chunk) = decode_blob_record(bytes).unwrap() else {
        panic!("C.8 must replay the denied ingest chunk");
    };
    assert_eq!(chunk.bytes(), &[0x5a; CHUNK]);
    reopened.close();
}

#[test]
fn real_reclaim_frame_denial_preserves_foreground_floor_and_later_spends_reclaim_class() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope("c11.blob.scheduler.reclaim.scope");
    let token = abandoned_prefix(&serving, &scope);
    let light_foreground = hold_one_foreground_slot(&serving);
    let before_reclaim = terminal_count(&serving, PhysicalWorkPressureClass::BackgroundBlobReclaim);
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    assert!(
        terminal_count(&serving, PhysicalWorkPressureClass::BackgroundBlobReclaim) > before_reclaim
    );
    drop(light_foreground);

    let before_records = selected_blob_records(&serving);
    let before_arena = arena_bytes(directory.path());
    let before_wal_end = wal_end(directory.path());
    let blocker = hold_background_share(&serving);
    let held_capacity = serving.physical_scheduler_capacity();
    let before_media = serving.media_counters();
    let before_work = serving.physical_work_counters();

    let denied = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .expect("selected reclaim admits a pre-effect handle")
        .wait();
    let Err(BlobReclaimFailure::Publication {
        stage: BlobReclaimPublicationStage::Manifest,
        manifest_record: None,
        cause,
    }) = &denied
    else {
        panic!("selected reclaim must reach its real control-frame producer: {denied:?}");
    };
    assert_wal_only_indeterminate(cause);
    assert_eq!(wal_end(directory.path()), before_wal_end + 1);
    assert_wal_prelude_without_root_replacement(before_media, serving.media_counters());
    assert_no_producer_work(
        before_work,
        serving.physical_work_counters(),
        PhysicalWorkPressureClass::BackgroundBlobReclaim,
    );
    // The reclaim fence intentionally blocks selected scans until C.8 settles
    // the durable WAL, so inspect arena/root media instead of bypassing it.
    assert_eq!(arena_bytes(directory.path()), before_arena);
    assert_no_capacity_leak(held_capacity, serving.physical_scheduler_capacity());
    foreground_floor_remains_available(&serving);
    drop(blocker);

    assert_eq!(
        serving.physical_scheduler_capacity().active_reservations(),
        0
    );
    drop(serving);
    recover_closed_store(directory.path());
    let reopened = serving_from_open(directory.path());
    let recovered = selected_blob_records(&reopened);
    assert_eq!(recovered.len(), before_records.len() + 1);
    for survivor in &before_records {
        assert!(
            recovered.contains(survivor),
            "a manifest WAL alone must not drop any selected record"
        );
    }
    let added = recovered
        .iter()
        .filter(|(record, _)| !before_records.iter().any(|(prior, _)| prior == record))
        .collect::<Vec<_>>();
    let [(_, bytes)] = added.as_slice() else {
        panic!("C.8 must replay exactly one WAL-bound manifest: {added:?}");
    };
    let BlobRecordV1::DropSetManifestV2(manifest) = decode_blob_record(bytes).unwrap() else {
        panic!("the manifest stage WAL cannot authorize a drop descriptor");
    };
    assert_eq!(manifest.count(), 1);
    let [candidate] = manifest.dropped() else {
        panic!("the one-record reclaim manifest must name exactly one candidate");
    };
    let (_, candidate_bytes) = recovered
        .iter()
        .find(|(record, _)| {
            PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal())
                == Some(*candidate)
        })
        .expect("manifest-only replay must leave its candidate selected");
    assert!(matches!(
        decode_blob_record(candidate_bytes),
        Ok(BlobRecordV1::Chunk(_))
    ));
    reopened.close();
}
