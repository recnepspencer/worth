#[allow(unused_imports)] use crate::physical_blob_journeys::blob_scheduler_interference::*;
use {std::fs, std::num::NonZeroU64, std::path::Path};

use {worth_store::physical_runtime::AdmittedBlobScope, worth_store::physical_runtime::BlobAppendFailure, worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobIngestFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobReclaimFailure, worth_store::physical_runtime::BlobReclaimPublicationStage, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIndeterminateStage, worth_store::physical_runtime::PhysicalWorkCounterSnapshot, worth_store::physical_runtime::PhysicalWorkCounterStage, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkPressureClass, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_blob_chunks::BlobChunkSize};
use {worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration, worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope, worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget, worth_store_io_scheduler::foreground_reservation::QueueSlot, worth_store_io_scheduler::foreground_reservation::WorkerPermit};
use {worth_store_physical_backend::MediaCounterSnapshot};
use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordKind, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::PersistedRecordIdentity};

use {crate::physical_blob_journeys::blob_expiry::wal_oracle};

use {crate::physical_blob_journeys::blob_crash::establish_recovery_frontier, crate::blob_crash::recovery::recover_closed_store, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, crate::physical_blob_journeys::blob_reclaim::abandoned_prefix, crate::physical_blob_journeys::blob_reclaim::request, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};

#[test]
fn real_ingest_frame_denial_preserves_foreground_floor_and_later_spends_ingest_class() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope("c11.blob.scheduler.ingest.scope");
    let light_foreground = hold_one_foreground_slot(&serving);
    let before_ingest = terminal_count(&serving, worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundBlobIngest);
    let mut admitted = begin_one_chunk(&serving, &scope);
    admitted.push(&[0x7b; CHUNK / 2]).unwrap();
    admitted.push(&[0x7b; CHUNK / 2]).unwrap();
    admitted.finish().unwrap();
    assert!(
        terminal_count(&serving, worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundBlobIngest) > before_ingest
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
    let Err(worth_store::physical_runtime::BlobIngestFailure::Append {
        kind: worth_store_physical_format::BlobRecordKind::Chunk,
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
        worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundBlobIngest,
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
    let worth_store_physical_format::BlobRecordV1::Chunk(chunk) = decode_blob_record(bytes).unwrap() else {
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
    let before_reclaim = terminal_count(&serving, worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundBlobReclaim);
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(request(token, &scope))
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(receipt.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
    assert!(
        terminal_count(&serving, worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundBlobReclaim) > before_reclaim
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
    let Err(worth_store::physical_runtime::BlobReclaimFailure::Publication {
        stage: worth_store::physical_runtime::BlobReclaimPublicationStage::Manifest,
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
        worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundBlobReclaim,
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
    let worth_store_physical_format::BlobRecordV1::DropSetManifestV2(manifest) = decode_blob_record(bytes).unwrap() else {
        panic!("the manifest stage WAL cannot authorize a drop descriptor");
    };
    assert_eq!(manifest.count(), 1);
    let [candidate] = manifest.dropped() else {
        panic!("the one-record reclaim manifest must name exactly one candidate");
    };
    let (_, candidate_bytes) = recovered
        .iter()
        .find(|(record, _)| {
            worth_store_physical_format::PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal())
                == Some(*candidate)
        })
        .expect("manifest-only replay must leave its candidate selected");
    assert!(matches!(
        decode_blob_record(candidate_bytes),
        Ok(worth_store_physical_format::BlobRecordV1::Chunk(_))
    ));
    reopened.close();
}
