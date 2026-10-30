#[allow(unused_imports)]
use crate::physical_blob_journeys::blob_corruption_localization::*;
use {std::fs, std::num::NonZeroU64, std::path::Path, std::time::Duration};

use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::BlobRecordDenial;
use {
    worth_store::physical_runtime::BlobCheckpointLimit,
    worth_store::physical_runtime::BlobIngestDeclaration,
    worth_store::physical_runtime::BlobReadFailure, worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::LayoutRebuildFailure,
    worth_store::physical_runtime::LayoutRebuildLimits,
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress,
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest,
    worth_store::physical_runtime::PhysicalIndexPointKey,
    worth_store::physical_runtime::PhysicalIntegrityScrubDeferral,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalWorkCounterStage,
    worth_store::physical_runtime::PhysicalWorkOperationFamily,
    worth_store::physical_runtime::PhysicalWorkPressureClass,
    worth_store::physical_runtime::PublishedBlobGeneration,
    worth_store::physical_runtime::RecordReadDenial,
    worth_store::physical_runtime::RecordReadWorkDenial,
};
use {
    worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration,
    worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope,
    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget,
    worth_store_io_scheduler::foreground_reservation::QueueSlot,
};
use {
    worth_store_physical_integrity::PhysicalIntegrityObservationOutcome,
    worth_store_physical_integrity::PhysicalIntegrityRejection,
};

use {
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::fixture::serving_from_open,
};

#[test]
fn damaged_middle_chunk_localizes_without_poisoning_other_ranges() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.chunk.corruption.scope");
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(512).unwrap());
    let (published, object) = {
        let serving = serving_from_initialization(directory.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = worth_store::physical_runtime::BlobIngestDeclaration::new(
            object,
            worth_store_blob_chunks::BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (3 * CHUNK_BYTES) as u64,
            &scope,
            worth_store::physical_runtime::BlobCheckpointLimit::bounded_horizon(64).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
            .unwrap();
        for ordinal in 0..3_u8 {
            ingest.push(&vec![ordinal + 0x31; CHUNK_BYTES]).unwrap();
        }
        let published = ingest.finish().unwrap();
        serving.close();
        (published, object)
    };

    flip_middle_chunk_payload_reseal_outer(directory.path());
    let report = observe_closed_store_named(directory.path(), "c11-middle-chunk", "damaged");
    assert_eq!(report["completeness"], "complete", "{report}");
    let chunks = report["artifacts"].as_array().unwrap();
    let damaged_rows = chunks
        .iter()
        .filter(|row| row["family"] == "blob_chunk_frame" && row["outcome"]["posture"] == "damaged")
        .collect::<Vec<_>>();
    assert_eq!(
        damaged_rows.len(),
        1,
        "only the middle chunk is damaged: {report}"
    );
    let offline_selected =
        crate::physical_blob_journeys::blob_corruption_localization::comparison::observe_selected(
            directory.path(),
        );
    let serving = serving_from_open(directory.path());
    let blobs = serving.blobs().unwrap();
    let resolved = blobs
        .resolve_publication(
            object.bytes(),
            published.generation().sequence(),
            &scope,
            limits,
        )
        .unwrap();
    assert_eq!(resolved, published);
    assert_eq!(read_one_byte(&blobs, resolved, &scope, 0, limits), 0x31);
    assert_eq!(
        read_one_byte(&blobs, resolved, &scope, (2 * CHUNK_BYTES) as u64, limits),
        0x33
    );
    let mut damaged = blobs
        .read(resolved, &scope, CHUNK_BYTES as u64, 1, limits)
        .unwrap();
    let error = damaged.read_next(&mut [0_u8; 1]).unwrap_err();
    assert!(
        matches!(&error, worth_store::physical_runtime::BlobReadFailure::ChunkCorruption {
            ordinal: 1,
            expected,
            observed,
            ..
        } if expected != observed),
        "inner digest damage must identify the middle chunk: {error:?}"
    );
    let record = match error {
        worth_store::physical_runtime::BlobReadFailure::ChunkCorruption { record, .. } => record,
        other => panic!("expected localized chunk corruption: {other:?}"),
    };
    assert_eq!(
        damaged_rows[0]["identity"],
        format!("blob-record:{}", hex_record(record)),
        "offline media and runtime must identify the same selected record"
    );
    let target = damaged
        .damaged_chunk_scrub_target()
        .unwrap()
        .expect("typed selected corruption issues an edge-bound scrub target");
    let selected_root = target.issued_under().unwrap().root();
    assert_eq!(
        read_one_byte(&blobs, resolved, &scope, 0, limits),
        0x31,
        "a failed middle read must not poison disjoint selected ranges"
    );
    assert_eq!(
        read_one_byte(&blobs, resolved, &scope, (2 * CHUNK_BYTES) as u64, limits),
        0x33
    );
    let catalog_key = worth_store::physical_runtime::PhysicalIndexPointKey::blob_catalog(
        object,
        published.generation().sequence(),
    )
    .unwrap();
    let before_rebuild = serving
        .layouts()
        .unwrap()
        .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
        .unwrap()
        .point(catalog_key)
        .unwrap()
        .selected_record();
    let rebuild = serving.layouts().unwrap().rebuild(
        worth_store_contracts::DurableArtifactFamilyId::BlobCatalog,
        worth_store::physical_runtime::LayoutRebuildLimits::new(
            std::num::NonZeroU64::new(1_000).unwrap(),
            std::num::NonZeroU64::new(1_000).unwrap(),
        ),
        placement(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    assert!(
        matches!(
            rebuild,
            Err(
                worth_store::physical_runtime::LayoutRebuildFailure::MalformedSelectedBlob(
                    worth_store_physical_format::BlobRecordDenial::IntegrityMismatch
                )
            )
        ),
        "authoritative damaged chunk must block derived rebuild before root effects: {rebuild:?}"
    );
    assert_eq!(
        serving
            .layouts()
            .unwrap()
            .btree(worth_store_contracts::DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(catalog_key)
            .unwrap()
            .selected_record(),
        before_rebuild,
    );
    drop(damaged);
    assert!(
        serving
            .certification_physical_residency()
            .drain_unpinned_clean_frames()
            > 0,
        "selected diagnostic read must fault from C5 media"
    );
    let request = worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        1 << 20,
        1 << 20,
        std::time::Duration::from_secs(30),
    )
    .unwrap();
    let mut scrub = serving.start_physical_integrity_scrub(request).unwrap();
    let before = serving.physical_work_counters();
    let capacity = serving.physical_scheduler_capacity();
    let held = serving
        .reserve_physical_scheduler_foreground(
            worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::artifact_metadata_read()
                .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
                    "selected-scrub-pressure",
                    1,
                ))
                .with_budget(
                    worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
                        .with_queue_slots(
                            worth_store_io_scheduler::foreground_reservation::QueueSlot::new(capacity.configured().queue_slots() / 2 + 1).unwrap(),
                        )
                        .with_worker_permits(
                            worth_store_io_scheduler::WorkerPermit::new(1).unwrap(),
                        ),
                ),
        )
        .unwrap();
    let before_media = serving.media_counters();
    assert!(matches!(
        scrub.next_window(),
        worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Deferred(
            worth_store::physical_runtime::PhysicalIntegrityScrubDeferral::SelectedRecordRead(worth_store::physical_runtime::RecordReadDenial::PhysicalWork(
                worth_store::physical_runtime::RecordReadWorkDenial::SchedulerRejected
                    | worth_store::physical_runtime::RecordReadWorkDenial::SchedulerReservationRejected
            ))
        )
    ));
    assert_eq!(
        serving.media_counters(),
        before_media,
        "denial precedes C5 media access"
    );
    assert_eq!(scrub.counters().completed_windows, 0);
    assert_eq!(scrub.counters().acquired_bytes, 0);
    drop(held);
    let worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(
        window,
    ) = scrub.next_window()
    else {
        panic!("selected chunk scrub must inspect one bounded window")
    };
    let after = serving.physical_work_counters();
    assert!(
        after.count_under_pressure(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
            worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundScrub,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal
        ) > before.count_under_pressure(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
            worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundScrub,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal
        ),
        "selected C5 frame miss must execute under scrub-background pressure"
    );
    for foreground in [
        worth_store::physical_runtime::PhysicalWorkPressureClass::ForegroundPointRead,
        worth_store::physical_runtime::PhysicalWorkPressureClass::ForegroundRangeRead,
        worth_store::physical_runtime::PhysicalWorkPressureClass::ForegroundInteractiveRead,
        worth_store::physical_runtime::PhysicalWorkPressureClass::ForegroundInternalRead,
    ] {
        assert_eq!(
            after.count_under_pressure(
                worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
                foreground,
                worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal
            ),
            before.count_under_pressure(
                worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
                foreground,
                worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal
            ),
            "selected scrub must not substitute foreground C5 reads"
        );
    }
    assert_eq!(window.scope.blob_record_identity().unwrap().0, record);
    assert!(matches!(
        window.outcome,
        worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(
            localization
        )) if localization.scope().blob_record_identity().unwrap().0 == record
    ));
    assert_eq!(window.counters.damaged_windows, 1);

    drop(scrub);
    crate::physical_blob_journeys::blob_corruption_localization::comparison::assert_selected_join(
        &serving,
        target,
        selected_root,
        record,
        &offline_selected,
    );
}
