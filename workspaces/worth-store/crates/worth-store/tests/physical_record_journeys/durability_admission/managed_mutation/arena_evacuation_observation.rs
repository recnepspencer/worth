use std::path::Path;
use std::process::Command;

const OBSERVER: &str = "WORTH_C9_OBSERVER_EXECUTABLE";

pub(super) fn assert_evacuated_arena_offline(root: &Path, directory: &Path) {
    let executable = std::env::var_os(OBSERVER)
        .unwrap_or_else(|| panic!("{OBSERVER} must name the Cargo-built independent observer"));
    let report_path = directory.join("evacuated-arena-offline.json");
    let output = Command::new(executable)
        .args(["observe", "--store-root"])
        .arg(root)
        .arg("--report")
        .arg(&report_path)
        .args([
            "--max-entries",
            "100000",
            "--max-bytes",
            "536870912",
            "--max-open-files",
            "8",
            "--max-depth",
            "12",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            "1800000",
            "--max-report-bytes",
            "33554432",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "independent observer failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(report_path).unwrap())
        .expect("independent observer report is valid JSON");
    assert_eq!(report["role"], "offline-root-observer");
    let artifacts = report["artifacts"].as_array().unwrap();
    let first_indeterminate = artifacts
        .iter()
        .find(|artifact| artifact["outcome"]["posture"] == "indeterminate")
        .map(|artifact| (&artifact["path"], &artifact["outcome"]));
    assert_eq!(
        report["completeness"], "complete",
        "offline counters={}, first indeterminate={first_indeterminate:?}",
        report["consumed"],
    );
    let arenas = artifacts
        .iter()
        .filter(|artifact| {
            artifact["path"]
                .as_str()
                .is_some_and(|path| path.contains("/arenas/"))
        })
        .collect::<Vec<_>>();
    assert!(!arenas.is_empty(), "offline report must include arena rows");
    let first_bad = arenas
        .iter()
        .find(|artifact| artifact["outcome"]["posture"] != "intact");
    assert!(
        first_bad.is_none(),
        "evacuated arena routes and accounting must be intact: {first_bad:?}",
    );
}
