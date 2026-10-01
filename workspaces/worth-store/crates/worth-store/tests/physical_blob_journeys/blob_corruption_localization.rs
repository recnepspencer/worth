use std::{fs, num::NonZeroU64, path::Path, time::Duration};

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadFailure, BlobReadLimits,
    LayoutRebuildFailure, LayoutRebuildLimits, ManagedPhysicalIntegrityScrubProgress,
    ManagedPhysicalIntegrityScrubRequest, PhysicalIndexPointKey, PhysicalIntegrityScrubDeferral,
    PhysicalMutationDeadline, PhysicalWorkCounterStage, PhysicalWorkOperationFamily,
    PhysicalWorkPressureClass, PublishedBlobGeneration, RecordReadDenial, RecordReadWorkDenial,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_io_scheduler::foreground_reservation::{
    ForegroundLaneDeclaration, ForegroundLatencyEnvelope, ForegroundResourceBudget, QueueSlot,
};
use worth_store_physical_format::BlobRecordDenial;
use worth_store_physical_integrity::{
    PhysicalIntegrityObservationOutcome, PhysicalIntegrityRejection,
};

use super::{
    blob_ingest_process::observe_closed_store_named,
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

#[path = "blob_corruption_localization/comparison.rs"]
mod comparison;

const CHUNK_BYTES: usize = 64 * 1024;
const C5_HEADER: usize = 48;
const C5_EXTENT_METADATA: usize = 64;
const C11_HEADER: usize = 48;
const C11_CHUNK_CLAIM: usize = 80;
const C11_CONTENT_PREFIX: usize = 12;

#[test]
fn damaged_middle_chunk_localizes_without_poisoning_other_ranges() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.chunk.corruption.scope");
    let limits = BlobReadLimits::new(NonZeroU64::new(512).unwrap());
    let (published, object) = {
        let serving = serving_from_initialization(directory.path());
        let blobs = serving.blobs().unwrap();
        let object = blobs.issue_object_id(limits).unwrap();
        let declaration = BlobIngestDeclaration::new(
            object,
            BlobChunkSize::from_bytes(CHUNK_BYTES as u64).unwrap(),
            (3 * CHUNK_BYTES) as u64,
            &scope,
            BlobCheckpointLimit::bounded_horizon(64).unwrap(),
            PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        )
        .unwrap();
        let mut ingest = blobs
            .begin_ingest(declaration, placement(), CHUNK_BYTES as u64, limits)
            .unwrap();
        for ordinal in 0..3_u8 {
            ingest.push(&vec![ordinal + 0x31; CHUNK_BYTES]).unwrap();
        }
        let published = ingest.finish().unwrap();
        drop(blobs);
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
    let offline_selected = comparison::observe_selected(directory.path());
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
        matches!(&error, BlobReadFailure::ChunkCorruption {
            ordinal: 1,
            expected,
            observed,
            ..
        } if expected != observed),
        "inner digest damage must identify the middle chunk: {error:?}"
    );
    let record = match error {
        BlobReadFailure::ChunkCorruption { record, .. } => record,
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
    let catalog_key =
        PhysicalIndexPointKey::blob_catalog(object, published.generation().sequence()).unwrap();
    let before_rebuild = serving
        .layouts()
        .unwrap()
        .btree(DurableArtifactFamilyId::BlobCatalog)
        .unwrap()
        .point(catalog_key)
        .unwrap()
        .selected_record();
    let rebuild = serving.layouts().unwrap().rebuild(
        DurableArtifactFamilyId::BlobCatalog,
        LayoutRebuildLimits::new(
            NonZeroU64::new(1_000).unwrap(),
            NonZeroU64::new(1_000).unwrap(),
        ),
        placement(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    assert!(
        matches!(
            rebuild,
            Err(LayoutRebuildFailure::MalformedSelectedBlob(
                BlobRecordDenial::IntegrityMismatch
            ))
        ),
        "authoritative damaged chunk must block derived rebuild before root effects: {rebuild:?}"
    );
    assert_eq!(
        serving
            .layouts()
            .unwrap()
            .btree(DurableArtifactFamilyId::BlobCatalog)
            .unwrap()
            .point(catalog_key)
            .unwrap()
            .selected_record(),
        before_rebuild,
    );
    drop(damaged);
    drop(blobs);
    assert!(
        serving
            .certification_physical_residency()
            .drain_unpinned_clean_frames()
            > 0,
        "selected diagnostic read must fault from C5 media"
    );
    let request = ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        1 << 20,
        1 << 20,
        Duration::from_secs(30),
    )
    .unwrap();
    let mut scrub = serving.start_physical_integrity_scrub(request).unwrap();
    let before = serving.physical_work_counters();
    let capacity = serving.physical_scheduler_capacity();
    let held = serving
        .reserve_physical_scheduler_foreground(
            ForegroundLaneDeclaration::artifact_metadata_read()
                .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
                    "selected-scrub-pressure",
                    1,
                ))
                .with_budget(
                    ForegroundResourceBudget::new()
                        .with_queue_slots(
                            QueueSlot::new(capacity.configured().queue_slots() / 2 + 1).unwrap(),
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
        ManagedPhysicalIntegrityScrubProgress::Deferred(
            PhysicalIntegrityScrubDeferral::SelectedRecordRead(RecordReadDenial::PhysicalWork(
                RecordReadWorkDenial::SchedulerRejected
                    | RecordReadWorkDenial::SchedulerReservationRejected
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
    let ManagedPhysicalIntegrityScrubProgress::WindowInspected(window) = scrub.next_window() else {
        panic!("selected chunk scrub must inspect one bounded window")
    };
    let after = serving.physical_work_counters();
    assert!(
        after.count_under_pressure(
            PhysicalWorkOperationFamily::ArtifactRangeRead,
            PhysicalWorkPressureClass::BackgroundScrub,
            PhysicalWorkCounterStage::Terminal
        ) > before.count_under_pressure(
            PhysicalWorkOperationFamily::ArtifactRangeRead,
            PhysicalWorkPressureClass::BackgroundScrub,
            PhysicalWorkCounterStage::Terminal
        ),
        "selected C5 frame miss must execute under scrub-background pressure"
    );
    for foreground in [
        PhysicalWorkPressureClass::ForegroundPointRead,
        PhysicalWorkPressureClass::ForegroundRangeRead,
        PhysicalWorkPressureClass::ForegroundInteractiveRead,
        PhysicalWorkPressureClass::ForegroundInternalRead,
    ] {
        assert_eq!(
            after.count_under_pressure(
                PhysicalWorkOperationFamily::ArtifactRangeRead,
                foreground,
                PhysicalWorkCounterStage::Terminal
            ),
            before.count_under_pressure(
                PhysicalWorkOperationFamily::ArtifactRangeRead,
                foreground,
                PhysicalWorkCounterStage::Terminal
            ),
            "selected scrub must not substitute foreground C5 reads"
        );
    }
    assert_eq!(window.scope.blob_record_identity().unwrap().0, record);
    assert!(matches!(
        window.outcome,
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Damaged(
            localization
        )) if localization.scope().blob_record_identity().unwrap().0 == record
    ));
    assert_eq!(window.counters.damaged_windows, 1);

    drop(scrub);
    comparison::assert_selected_join(&serving, target, selected_root, record, &offline_selected);
}

fn hex_record(record: worth_store_physical_format::PersistedRecordIdentity) -> String {
    let mut encoded = [0_u8; 24];
    encoded[..16].copy_from_slice(&record.allocation_epoch());
    encoded[16..].copy_from_slice(&record.ordinal().to_le_bytes());
    encoded.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn read_one_byte(
    blobs: &worth_store::physical_runtime::PhysicalBlobFacade<'_>,
    published: PublishedBlobGeneration,
    scope: &worth_store::physical_runtime::AdmittedBlobScope,
    offset: u64,
    limits: BlobReadLimits,
) -> u8 {
    let mut read = blobs.read(published, scope, offset, 1, limits).unwrap();
    let mut byte = [0_u8; 1];
    assert_eq!(read.read_next(&mut byte).unwrap(), 1);
    byte[0]
}

fn flip_middle_chunk_payload_reseal_outer(root: &Path) {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(root.join("families/records/arenas")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "data") {
            continue;
        }
        let media = fs::read(&path).unwrap();
        for inner_offset in 0..media.len().saturating_sub(C11_HEADER) {
            let Some(outer_offset) = inner_offset.checked_sub(C5_HEADER + C5_EXTENT_METADATA)
            else {
                continue;
            };
            if !media[inner_offset..].starts_with(b"WRC11BLB")
                || media[inner_offset + 8] != 2
                || !media[outer_offset..].starts_with(b"WRC5FRM\0")
                || media[outer_offset + 8] != 4
            {
                continue;
            }
            let ordinal = u64::from_le_bytes(
                media[inner_offset + C11_HEADER + 32..inner_offset + C11_HEADER + 40]
                    .try_into()
                    .unwrap(),
            );
            if ordinal != 1 {
                continue;
            }
            let payload_len = u32::from_le_bytes(
                media[outer_offset + 24..outer_offset + 28]
                    .try_into()
                    .unwrap(),
            ) as usize;
            let inner_len = C11_HEADER
                + u32::from_le_bytes(
                    media[inner_offset + 12..inner_offset + 16]
                        .try_into()
                        .unwrap(),
                ) as usize;
            let frame_len = C5_HEADER + payload_len;
            if outer_offset + frame_len <= media.len()
                && inner_len == C11_HEADER + C11_CHUNK_CLAIM + C11_CONTENT_PREFIX + CHUNK_BYTES
                && payload_len
                    > C5_EXTENT_METADATA + C11_HEADER + C11_CHUNK_CLAIM + C11_CONTENT_PREFIX
            {
                candidates.push((path.clone(), outer_offset, frame_len));
            }
        }
    }
    assert_eq!(candidates.len(), 1, "one selected ordinal-one chunk");
    let (path, outer_offset, frame_len) = candidates.pop().unwrap();
    let mut media = fs::read(&path).unwrap();
    let outer = &mut media[outer_offset..outer_offset + frame_len];
    let payload_byte =
        C5_HEADER + C5_EXTENT_METADATA + C11_HEADER + C11_CHUNK_CLAIM + C11_CONTENT_PREFIX;
    outer[payload_byte] ^= 0x01;
    let checksum = crc32c(&outer[..44], &outer[C5_HEADER..]);
    outer[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    fs::write(path, media).unwrap();
}

fn crc32c(prefix: &[u8], payload: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in prefix.iter().chain(payload) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}
