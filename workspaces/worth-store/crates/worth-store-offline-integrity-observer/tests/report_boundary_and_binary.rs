use std::path::{Component, Path, PathBuf};
use std::process::Command;

use worth_store_offline_integrity_observer::{
    emit_offline_integrity_report, encode_offline_integrity_report, observe_store,
    OfflineIntegrityObservationDenial, OfflineIntegrityObservationLimits,
    OfflineIntegrityObservationRequest, OfflineIntegrityProtocolContext,
    OfflineIntegrityReportBoundaryDenial, OfflineIntegrityReportDestination,
};

use crate::support::{
    artifact_path, clean_store, parse_json, refresh_crc32c, request, StoreFixture, Target,
};

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
    let source = artifact_path(&fixture, Target::Current);
    let before = std::fs::read(&source).unwrap();
    std::fs::hard_link(&source, &fixture.report)
        .expect("same-volume hard-link fixture must be supported");
    let request = request_with_destination(&fixture, fixture.report.clone());
    assert_eq!(
        observe_store(&request),
        Err(OfflineIntegrityObservationDenial::ReportBoundary(
            OfflineIntegrityReportBoundaryDenial::DestinationAlreadyExists,
        ))
    );
    assert_eq!(std::fs::read(&source).unwrap(), before);
    assert_eq!(std::fs::read(&fixture.report).unwrap(), before);
}

#[test]
fn file_report_requires_two_open_handle_slots_before_observation() {
    let fixture = clean_store("report-open-bound");
    let request = OfflineIntegrityObservationRequest::new(
        fixture.store.clone(),
        OfflineIntegrityObservationLimits::new(100, 16 * 1024, 1, 8, 0, 5_000, 65_536).unwrap(),
        OfflineIntegrityReportDestination::file(fixture.report.clone()).unwrap(),
        request(&fixture).protocol_context().clone(),
    )
    .unwrap();
    assert_eq!(
        observe_store(&request),
        Err(OfflineIntegrityObservationDenial::ReportBoundary(
            OfflineIntegrityReportBoundaryDenial::OpenFileBoundInsufficient,
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
    let output = Command::new(env!("CARGO_BIN_EXE_physical_store_integrity_observer"))
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
            "5",
            "--max-depth",
            "8",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            "5000",
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
    let output = Command::new(env!("CARGO_BIN_EXE_physical_store_integrity_observer"))
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
            "5",
            "--max-depth",
            "8",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            "5000",
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
            let path = artifact_path(&fixture, Target::Current);
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
    let limits = OfflineIntegrityObservationLimits::new(100, 72, 5, 8, 0, 5_000, 65_536).unwrap();
    let report = observe_store(&bounded_request(&fixture, limits)).unwrap();
    assert_wire_posture(
        &encode_offline_integrity_report(&report).unwrap(),
        "indeterminate",
    );
}

#[test]
fn shipped_help_has_normative_flags_without_patch_markers() {
    let output = Command::new(env!("CARGO_BIN_EXE_physical_store_integrity_observer"))
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

fn assert_wire_posture(wire: &str, expected: &str) {
    let parsed = parse_json(wire);
    assert_eq!(
        parsed.field("protocol").string(),
        "store.physical.integrity-observation"
    );
    assert!(parsed
        .field("artifacts")
        .array()
        .iter()
        .any(|artifact| artifact.field("outcome").field("posture").string() == expected));
    assert_eq!(
        parsed.field("consumed").field("report_bytes").number(),
        wire.len() as u64
    );
}

fn assert_inside_denial(fixture: &StoreFixture, destination: PathBuf) {
    let request = request_with_destination(fixture, destination);
    assert_eq!(
        observe_store(&request),
        Err(OfflineIntegrityObservationDenial::ReportBoundary(
            OfflineIntegrityReportBoundaryDenial::DestinationInsideStoreRoot,
        ))
    );
}

fn request_with_destination(
    fixture: &StoreFixture,
    destination: PathBuf,
) -> OfflineIntegrityObservationRequest {
    let template = request(fixture);
    OfflineIntegrityObservationRequest::new(
        fixture.store.clone(),
        template.limits(),
        OfflineIntegrityReportDestination::file(destination).unwrap(),
        OfflineIntegrityProtocolContext::new(
            "fixture-observer",
            "process-1",
            "run-1",
            "scenario-1",
        )
        .unwrap(),
    )
    .unwrap()
}

fn bounded_request(
    fixture: &StoreFixture,
    limits: OfflineIntegrityObservationLimits,
) -> OfflineIntegrityObservationRequest {
    let template = request(fixture);
    OfflineIntegrityObservationRequest::new(
        fixture.store.clone(),
        limits,
        OfflineIntegrityReportDestination::standard_output(),
        template.protocol_context().clone(),
    )
    .unwrap()
}

fn relative_from_current_directory(target: &Path) -> PathBuf {
    let current = std::env::current_dir().unwrap();
    let current_components: Vec<_> = current.components().collect();
    let target_components: Vec<_> = target.components().collect();
    let shared = current_components
        .iter()
        .zip(&target_components)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for component in &current_components[shared..] {
        if matches!(component, Component::Normal(_)) {
            relative.push("..");
        }
    }
    for component in &target_components[shared..] {
        relative.push(component.as_os_str());
    }
    relative
}

#[cfg(unix)]
fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_directory_symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}
