#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::record_read_path::*;

use worth_store_testkit_store::physical_work::record_read_signal_cleanup::await_read_signal_cleanup;
use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};
use {
    worth_store_testkit_store::physical_record_journeys::read_record,
    worth_store_testkit_store::physical_record_journeys::serving_from_open,
};

#[test]
fn cold_read_carries_artifact_proof_so_hot_read_creates_no_physical_work() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let initial = serving_from_initialization(&root);
    let publication = worth_store_testkit_store::durable_publication::root::publish_single(
        &initial,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "physical-work-record-read-path",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([PAYLOAD]).unwrap(),
    );
    let record = publication.settled_members()[0].record_id(0).unwrap();
    initial.close();

    let serving = serving_from_open(&root);
    let limits = worth_store::physical_runtime::RecordReadLimits::new(
        worth_store::physical_runtime::RecordByteLimit::new(PAYLOAD.len() as u32).unwrap(),
    );
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
            worth_store_physical_backend::MediaOperationRole::ReadMetadata,
        ) > 0,
        "cold reads must causally expose their metadata effects"
    );
    assert_eq!(
        media_role_delta(
            media_after_cold,
            media_after_hot,
            worth_store_physical_backend::MediaOperationRole::ReadMetadata,
        ),
        0,
        "resident frames must carry their admitted artifact-length proof"
    );

    assert!(
        family_work_delta(
            work_before,
            work_after_cold,
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ) > 0
    );
    assert!(
        family_work_delta(
            work_before,
            work_after_cold,
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactMetadataRead,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ) > 0
    );
    assert_eq!(
        family_work_delta(
            work_after_cold,
            work_after_hot,
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ),
        0,
        "a hot frame hit must create no source-range work or Signal authority"
    );
    assert_eq!(
        family_work_delta(
            work_after_cold,
            work_after_hot,
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactMetadataRead,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ),
        0,
        "hot reads must not rediscover metadata proof"
    );
    assert_eq!(
        work_delta(
            work_before,
            work_after_cold,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ),
        cold.physical_work_count()
    );
    assert_eq!(
        work_delta(
            work_after_cold,
            work_after_hot,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        ),
        hot.physical_work_count()
    );
    serving.close();
}
