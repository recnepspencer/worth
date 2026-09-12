use worth_store::physical_runtime::{
    PhysicalWorkCounterSnapshot, PhysicalWorkCounterStage, PhysicalWorkOperationFamily,
    RecordAppendBatch, RecordByteLimit, RecordReadLimits, RecordReadObservation,
};
use worth_store_physical_backend::MediaOperationRole;

use super::super::{durable_publication, read_record, serving_from_open};
use super::record_read_signal_cleanup::await_read_signal_cleanup;
use super::{configuration, serving_from_initialization};

const PAYLOAD: &[u8] = b"canonical cold and hot record read";

#[test]
fn cold_read_carries_artifact_proof_so_hot_read_creates_no_physical_work() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let initial = serving_from_initialization(&root);
    let publication = durable_publication::publish_single(
        &initial,
        placement,
        durable_publication::certification_material("physical-work-record-read-path", 1),
        RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    initial.close();

    let serving = serving_from_open(&root);
    let limits = RecordReadLimits::new(RecordByteLimit::new(PAYLOAD.len() as u32).unwrap());
    let media_before = serving.media_counters();
    let residency_before = serving.residency_observation().counters();
    let work_before = serving.physical_work_counters();
    let invalidations_before = serving
        .physical_signal_observation()
        .unwrap()
        .aspect_invalidation_count();
    let settled_before = serving.physical_work_observer().causal().records().len();
    let (cold_bytes, cold) = read_record(
        serving
            .records()
            .expect("read protection admission")
            .open(record, limits)
            .unwrap(),
        PAYLOAD.len(),
    );
    await_read_signal_cleanup(&serving);
    let media_after_cold = serving.media_counters();
    let residency_after_cold = serving.residency_observation().counters();
    let work_after_cold = serving.physical_work_counters();
    let settled_after_cold = serving.physical_work_observer().causal().records();

    let (hot_bytes, hot) = read_record(
        serving
            .records()
            .expect("read protection admission")
            .open(record, limits)
            .unwrap(),
        PAYLOAD.len(),
    );
    await_read_signal_cleanup(&serving);
    let media_after_hot = serving.media_counters();
    let residency_after_hot = serving.residency_observation().counters();
    let work_after_hot = serving.physical_work_counters();
    let settled_after_hot = serving.physical_work_observer().causal().records();
    let invalidations_after = serving
        .physical_signal_observation()
        .unwrap()
        .aspect_invalidation_count();

    assert_eq!(cold_bytes, PAYLOAD);
    assert_eq!(hot_bytes, PAYLOAD);
    assert_same_semantic_observation(cold, hot);
    assert!(cold.physical_work_count() > 0);
    assert_eq!(hot.physical_work_count(), 0);
    assert_eq!(hot.first_physical_work(), None);
    assert_eq!(hot.last_physical_work(), None);
    assert_eq!(
        invalidations_after, invalidations_before,
        "successful cold and hot reads must not manufacture dependency changes"
    );

    let cold_reads = media_delta(media_before, media_after_cold);
    let hot_reads = media_delta(media_after_cold, media_after_hot);
    assert!(
        cold_reads > 0,
        "the cold path must fault through real media"
    );
    assert_eq!(hot_reads, 0, "the resident path must skip positioned media");
    assert!(
        residency_after_cold.faults() > residency_before.faults(),
        "the cold path must be observed as a residency fault"
    );
    assert!(
        residency_after_hot.hits() > residency_after_cold.hits(),
        "the hot path must be observed as a residency hit"
    );

    assert!(settled_after_cold[settled_before..]
        .iter()
        .any(|record| record.backend_operation().is_some()));
    assert!(settled_after_cold[settled_before..]
        .iter()
        .all(|record| record.backend_operation().is_some()));
    assert_eq!(
        settled_after_hot.len(),
        settled_after_cold.len(),
        "the hot read must create no physical work identity"
    );
    assert!(settled_after_hot.iter().all(|record| {
        record.derived_completion()
            == Some(worth_store::physical_runtime::PhysicalSignalSettlementOutcome::Committed)
    }));
    assert!(
        media_role_delta(
            media_before,
            media_after_cold,
            MediaOperationRole::ReadMetadata,
        ) > 0,
        "cold reads must causally expose their metadata effects"
    );
    assert_eq!(
        media_role_delta(
            media_after_cold,
            media_after_hot,
            MediaOperationRole::ReadMetadata,
        ),
        0,
        "resident frames must carry their admitted artifact-length proof"
    );

    assert!(
        family_work_delta(
            work_before,
            work_after_cold,
            PhysicalWorkOperationFamily::ArtifactRangeRead,
            PhysicalWorkCounterStage::Terminal,
        ) > 0
    );
    assert!(
        family_work_delta(
            work_before,
            work_after_cold,
            PhysicalWorkOperationFamily::ArtifactMetadataRead,
            PhysicalWorkCounterStage::Terminal,
        ) > 0
    );
    assert_eq!(
        family_work_delta(
            work_after_cold,
            work_after_hot,
            PhysicalWorkOperationFamily::ArtifactRangeRead,
            PhysicalWorkCounterStage::Terminal,
        ),
        0,
        "a hot frame hit must create no source-range work or Signal authority"
    );
    assert_eq!(
        family_work_delta(
            work_after_cold,
            work_after_hot,
            PhysicalWorkOperationFamily::ArtifactMetadataRead,
            PhysicalWorkCounterStage::Terminal,
        ),
        0,
        "hot reads must not rediscover metadata proof"
    );
    assert_eq!(
        work_delta(
            work_before,
            work_after_cold,
            PhysicalWorkCounterStage::Terminal,
        ),
        cold.physical_work_count()
    );
    assert_eq!(
        work_delta(
            work_after_cold,
            work_after_hot,
            PhysicalWorkCounterStage::Terminal,
        ),
        hot.physical_work_count()
    );
    serving.close();
}

fn assert_same_semantic_observation(left: RecordReadObservation, right: RecordReadObservation) {
    assert_eq!(left.touched_segments(), right.touched_segments());
    assert_eq!(left.touched_pages(), right.touched_pages());
    assert_eq!(left.touched_extents(), right.touched_extents());
    assert_eq!(left.payload_bytes(), right.payload_bytes());
    assert_eq!(left.bytes_requested(), right.bytes_requested());
    assert_eq!(left.transfer_count(), right.transfer_count());
    assert_eq!(left.peak_transfer_width(), right.peak_transfer_width());
    assert_eq!(left.explicit_copy_count(), right.explicit_copy_count());
    assert_eq!(left.copied_bytes(), right.copied_bytes());
    assert_eq!(left.generation_checks(), right.generation_checks());
    assert_eq!(left.generation_rejections(), right.generation_rejections());
    assert_eq!(left.peak_scratch_bytes(), right.peak_scratch_bytes());
    assert_eq!(left.manifest_blocks(), right.manifest_blocks());
    assert_eq!(left.manifest_comparisons(), right.manifest_comparisons());
    assert_eq!(left.manifest_bytes(), right.manifest_bytes());
}

fn media_delta(
    before: worth_store_physical_backend::MediaCounterSnapshot,
    after: worth_store_physical_backend::MediaCounterSnapshot,
) -> u64 {
    media_role_delta(before, after, MediaOperationRole::PositionedRead)
}

fn media_role_delta(
    before: worth_store_physical_backend::MediaCounterSnapshot,
    after: worth_store_physical_backend::MediaCounterSnapshot,
    role: MediaOperationRole,
) -> u64 {
    after.attempts_for(role) - before.attempts_for(role)
}

fn work_delta(
    before: PhysicalWorkCounterSnapshot,
    after: PhysicalWorkCounterSnapshot,
    stage: PhysicalWorkCounterStage,
) -> u64 {
    [
        PhysicalWorkOperationFamily::ArtifactMetadataRead,
        PhysicalWorkOperationFamily::ArtifactRangeRead,
    ]
    .into_iter()
    .map(|family| after.count(family, stage) - before.count(family, stage))
    .sum()
}

fn family_work_delta(
    before: PhysicalWorkCounterSnapshot,
    after: PhysicalWorkCounterSnapshot,
    family: PhysicalWorkOperationFamily,
    stage: PhysicalWorkCounterStage,
) -> u64 {
    after.count(family, stage) - before.count(family, stage)
}
