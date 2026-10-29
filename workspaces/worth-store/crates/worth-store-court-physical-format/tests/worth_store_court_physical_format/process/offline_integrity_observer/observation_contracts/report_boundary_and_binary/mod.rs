#[allow(unused_imports)]
use crate::report_boundary_and_binary::*;
use crate::support::checksum::refresh_crc32c;

#[test]
fn shipped_observe_binary_emits_the_version_one_report() {
    let fixture = clean_store("binary-output");
    let output = std::process::Command::new(
        worth_store_testkit_executables::physical_store_integrity_observer(),
    )
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
    let output = std::process::Command::new(
        worth_store_testkit_executables::physical_store_integrity_observer(),
    )
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
            let path = artifact_path(
                &fixture,
                crate::store_offline_integrity_observer_support::Target::Current,
            );
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
    let output = std::process::Command::new(
        worth_store_testkit_executables::physical_store_integrity_observer(),
    )
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
