//! Executable Rust <-> Lean conformance traces for all five proved `StoreM4`
//! modules (`proofs/lean/store_m4/`):
//! - `PinSafety` and `GCPinSafety` (`pin_trace`)
//! - `ChunkedCoverage` (`chunked_trace`)
//! - `ExtentGenerationSafety` (`extent_generation_trace`)
//! - `ReclamationSafety` (`reclamation_trace`)
//!
//! Each subsystem drives the real compiled Rust production types, emits
//! line-delimited JSON transitions, invokes the built
//! `store_m4_core_conformance` Lean executable on the genuine trace (asserting
//! exit 0), and runs targeted negative drift controls proving non-zero exit
//! whenever a Rust outcome diverges from any proved Lean model.

mod chunked_trace;
mod extent_generation_trace;
mod pin_trace;
mod reclamation_trace;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

#[derive(Default)]
pub(super) struct Trace {
    lines: Vec<String>,
}

impl Trace {
    pub(super) fn push(&mut self, kind: &str, body: String) {
        let step = self.lines.len();
        self.lines
            .push(format!(r#"{{"step":{step},"kind":"{kind}",{body}}}"#));
    }

    fn lines(&self) -> &[String] {
        &self.lines
    }

    fn to_ndjson(&self) -> String {
        let mut out = self.lines.join("\n");
        out.push('\n');
        out
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .expect("worth-store crate sits four levels below repository root")
        .to_path_buf()
}

fn find_lake_executable() -> PathBuf {
    let exe_name = if cfg!(windows) { "lake.exe" } else { "lake" };
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"));
    if let Some(home) = home {
        let candidate = PathBuf::from(home).join(".elan").join("bin").join(exe_name);
        if candidate.is_file() {
            return candidate;
        }
    }
    PathBuf::from(exe_name)
}

fn conformance_binary() -> &'static PathBuf {
    static BIN: OnceLock<PathBuf> = OnceLock::new();
    BIN.get_or_init(|| {
        let lean_dir = repo_root().join("proofs").join("lean").join("store_m4");
        let bin_name = if cfg!(windows) {
            "store_m4_core_conformance.exe"
        } else {
            "store_m4_core_conformance"
        };
        let bin_path = lean_dir
            .join(".lake")
            .join("build")
            .join("bin")
            .join(bin_name);
        let source_path = lean_dir.join("StoreM4CoreConformance.lean");
        let needs_build = match (
            std::fs::metadata(&bin_path),
            std::fs::metadata(&source_path),
        ) {
            (Ok(bin_meta), Ok(src_meta)) => bin_meta
                .modified()
                .ok()
                .zip(src_meta.modified().ok())
                .is_none_or(|(b, s)| b < s),
            _ => true,
        };
        if needs_build {
            let output = Command::new(find_lake_executable())
                .arg("--dir")
                .arg(&lean_dir)
                .arg("build")
                .arg("store_m4_core_conformance")
                .output()
                .expect("failed to invoke `lake build store_m4_core_conformance`");
            assert!(
                output.status.success(),
                "lake build store_m4_core_conformance failed:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        assert!(
            bin_path.is_file(),
            "expected built Lean executable at {}",
            bin_path.display()
        );
        bin_path
    })
}

fn run_conformance_checker(ndjson: &str) -> Output {
    let temp = tempfile::NamedTempFile::new().expect("create trace temp file");
    std::fs::write(temp.path(), ndjson).expect("write conformance trace");
    Command::new(conformance_binary())
        .arg(temp.path())
        .output()
        .expect("execute store_m4_core_conformance")
}

fn collect_full_trace() -> Trace {
    let mut trace = Trace::default();
    pin_trace::record(&mut trace);
    chunked_trace::record(&mut trace);
    extent_generation_trace::record(&mut trace);
    reclamation_trace::record(&mut trace);
    trace
}

fn mutate_first_matching_line(lines: &[String], line_filter: &str, from: &str, to: &str) -> String {
    let mut mutated = lines.to_vec();
    let target = mutated
        .iter_mut()
        .find(|line| line.contains(line_filter) && line.contains(from))
        .unwrap_or_else(|| {
            panic!("expected trace line matching `{line_filter}` containing `{from}`")
        });
    *target = target.replacen(from, to, 1);
    let mut out = mutated.join("\n");
    out.push('\n');
    out
}

#[test]
fn store_m4_real_rust_transitions_conform_to_proved_lean_models() {
    let trace = collect_full_trace();
    assert!(
        trace.lines().len() >= 45,
        "expected comprehensive trace across all five StoreM4 models, got {}",
        trace.lines().len()
    );
    let ndjson = trace.to_ndjson();
    if let Ok(path) = std::env::var("WORTH_STORE_M4_CONFORMANCE_TRACE") {
        std::fs::write(path, &ndjson).expect("write requested conformance trace output");
    }
    let output = run_conformance_checker(&ndjson);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Lean conformance checker rejected genuine Rust trace:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("refine the proved StoreM4 models"),
        "unexpected conformance stdout: {stdout}"
    );
}

#[test]
fn store_m4_conformance_rejects_drifted_transitions_across_all_subsystems() {
    let trace = collect_full_trace();
    let lines = trace.lines();

    let negative_controls: [(&str, &str, &str, &str); 6] = [
        (
            "PinSafety: premature expiry while reader loan held",
            r#""kind":"pin_expire_step","before":[{"id":10,"root":100,"parent":null,"state":"expiring","loans":1"#,
            r#""after":[{"id":10,"root":100,"parent":null,"state":"expiring""#,
            r#""after":[{"id":10,"root":100,"parent":null,"state":"expired""#,
        ),
        (
            "GCPinSafety: accepting reader loan on expiring pin",
            r#""kind":"gc_pin_transition","op":"loan_acquire","id":1"#,
            r#""ok":false"#,
            r#""ok":true"#,
        ),
        (
            "ChunkedCoverage: accepting incomplete chunk partition",
            r#""kind":"chunked_partition","chunk_width":4,"total":8,"submitted":[[0,4]]"#,
            r#""admitted":false"#,
            r#""admitted":true"#,
        ),
        (
            "ExtentGenerationSafety: admitting drain while writeback in flight",
            r#""kind":"extent_drain","range":[9,64,32],"barrier":"completed_retirement""#,
            r#""admitted":false"#,
            r#""admitted":true"#,
        ),
        (
            "ReclamationSafety: admitting DerivedOnly candidate (M8)",
            r#""kind":"reclaim_verdict""#,
            r#""coverage":"derived_only","verdict":"uncovered""#,
            r#""coverage":"derived_only","verdict":"accept""#,
        ),
        (
            "ReclamationSafety: double-releasing published range on resume (M9)",
            r#""kind":"reclaim_resume","barrier":"free_space_published""#,
            r#""release":false"#,
            r#""release":true"#,
        ),
    ];

    for (label, filter, from, to) in negative_controls {
        let tampered = mutate_first_matching_line(lines, filter, from, to);
        let output = run_conformance_checker(&tampered);
        assert!(
            !output.status.success(),
            "negative control `{label}` unexpectedly passed Lean conformance!\nstdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("fails:"),
            "negative control `{label}` exited non-zero without diagnostic: {stderr}"
        );
    }
}
