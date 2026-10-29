use {worth_store_physical_integrity::PhysicalIntegrityObservationOutcome, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress as Progress, worth_store_testkit_store::integrity_scrub::assert_released, worth_store_testkit_store::integrity_scrub::request, worth_store_testkit_store::integrity_scrub::target};

#[test]
fn short_actual_read_counts_only_prefix_without_validating_zero_padding() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    let path = parent.path().join("families/records/root-current.selector");
    let prefix_length = u64::from(target.declared_bytes()) / 2;
    std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(prefix_length)
        .unwrap();
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let progress = handle.next_window();
    assert!(
        matches!(progress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(observation)
        if matches!(observation.outcome, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Indeterminate(_)))
        && observation.validation_counters.inspected_frames() == 0 && observation.counters.acquired_bytes == prefix_length
        && observation.quarantine.is_none()),
        "{progress:?}"
    );
    assert!(
        matches!(handle.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Indeterminate(counters) if counters.completed_windows == 1)
    );
    assert_released(&serving);
    assert_eq!(std::fs::metadata(&path).unwrap().len(), prefix_length);
    serving.abort();
}

#[test]
fn absent_expected_source_is_unknown_and_does_not_revoke_serving() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    let path = parent.path().join("families/records/root-current.selector");
    std::fs::remove_file(&path).unwrap();
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let progress = handle.next_window();
    assert!(
        matches!(progress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(observation)
        if matches!(observation.outcome, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Unknown(_)))
        && observation.counters.acquired_bytes == 0 && observation.quarantine.is_none()),
        "{progress:?}"
    );
    assert_released(&serving);
    assert!(serving.observer().acquisition_snapshot().is_ok());
    assert!(!path.exists());
    serving.abort();
}
