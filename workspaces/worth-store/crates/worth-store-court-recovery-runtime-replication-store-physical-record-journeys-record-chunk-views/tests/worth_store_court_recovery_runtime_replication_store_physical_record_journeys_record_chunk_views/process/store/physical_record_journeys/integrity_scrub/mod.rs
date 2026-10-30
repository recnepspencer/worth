#[allow(unused_imports)]
use worth_store_testkit_store::integrity_scrub::*;

#[test]
fn managed_scrub_reads_real_bytes_once_and_settles_background_resources() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(
        parent.path(),
    );
    let target = target(&serving, parent.path());
    let before = serving.physical_work_counters().count_under_pressure(
        worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
        worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundScrub,
        worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
    );
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let progress = handle.next_window();
    let worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(
        observation,
    ) = progress
    else {
        panic!("one real inspection: {progress:?}")
    };
    assert_eq!(observation.ordinal, 0);
    assert!(
        matches!(observation.outcome, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Intact(scope) if scope == target.scope())
    );
    assert_eq!(observation.validation_counters.inspected_frames(), 1);
    assert_eq!(
        observation.counters.acquired_bytes,
        target.declared_bytes() as u64
    );
    assert_eq!(
        observation.counters.peak_allocation_bytes,
        target.declared_bytes() as u64
    );
    assert_released(&serving);
    for _ in 0..2 {
        assert!(
            matches!(handle.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Completed(counters) if counters.completed_windows == 1)
        );
    }
    assert_eq!(
        serving.physical_work_counters().count_under_pressure(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeRead,
            worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundScrub,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal
        ),
        before + 1
    );
    serving.close();
}

#[test]
fn cancellation_pause_and_cross_handle_resume_do_not_read_or_replay() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(
        parent.path(),
    );
    let target = target(&serving, parent.path());
    let mut first = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let mut second = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let token = first.pause();
    second.pause();
    assert!(second.resume(token).is_err());
    assert!(matches!(
        first.next_window(),
        worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Paused
    ));
    first.resume(token).unwrap();
    assert!(matches!(
        first.next_window(),
        worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(_)
    ));
    first.pause();
    assert!(
        first.resume(token).is_err(),
        "a consumed target cannot be replayed"
    );
    first.cancellation().cancel();
    second.cancellation().cancel();
    assert!(
        matches!(first.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Cancelled(counters) if counters.completed_windows == 1)
    );
    assert!(
        matches!(second.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Cancelled(counters) if counters.acquired_bytes == 0)
    );
    assert_released(&serving);
    serving.close();
}

#[test]
fn close_terminates_retained_handle_and_releases_registration_capacity() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(
        parent.path(),
    );
    let target = target(&serving, parent.path());
    let mut retained_tokens = Vec::new();
    for _ in 0..20 {
        let mut handle = serving
            .start_physical_integrity_scrub(request(&serving, target))
            .unwrap();
        retained_tokens.push(handle.cancellation());
        handle.cancellation().cancel();
        assert!(matches!(
            handle.next_window(),
            worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Cancelled(_)
        ));
    }
    let mut outstanding = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let token = outstanding.pause();
    serving.close();
    assert!(outstanding.resume(token).is_err());
    assert!(
        matches!(outstanding.next_window(), worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::Closed(counters) if counters.acquired_bytes == 0)
    );
}

#[test]
fn damaged_selector_observation_never_repairs_or_revokes_serving_authority() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(
        parent.path(),
    );
    let target = target(&serving, parent.path());
    let path = parent.path().join("families/records/root-current.selector");
    let mut damaged = std::fs::read(&path).unwrap();
    damaged[0] ^= 1;
    std::fs::write(&path, &damaged).unwrap();
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(
        observation,
    ) = handle.next_window()
    else {
        panic!("actual damaged byte window")
    };
    assert!(
        matches!(observation.outcome, worth_store_physical_integrity::PhysicalIntegrityObservationOutcome::Rejected(worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(damage)) if damage.cause() == worth_store_physical_integrity::PhysicalDamageCause::WrongMagic)
    );
    assert!(observation.quarantine.is_some());
    assert_eq!(
        std::fs::read(&path).unwrap(),
        damaged,
        "diagnostics have no repair effect"
    );
    assert_released(&serving);
    assert!(serving.observer().acquisition_snapshot().is_ok());
    serving.abort();
}

#[test]
fn explicit_scope_rejects_duplicate_and_byte_limit_before_start() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(
        parent.path(),
    );
    let target = target(&serving, parent.path());
    assert!(matches!(
        worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
            serving.store_identity(),
            [target, target],
            65_536,
            131_072,
            std::time::Duration::from_secs(30)
        ),
        Err(worth_store::physical_runtime::PhysicalIntegrityScrubRequestDenial::DuplicateOrOverlappingTarget)
    ));
    assert!(matches!(
        worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
            serving.store_identity(),
            [target],
            1,
            65_536,
            std::time::Duration::from_secs(30)
        ),
        Err(
            worth_store::physical_runtime::PhysicalIntegrityScrubRequestDenial::WindowBoundExceeded
        )
    ));
    assert!(matches!(
        worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
            serving.store_identity(),
            [target],
            65_536,
            1,
            std::time::Duration::from_secs(30)
        ),
        Err(worth_store::physical_runtime::PhysicalIntegrityScrubRequestDenial::TotalByteBoundExceeded)
    ));
    assert_released(&serving);
    serving.close();
}
#[path = "incomplete_source/mod.rs"]
pub(crate) mod incomplete_source;
