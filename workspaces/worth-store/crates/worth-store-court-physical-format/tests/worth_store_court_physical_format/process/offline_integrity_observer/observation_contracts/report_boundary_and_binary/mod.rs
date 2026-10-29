#[allow(unused_imports)] use crate::report_boundary_and_binary::*;
use {std::path::Component, std::path::Path, std::path::PathBuf};
use {std::process::Command};

use {worth_store_offline_integrity_observer::emit_offline_integrity_report, worth_store_offline_integrity_observer::encode_offline_integrity_report, worth_store_offline_integrity_observer::observe_store, worth_store_offline_integrity_observer::OfflineIntegrityObservationDenial, worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits, worth_store_offline_integrity_observer::OfflineIntegrityObservationRequest, worth_store_offline_integrity_observer::OfflineIntegrityProtocolContext, worth_store_offline_integrity_observer::OfflineIntegrityReportBoundaryDenial, worth_store_offline_integrity_observer::OfflineIntegrityReportDestination};

use {crate::store_offline_integrity_observer_support::artifact_path, crate::support::fixtures::clean_store, crate::support::json::parse_json, crate::store_offline_integrity_observer_support::request, crate::support::fixtures::StoreFixture, crate::store_offline_integrity_observer_support::Target};

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
    let source = artifact_path(&fixture, crate::store_offline_integrity_observer_support::Target::Current);
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
        worth_store_offline_integrity_observer::OfflineIntegrityReportDestination::file(fixture.report.clone()).unwrap(),
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

#[test]
fn shipped_observe_binary_emits_the_version_one_report() {
    let fixture = clean_store("binary-output");
    let output = std::process::Command::new(worth_store_testkit_executables::physical_store_integrity_observer())
        .args([
            "observe",
            "--store-root",
            fixture.store.to_str().unwrap(),
            "--report",
            "-",
            "--run",
            "courtroom-run",
            "--scenario",
            "courtroom-scenario",
            "--max-entries",
            "100",
            "--max-bytes",
            "16384",
            "--max-open-files",
            crate::store_offline_integrity_observer_support::FIXTURE_OPEN_FILE_ARGUMENT,
            "--max-depth",
            "8",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            crate::store_offline_integrity_observer_support::FIXTURE_ELAPSED_ARGUMENT,
            "--max-report-bytes",
            "65536",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = String::from_utf8(output.stdout).unwrap();
    let parsed = parse_json(&report);
    assert!(
        report.starts_with("{\"protocol\":\"store.physical.integrity-observation\",\"version\":1")
    );
    assert!(report.contains("\"completeness\":\"complete\""));
    assert_eq!(report.matches("\"posture\":\"intact\"").count(), 4);
    assert!(!report.contains("{,"), "object starts with a separator");
    assert_eq!(parsed.field("version").number(), 1);
    assert_eq!(parsed.field("run").string(), "courtroom-run");
    assert_eq!(parsed.field("scenario").string(), "courtroom-scenario");
}

#[test]
fn shipped_observe_binary_defaults_run_identity_to_its_process() {
    let fixture = clean_store("binary-default-run");
    let output = std::process::Command::new(worth_store_testkit_executables::physical_store_integrity_observer())
        .args([
            "observe",
            "--store-root",
            fixture.store.to_str().unwrap(),
            "--report",
            "-",
            "--max-entries",
            "100",
            "--max-bytes",
            "16384",
            "--max-open-files",
            crate::store_offline_integrity_observer_support::FIXTURE_OPEN_FILE_ARGUMENT,
            "--max-depth",
            "8",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            crate::store_offline_integrity_observer_support::FIXTURE_ELAPSED_ARGUMENT,
            "--max-report-bytes",
            "65536",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = String::from_utf8(output.stdout).unwrap();
    let parsed = parse_json(&report);
    assert_eq!(
        parsed.field("run").string(),
        format!("offline-{}", parsed.field("process").string())
    );
    assert_eq!(parsed.field("scenario").string(), "operator-observe");
}

#[test]
fn clean_damaged_unsupported_and_indeterminate_wires_are_independent_json() {
    for (label, mutate, expected) in [
        ("clean", None, "intact"),
        ("damaged", Some((48_usize, false)), "damaged"),
        ("unsupported", Some((9_usize, true)), "unsupported"),
    ] {
        let fixture = clean_store(label);
        if let Some((offset, refresh)) = mutate {
            let path = artifact_path(&fixture, crate::store_offline_integrity_observer_support::Target::Current);
            let mut bytes = std::fs::read(&path).unwrap();
            bytes[offset] ^= 1;
            if refresh {
                refresh_crc32c(&mut bytes);
            }
            std::fs::write(path, bytes).unwrap();
        }
        let report = observe_store(&request(&fixture)).unwrap();
        assert_wire_posture(&encode_offline_integrity_report(&report).unwrap(), expected);
    }

    let fixture = clean_store("indeterminate-wire");
    let limits = worth_store_offline_integrity_observer::OfflineIntegrityObservationLimits::new(
        100,
        72,
        5,
        8,
        0,
        crate::store_offline_integrity_observer_support::FIXTURE_ELAPSED_MILLISECONDS,
        65_536,
    )
    .unwrap();
    let report = observe_store(&bounded_request(&fixture, limits)).unwrap();
    assert_wire_posture(
        &encode_offline_integrity_report(&report).unwrap(),
        "indeterminate",
    );
}

#[test]
fn shipped_help_has_normative_flags_without_patch_markers() {
    let output = std::process::Command::new(worth_store_testkit_executables::physical_store_integrity_observer())
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for flag in [
        "--store-root",
        "--report",
        "--run",
        "--scenario",
        "--max-entries",
        "--max-bytes",
        "--max-open-files",
        "--max-depth",
        "--max-symlinks",
        "--max-elapsed-ms",
        "--max-report-bytes",
    ] {
        assert!(help.contains(flag), "missing {flag}");
    }
    assert!(!help.lines().any(|line| line.starts_with('+')));
}
