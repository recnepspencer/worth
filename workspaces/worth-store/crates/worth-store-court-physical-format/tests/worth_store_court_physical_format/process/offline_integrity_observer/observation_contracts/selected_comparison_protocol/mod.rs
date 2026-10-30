#[allow(unused_imports)]
use crate::selected_comparison_protocol::*;

#[test]
fn observe_selected_cli_emits_from_a_distinct_real_process() {
    let fixture = clean_store("selected-comparison-child");
    let output = std::process::Command::new(
        worth_store_testkit_executables::physical_store_integrity_observer(),
    )
    .args([
        "observe-selected",
        "--store-root",
        fixture.store.to_str().unwrap(),
        "--report",
        fixture.report.to_str().unwrap(),
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
        "--run",
        "child-run",
        "--scenario",
        "child-selected-scenario",
    ])
    .output()
    .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&std::fs::read(&fixture.report).unwrap()).unwrap();
    assert_eq!(report["version"], 2);
    assert_eq!(report["role"], "offline-root-observer");
    assert_eq!(report["executable"], "physical_store_integrity_observer");
    assert_ne!(report["process"], std::process::id().to_string());
    assert_eq!(report["scenario"], "child-selected-scenario");
}
