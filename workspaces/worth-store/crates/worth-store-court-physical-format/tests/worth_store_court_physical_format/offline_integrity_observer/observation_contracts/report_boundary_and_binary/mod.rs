#[allow(unused_imports)]
use crate::report_boundary_and_binary::*;

#[test]
fn report_destination_fails_closed_for_inside_relative_and_dot_segment_paths() {
    let fixture = clean_store("inside-output");
    let inside = fixture.store.join("reports");
    std::fs::create_dir(&inside).unwrap();
    let direct = inside.join("direct.json");
    assert_inside_denial(&fixture, direct.clone());
    assert!(!direct.exists());

    let dotted = fixture
        .store
        .parent()
        .unwrap()
        .join("reports")
        .join("..")
        .join("store/reports/dotted.json");
    assert_inside_denial(&fixture, dotted.clone());
    assert!(!inside.join("dotted.json").exists());

    let relative = relative_from_current_directory(&inside.join("relative.json"));
    assert!(!relative.is_absolute());
    assert_inside_denial(&fixture, relative);
    assert!(!inside.join("relative.json").exists());
}

#[test]
fn lexical_lookalike_is_external_and_emission_is_create_new() {
    let fixture = clean_store("lookalike-output");
    let lookalike = fixture.store.parent().unwrap().join("store-lookalike");
    std::fs::create_dir(&lookalike).unwrap();
    let output = lookalike.join("report.json");
    let request = request_with_destination(&fixture, output.clone());
    let report = observe_store(&request).unwrap();
    emit_offline_integrity_report(&request, &report).unwrap();
    let bytes = std::fs::read(&output).unwrap();
    assert_eq!(bytes.len() as u64, report.counters().report_bytes());
    let second = emit_offline_integrity_report(&request, &report).unwrap_err();
    assert!(format!("{second:?}").contains("DestinationAlreadyExists"));
}

#[test]
fn existing_report_hard_link_to_store_artifact_is_refused_without_mutation() {
    let fixture = clean_store("hard-linked-output");
    let source = artifact_path(
        &fixture,
        crate::store_offline_integrity_observer_support::Target::Current,
    );
    let before = std::fs::read(&source).unwrap();
    std::fs::hard_link(&source, &fixture.report)
        .expect("same-volume hard-link fixture must be supported");
    let request = request_with_destination(&fixture, fixture.report.clone());
    assert_eq!(
        observe_store(&request),
        Err(worth_store_offline_integrity_observer::OfflineIntegrityObservationDenial::ReportBoundary(
            worth_store_offline_integrity_observer::OfflineIntegrityReportBoundaryDenial::DestinationAlreadyExists,
        ))
    );
    assert_eq!(std::fs::read(&source).unwrap(), before);
    assert_eq!(std::fs::read(&fixture.report).unwrap(), before);
}

#[test]
fn file_report_requires_two_open_handle_slots_before_observation() {
    let fixture = clean_store("report-open-bound");
    let request = worth_store_offline_integrity_observer::OfflineIntegrityObservationRequest::new(
        fixture.store.clone(),
        worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(
            100,
            16 * 1024,
            1,
            8,
            0,
            crate::store_offline_integrity_observer_support::FIXTURE_ELAPSED_MILLISECONDS,
            65_536,
        )
        .unwrap(),
        worth_store_offline_integrity_observer::OfflineIntegrityReportDestination::file(
            fixture.report.clone(),
        )
        .unwrap(),
        request(&fixture).protocol_context().clone(),
    )
    .unwrap();
    assert_eq!(
        observe_store(&request),
        Err(worth_store_offline_integrity_observer::OfflineIntegrityObservationDenial::ReportBoundary(
            worth_store_offline_integrity_observer::OfflineIntegrityReportBoundaryDenial::OpenFileBoundInsufficient,
        ))
    );
    assert!(!fixture.report.exists());
}

#[test]
fn resolvable_symlink_parent_into_store_is_rejected_before_creation() {
    let fixture = clean_store("symlink-output");
    let inside = fixture.store.join("reports");
    std::fs::create_dir(&inside).unwrap();
    let alias = fixture.store.parent().unwrap().join("store-report-alias");
    create_directory_symlink(&inside, &alias)
        .expect("report-boundary symlink fixture must be supported on the admitted test host");
    let aliased_report = alias.join("symlinked.json");
    assert_inside_denial(&fixture, aliased_report);
    assert!(!inside.join("symlinked.json").exists());
}
