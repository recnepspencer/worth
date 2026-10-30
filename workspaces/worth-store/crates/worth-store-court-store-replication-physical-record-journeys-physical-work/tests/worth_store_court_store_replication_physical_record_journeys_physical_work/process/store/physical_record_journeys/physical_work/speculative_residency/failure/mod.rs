use {
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::coordinate,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::initialize_store,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::open_store,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::positioned_reads,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::residency_policy,
};

#[test]
fn second_read_ahead_fault_reports_partial_and_releases_every_grant() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("speculative-partial");
    initialize_store(&root);
    let calibration = open_store(&root, 4, 4);
    let bootstrap_reads = positioned_reads(&calibration);
    assert!(!calibration.close().residency().requires_inspection());
    let (format, _, _) = worth_store_testkit_store::physical_record_journeys::configuration();
    let serving =
        worth_store_testkit_store::physical_record_journeys::physical_work::fault_fixture::serving_from_open_with_positioned_read_fault_and_policy(
            &root,
            bootstrap_reads + 2,
            worth_store_physical_backend::MediaFaultDirective::FailBefore {
                kind: std::io::ErrorKind::Other,
                raw_os_error: None,
            },
            residency_policy(format, 4, 4),
        );
    let residency = serving.certification_physical_residency();
    residency.drain_unpinned_clean_frames();
    let coordinates = [coordinate(0), coordinate(1)];
    let media_before = positioned_reads(&serving);
    let counters_before = residency.counters();

    let batch = match residency.read_ahead(
        worth_store::physical_runtime::PhysicalReadAheadIntent::new(&coordinates).unwrap(),
    ) {
        worth_store::physical_runtime::PhysicalReadAheadOutcome::Partial(batch) => batch,
        outcome => panic!("second-frame backend failure was not partial: {outcome:?}"),
    };
    assert_eq!((batch.loaded(), batch.failed()), (1, 1));
    assert!(matches!(
        batch.frames()[0],
        worth_store::physical_runtime::PhysicalReadAheadFrameOutcome::Loaded { coordinate, .. }
            if coordinate == coordinates[0]
    ));
    assert!(matches!(
        batch.frames()[1].failure(),
        Some(
            worth_store::physical_runtime::PhysicalSpeculativeReadFailure::Frame(
                worth_store::physical_runtime::PhysicalFrameReadFailure::FaultTerminated {
                    cause: worth_store::physical_runtime::PhysicalFrameFaultCause::PhysicalWork(
                        worth_store::physical_runtime::PhysicalFrameWorkFailure::Backend(_)
                    ),
                    ..
                }
            )
        )
    ));
    assert_eq!(positioned_reads(&serving), media_before + 2);
    let after = residency.counters();
    assert_eq!(
        after.speculative_attempts(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead
        ),
        counters_before.speculative_attempts(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead
        ) + 1
    );
    assert_eq!(
        after.speculative_admissions(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead
        ),
        counters_before.speculative_admissions(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead
        ) + 1
    );
    assert_eq!(
        after.speculative_completions(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead
        ),
        counters_before.speculative_completions(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead
        ) + 1
    );
    assert_eq!(
        after.active_speculative_frames(
            worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead
        ),
        0
    );
    assert_eq!(after.active_operation_bytes(), 0);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedRead),
        media_before + 2
    );
    assert!(!serving.close().residency().requires_inspection());
}
