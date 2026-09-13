use std::{path::Path, time::Duration};
use worth_store::physical_runtime::{
    ManagedPhysicalIntegrityScrubProgress as Progress,
    ManagedPhysicalIntegrityScrubRequest as Request, PhysicalIntegrityScrubRequestDenial as Denial,
    PhysicalIntegrityScrubTarget as Target, PhysicalOperationAllocationScope,
    PhysicalWorkCounterStage, PhysicalWorkOperationFamily, PhysicalWorkPressureClass,
    ServingPhysicalRuntime,
};
use worth_store_physical_format::{
    PhysicalArtifactReadTarget, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};
use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalByteRange, PhysicalDamageCause,
    PhysicalIntegrityObservationOutcome, PhysicalIntegrityRejection,
};

#[path = "integrity_scrub/in_flight.rs"]
mod in_flight;
#[path = "integrity_scrub/incomplete_source.rs"]
mod incomplete_source;
#[path = "integrity_scrub/pressure.rs"]
mod pressure;
#[path = "integrity_scrub/pressure_world.rs"]
mod pressure_world;
#[path = "integrity_scrub/report.rs"]
mod report;

fn target(serving: &ServingPhysicalRuntime, root: &Path) -> Target {
    let length = std::fs::metadata(root.join("families/records/root-current.selector"))
        .unwrap()
        .len();
    Target::new(
        PhysicalArtifactReadTarget::Record(RecordArtifactFile::CurrentRootSelector),
        PhysicalArtifactScope::current_root_selector(
            serving.store_identity(),
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
            PhysicalByteRange::new(0, length).unwrap(),
        ),
    )
    .unwrap()
}
fn request(serving: &ServingPhysicalRuntime, target: Target) -> Request {
    Request::new(
        serving.store_identity(),
        [target],
        65_536,
        65_536,
        Duration::from_secs(30),
    )
    .unwrap()
}
fn assert_released(serving: &ServingPhysicalRuntime) {
    assert_eq!(
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(PhysicalOperationAllocationScope::Scrub),
        0
    );
    assert_eq!(
        serving.physical_scheduler_capacity().active_reservations(),
        0
    );
}

#[test]
fn managed_scrub_reads_real_bytes_once_and_settles_background_resources() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    let before = serving.physical_work_counters().count_under_pressure(
        PhysicalWorkOperationFamily::ArtifactRangeRead,
        PhysicalWorkPressureClass::BackgroundScrub,
        PhysicalWorkCounterStage::Terminal,
    );
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let progress = handle.next_window();
    let Progress::WindowInspected(observation) = progress else {
        panic!("one real inspection: {progress:?}")
    };
    assert_eq!(observation.ordinal, 0);
    assert!(
        matches!(observation.outcome, PhysicalIntegrityObservationOutcome::Intact(scope) if scope == target.scope())
    );
    assert_eq!(observation.validation_counters.inspected_frames(), 1);
    assert_eq!(
        observation.counters.acquired_bytes,
        target.range().length() as u64
    );
    assert_eq!(
        observation.counters.peak_allocation_bytes,
        target.range().length() as u64
    );
    assert_released(&serving);
    for _ in 0..2 {
        assert!(
            matches!(handle.next_window(), Progress::Completed(counters) if counters.completed_windows == 1)
        );
    }
    assert_eq!(
        serving.physical_work_counters().count_under_pressure(
            PhysicalWorkOperationFamily::ArtifactRangeRead,
            PhysicalWorkPressureClass::BackgroundScrub,
            PhysicalWorkCounterStage::Terminal
        ),
        before + 1
    );
    serving.close();
}

#[test]
fn cancellation_pause_and_cross_handle_resume_do_not_read_or_replay() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::serving_from_initialization(parent.path());
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
    assert!(matches!(first.next_window(), Progress::Paused));
    first.resume(token).unwrap();
    assert!(matches!(first.next_window(), Progress::WindowInspected(_)));
    first.pause();
    assert!(
        first.resume(token).is_err(),
        "a consumed target cannot be replayed"
    );
    first.cancellation().cancel();
    second.cancellation().cancel();
    assert!(
        matches!(first.next_window(), Progress::Cancelled(counters) if counters.completed_windows == 1)
    );
    assert!(
        matches!(second.next_window(), Progress::Cancelled(counters) if counters.acquired_bytes == 0)
    );
    assert_released(&serving);
    serving.close();
}

#[test]
fn close_terminates_retained_handle_and_releases_registration_capacity() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    let mut retained_tokens = Vec::new();
    for _ in 0..20 {
        let mut handle = serving
            .start_physical_integrity_scrub(request(&serving, target))
            .unwrap();
        retained_tokens.push(handle.cancellation());
        handle.cancellation().cancel();
        assert!(matches!(handle.next_window(), Progress::Cancelled(_)));
    }
    let mut outstanding = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let token = outstanding.pause();
    serving.close();
    assert!(outstanding.resume(token).is_err());
    assert!(
        matches!(outstanding.next_window(), Progress::Closed(counters) if counters.acquired_bytes == 0)
    );
}

#[test]
fn damaged_selector_observation_never_repairs_or_revokes_serving_authority() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    let path = parent.path().join("families/records/root-current.selector");
    let mut damaged = std::fs::read(&path).unwrap();
    damaged[0] ^= 1;
    std::fs::write(&path, &damaged).unwrap();
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let Progress::WindowInspected(observation) = handle.next_window() else {
        panic!("actual damaged byte window")
    };
    assert!(
        matches!(observation.outcome, PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Damaged(damage)) if damage.cause() == PhysicalDamageCause::WrongMagic)
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
    let serving = super::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    assert!(matches!(
        Request::new(
            serving.store_identity(),
            [target, target],
            65_536,
            131_072,
            Duration::from_secs(30)
        ),
        Err(Denial::DuplicateOrOverlappingTarget)
    ));
    assert!(matches!(
        Request::new(
            serving.store_identity(),
            [target],
            1,
            65_536,
            Duration::from_secs(30)
        ),
        Err(Denial::WindowBoundExceeded)
    ));
    assert!(matches!(
        Request::new(
            serving.store_identity(),
            [target],
            65_536,
            1,
            Duration::from_secs(30)
        ),
        Err(Denial::TotalByteBoundExceeded)
    ));
    assert_released(&serving);
    serving.close();
}
