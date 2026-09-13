//! Explicit measurement-only output. The external runner owns whole-run success.
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use super::measurement_capture::{
    perf_sample_count, perf_warmup_count, PerfCaseContract, PerfMeasurement,
};

const OUTPUT_ENV: &str = "WORTH_SIGNAL_PERF_OUTPUT";
static OUTPUT: OnceLock<Mutex<MeasurementFile>> = OnceLock::new();

pub(super) fn capture_requested() -> bool {
    std::env::var_os(OUTPUT_ENV).is_some()
}

#[allow(
    clippy::assertions_on_constants,
    reason = "runtime opt-in capture must reject incompatible compile-time features"
)]
pub(super) fn validate_capture_posture() {
    if !capture_requested() {
        return;
    }
    assert!(
        !cfg!(debug_assertions),
        "measurement capture requires --release"
    );
    assert!(
        cfg!(feature = "profile-extended")
            && !cfg!(feature = "profile-compact")
            && !cfg!(feature = "profile-standard")
            && !cfg!(feature = "parallel")
            && !cfg!(feature = "test-operation-control"),
        "capture requires serial profile-extended only (plus optional test-peak-allocation)"
    );
    assert!(
        std::env::var_os("WORTH_SIGNAL_UPDATE_PERF_BASELINE").is_none(),
        "measurement capture must not update goldens"
    );
    assert!(
        std::env::var_os("WORTH_SIGNAL_PERF_SAMPLES").is_none()
            && std::env::var_os("WORTH_SIGNAL_PERF_WARMUPS").is_none(),
        "whole-family capture uses unchanged per-policy samples and warmups"
    );
    let args: Vec<_> = std::env::args().collect();
    for required in [
        "tests::performance_profiles::",
        "--ignored",
        "--test-threads=1",
        "--nocapture",
    ] {
        assert!(
            args.iter().any(|arg| arg == required),
            "capture requires {required}"
        );
    }
    let path = output_path();
    if let Some(output) = OUTPUT.get() {
        output
            .lock()
            .expect("measurement output lock")
            .validate_reuse(&path);
    } else {
        validate_output_path(&path, repository()).expect("valid explicit measurement output path");
        assert!(!path.exists(), "measurement output already exists");
    }
}

pub(super) fn record_case(contract: PerfCaseContract<'_>, samples: &[PerfMeasurement]) {
    if !capture_requested() {
        return;
    }
    let path = output_path();
    let output = OUTPUT.get_or_init(|| Mutex::new(MeasurementFile::create(&path)));
    let mut output = output.lock().expect("measurement output lock");
    let peak_probe = cfg!(feature = "test-peak-allocation");
    let record = serde_json::json!({
        "measurement_protocol": super::measurement_protocol::case_protocol(contract.timing_policy),
        "contract": contract,
        "probe": if peak_probe { "peak" } else { "ordinary" },
        "sample_count": perf_sample_count(contract.timing_policy),
        "warmup_count": perf_warmup_count(contract.timing_policy),
        "workload_warmup": workload_warmup(contract.suite, contract.profile),
        "relative_budgets": super::regression_budgets::relative_budgets(contract, peak_probe),
        "samples": samples,
    });
    output.write(&path, &record);
}

fn workload_warmup(suite: &str, profile: &str) -> &'static str {
    match (suite, profile) {
        ("chain_10k_bootstrap", "balanced") => "once: build-plan-execute prime",
        ("fintech_mixed_fanout", "operational" | "development" | "forensic") => {
            "once per profile: read-mutate-read prime"
        }
        ("topology_rewiring_churn", "balanced") => "once: churn-rewire prime",
        ("topology_rewiring_rotating_window", "balanced") => "once: window-rewire prime",
        _ => "none",
    }
}

fn output_path() -> PathBuf {
    PathBuf::from(std::env::var_os(OUTPUT_ENV).expect("missing explicit output"))
}

fn repository() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}

struct MeasurementFile {
    path: PathBuf,
    file: File,
}

impl MeasurementFile {
    fn create(requested: &Path) -> Self {
        let path = validate_output_path(requested, repository()).expect("valid measurement path");
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap_or_else(|err| {
                panic!("create new measurement output {}: {err}", path.display())
            });
        Self { path, file }
    }

    fn validate_reuse(&self, requested: &Path) {
        let path = validate_output_path(requested, repository()).expect("valid measurement path");
        assert_eq!(
            self.path, path,
            "measurement output path changed during run"
        );
    }

    fn write(&mut self, requested: &Path, record: &serde_json::Value) {
        self.validate_reuse(requested);
        serde_json::to_writer(&mut self.file, record).expect("write measurement record");
        writeln!(&mut self.file).expect("finish measurement record");
        self.file.flush().expect("flush measurement record");
    }
}

fn validate_output_path(path: &Path, repository: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() || path.file_name().is_none() {
        return Err("output must be an absolute file path".into());
    }
    let parent = path
        .parent()
        .ok_or("output needs a parent")?
        .canonicalize()
        .map_err(|err| format!("output parent must already exist: {err}"))?;
    let root = repository.canonicalize().map_err(|err| err.to_string())?;
    if parent.starts_with(root) || path.file_name().unwrap() == "performance_baseline.json" {
        return Err("measurement output must be outside source and cannot be a golden".into());
    }
    Ok(parent.join(path.file_name().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::{validate_output_path, workload_warmup, MeasurementFile};
    use std::path::Path;

    #[test]
    fn measurement_output_cannot_target_source_existing_or_relative_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(validate_output_path(&root.join("Cargo.toml"), root).is_err());
        assert!(validate_output_path(&root.join("not-a-golden.json"), root).is_err());
        assert!(validate_output_path(Path::new("relative.json"), root).is_err());
        assert!(validate_output_path(
            &std::env::temp_dir().join("performance_baseline.json"),
            root
        )
        .is_err());
    }

    #[test]
    fn admitted_file_accepts_two_cases_but_cannot_be_reopened_or_redirected() {
        let path = std::env::temp_dir().join(format!(
            "signal-perf-output-{}-{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut output = MeasurementFile::create(&path);
        for case in ["first", "second"] {
            output.write(&path, &serde_json::json!({"case": case}));
        }
        assert!(std::panic::catch_unwind(|| MeasurementFile::create(&path)).is_err());
        assert!(
            std::panic::catch_unwind(|| output.validate_reuse(&path.with_extension("other")))
                .is_err()
        );
        drop(output);
        let raw = std::fs::read_to_string(&path).unwrap();
        let cases: Vec<serde_json::Value> = raw
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(
            cases,
            vec![
                serde_json::json!({"case": "first"}),
                serde_json::json!({"case": "second"})
            ]
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn workload_warmup_posture_names_each_once_family() {
        assert_eq!(
            workload_warmup("chain_10k_bootstrap", "balanced"),
            "once: build-plan-execute prime"
        );
        for profile in ["operational", "development", "forensic"] {
            assert_eq!(
                workload_warmup("fintech_mixed_fanout", profile),
                "once per profile: read-mutate-read prime"
            );
        }
        assert_eq!(
            workload_warmup("topology_rewiring_churn", "balanced"),
            "once: churn-rewire prime"
        );
        assert_eq!(
            workload_warmup("topology_rewiring_rotating_window", "balanced"),
            "once: window-rewire prime"
        );
        assert_eq!(
            workload_warmup("suppression_wide_fanout", "balanced"),
            "none"
        );
    }
}
