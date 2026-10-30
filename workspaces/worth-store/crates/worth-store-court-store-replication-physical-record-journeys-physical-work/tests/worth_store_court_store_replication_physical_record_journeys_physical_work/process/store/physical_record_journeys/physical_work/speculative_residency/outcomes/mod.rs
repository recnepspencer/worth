#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::speculative_residency::outcomes::*;

use {
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::causal_record,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::coordinate,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::initialize_store,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::open_store,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::positioned_reads,
};

#[test]
fn cold_hot_and_mixed_speculation_reconcile_work_media_and_residency_truth() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("speculative-outcomes");
    initialize_store(&root);
    let serving = open_store(&root, 4, 4);
    let residency = serving.certification_physical_residency();
    residency.drain_unpinned_clean_frames();
    let first = coordinate(0);

    let media_before = positioned_reads(&serving);
    let causal_before = serving.physical_work_observer().causal().records().len();
    let counters_before = residency.counters();
    let cold_work = match residency.prefetch(
        worth_store::physical_runtime::PhysicalPrefetchIntent::new(first),
    ) {
        worth_store::physical_runtime::PhysicalPrefetchOutcome::Loaded { coordinate, work } => {
            assert_eq!(coordinate, first);
            work
        }
        outcome => panic!("cold prefetch did not use canonical work: {outcome:?}"),
    };
    assert_eq!(positioned_reads(&serving), media_before + 1);
    assert_eq!(
        serving.physical_work_observer().causal().records().len(),
        causal_before + 1
    );
    let cold_record = causal_record(&serving, cold_work);
    assert_eq!(
        cold_record.operation(),
        worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead
    );
    assert_eq!(
        cold_record.signal_family(),
        worth_store::physical_runtime::PhysicalWorkSignalFamily::ReadFault
    );
    assert!(cold_record.backend_operation().is_some());
    assert_kind_completion(
        counters_before,
        residency.counters(),
        worth_store::physical_runtime::PhysicalSpeculativeWorkKind::Prefetch,
        1,
    );

    let hot_media = serving.media_counters();
    let hot_work = serving.physical_work_counters();
    let hot_causal = serving.physical_work_observer().causal().records().len();
    assert_eq!(
        residency.prefetch(worth_store::physical_runtime::PhysicalPrefetchIntent::new(
            first
        )),
        worth_store::physical_runtime::PhysicalPrefetchOutcome::Hit { coordinate: first }
    );
    assert_eq!(serving.media_counters(), hot_media);
    assert_eq!(serving.physical_work_counters(), hot_work);
    assert_eq!(
        serving.physical_work_observer().causal().records().len(),
        hot_causal
    );

    let second = coordinate(1);
    let read_ahead_before = residency.counters();
    let mixed_media = positioned_reads(&serving);
    let coordinates = [first, second];
    let batch = match residency.read_ahead(
        worth_store::physical_runtime::PhysicalReadAheadIntent::new(&coordinates).unwrap(),
    ) {
        worth_store::physical_runtime::PhysicalReadAheadOutcome::Complete(batch) => batch,
        outcome => panic!("mixed read-ahead did not complete exactly: {outcome:?}"),
    };
    assert_eq!(batch.hits(), 1);
    assert_eq!(batch.coalesced(), 0);
    assert_eq!(batch.loaded(), 1);
    assert_eq!(batch.failed(), 0);
    assert!(matches!(
        batch.frames(),
        [
            worth_store::physical_runtime::PhysicalReadAheadFrameOutcome::Hit { coordinate } ,
            worth_store::physical_runtime::PhysicalReadAheadFrameOutcome::Loaded { coordinate: loaded, .. }
        ] if *coordinate == first && *loaded == second
    ));
    let mixed_work = batch.frames()[1]
        .work()
        .expect("the cold read-ahead frame reports its canonical identity");
    let mixed_record = causal_record(&serving, mixed_work);
    assert_eq!(
        mixed_record.operation(),
        worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead
    );
    assert_eq!(
        mixed_record.signal_family(),
        worth_store::physical_runtime::PhysicalWorkSignalFamily::ReadFault
    );
    assert_eq!(positioned_reads(&serving), mixed_media + 1);
    assert_kind_completion(
        read_ahead_before,
        residency.counters(),
        worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead,
        1,
    );
    assert!(!serving.close().residency().requires_inspection());
}
