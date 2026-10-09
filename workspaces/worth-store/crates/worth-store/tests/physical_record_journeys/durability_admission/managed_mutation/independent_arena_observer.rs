use std::{
    path::Path,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_REPORT: AtomicU64 = AtomicU64::new(0);
const OBSERVER: &str = "WORTH_C9_OBSERVER_EXECUTABLE";

pub(super) fn observe_arena_in_separate_process(
    root: &Path,
    directory: &Path,
    run: &str,
    scenario: &str,
) -> serde_json::Value {
    let executable = std::env::var_os(OBSERVER)
        .unwrap_or_else(|| panic!("{OBSERVER} must name the independent observer executable"));
    let ordinal = NEXT_REPORT.fetch_add(1, Ordering::Relaxed);
    let report_path = directory.join(format!("c11-{run}-{scenario}-{ordinal}.json"));
    let output = Command::new(executable)
        .args(["observe", "--store-root"])
        .arg(root)
        .arg("--report")
        .arg(&report_path)
        .args([
            "--max-entries",
            "10000",
            "--max-bytes",
            "16777216",
            "--max-open-files",
            "8",
            "--max-depth",
            "12",
            "--max-symlinks",
            "0",
            "--max-elapsed-ms",
            "120000",
            "--max-report-bytes",
            "8388608",
            "--run",
            run,
            "--scenario",
            scenario,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "independent observer failed: status={} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(report_path).unwrap()).unwrap();
    assert_eq!(report["role"], "offline-root-observer");
    assert_eq!(report["completeness"], "complete");
    assert_eq!(report["run"], run);
    assert_eq!(report["scenario"], scenario);
    let observer_pid: u32 = report["process"].as_str().unwrap().parse().unwrap();
    assert_ne!(observer_pid, std::process::id());
    let declared = &report["declared_limits"];
    assert_eq!(declared["entries"], 10000);
    assert_eq!(declared["bytes"], 16 * 1024 * 1024);
    assert_eq!(declared["open_files"], 8);
    assert_eq!(declared["depth"], 12);
    assert_eq!(declared["symlinks"], 0);
    assert_eq!(declared["elapsed_ms"], 120000);
    assert_eq!(declared["report_bytes"], 8 * 1024 * 1024);
    report
}
