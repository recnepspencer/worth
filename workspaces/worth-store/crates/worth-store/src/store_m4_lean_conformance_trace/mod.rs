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

fn find_lake_executable() -> Option<PathBuf> {
    let exe_name = if cfg!(windows) { "lake.exe" } else { "lake" };
    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"));
    if let Some(home) = home {
        let candidate = PathBuf::from(home).join(".elan").join("bin").join(exe_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(exe_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn require_lean_conformance() -> bool {
    std::env::var("WORTH_REQUIRE_LEAN_CONFORMANCE")
        .ok()
        .is_some_and(|v| !v.is_empty() && v != "0")
}

fn conformance_binary() -> Option<&'static PathBuf> {
    static BIN: OnceLock<Option<PathBuf>> = OnceLock::new();
    BIN.get_or_init(|| {
        let Some(lake) = find_lake_executable() else {
            assert!(
                !require_lean_conformance(),
                "WORTH_REQUIRE_LEAN_CONFORMANCE is set, but `lake` was not found in ~/.elan/bin or PATH"
            );
            eprintln!(
                "store_m4_lean_conformance: skipping Lean binary invocation (`lake` not installed)"
            );
            return None;
        };
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
        // Always invoke `lake build` once per process with `current_dir(&lean_dir)`
        // so Elan selects `proofs/lean/store_m4/lean-toolchain` and Lake's own
        // dependency graph rebuilds whenever any imported `.lean`, `lakefile.toml`,
        // or `lean-toolchain` changed.
        let output = Command::new(&lake)
            .current_dir(&lean_dir)
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
        assert!(
            bin_path.is_file(),
            "expected built Lean executable at {}",
            bin_path.display()
        );
        Some(bin_path)
    })
    .as_ref()
}

fn run_conformance_checker(bin: &Path, ndjson: &str) -> Output {
    let temp = tempfile::NamedTempFile::new().expect("create trace temp file");
    std::fs::write(temp.path(), ndjson).expect("write conformance trace");
    Command::new(bin)
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

fn mutate_first_matching_line(
    lines: &[String],
    line_filter: &str,
    replacements: &[(&str, &str)],
) -> (usize, String) {
    let mut mutated = lines.to_vec();
    let (step_idx, target) = mutated
        .iter_mut()
        .enumerate()
        .find(|(_, line)| {
            line.contains(line_filter) && replacements.iter().all(|(from, _)| line.contains(from))
        })
        .unwrap_or_else(|| {
            panic!("expected trace line matching `{line_filter}` with {replacements:?}")
        });
    assert!(
        target.contains(&format!(r#""step":{step_idx},"#)),
        "expected step index {step_idx} in line: {target}"
    );
    for &(from, to) in replacements {
        let updated = target.replacen(from, to, 1);
        assert_ne!(
            updated, *target,
            "replacement `{from}` -> `{to}` did not modify step {step_idx}"
        );
        *target = updated;
    }
    let mut out = mutated.join("\n");
    out.push('\n');
    (step_idx, out)
}

#[test]
fn store_m4_real_rust_transitions_conform_to_proved_lean_models() {
    let trace = collect_full_trace();
    assert!(
        trace.lines().len() >= 48,
        "expected comprehensive trace across all five StoreM4 models, got {}",
        trace.lines().len()
    );
    let ndjson = trace.to_ndjson();
    if let Ok(path) = std::env::var("WORTH_STORE_M4_CONFORMANCE_TRACE") {
        std::fs::write(path, &ndjson).expect("write requested conformance trace output");
    }
    let Some(bin) = conformance_binary() else {
        return;
    };
    let output = run_conformance_checker(bin, &ndjson);
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
    let Some(bin) = conformance_binary() else {
        return;
    };
    let trace = collect_full_trace();
    let lines = trace.lines();

    let negative_controls: [(&str, &str, &[(&str, &str)], &str); 7] = [
        (
            "PinSafety: premature expiry while reader loan held",
            r#""kind":"pin_expire_step","before":[{"id":10,"root":100,"parent":null,"state":"expiring","loans":1"#,
            &[
                (
                    r#""after":[{"id":10,"root":100,"parent":null,"state":"expiring""#,
                    r#""after":[{"id":10,"root":100,"parent":null,"state":"expired""#,
                ),
                (r#""probes":[[100,true]"#, r#""probes":[[100,false]"#),
            ],
            "PinSafety.expireById mismatch:",
        ),
        (
            "GCPinSafety: accepting reader loan on expiring pin",
            r#""kind":"gc_pin_transition","op":"loan_acquire","id":1"#,
            &[
                (r#""ok":false"#, r#""ok":true"#),
                (
                    r#""after":[{"id":1,"root":10,"parent":null,"state":"expiring","loans":0"#,
                    r#""after":[{"id":1,"root":10,"parent":null,"state":"expiring","loans":1"#,
                ),
            ],
            "Rust accepted loan_acquire on pin 1 where GCPinSafety model refuses",
        ),
        (
            "ChunkedCoverage: accepting incomplete chunk partition",
            r#""kind":"chunked_partition","chunk_width":4,"total":8,"submitted":[[0,4]]"#,
            &[(
                r#""admitted":false,"output_ranges":[],"chunk_count":0"#,
                r#""admitted":true,"output_ranges":[[0,4]],"chunk_count":1"#,
            )],
            "chunked_partition (c=4, total=8): Rust admitted=true, model=false",
        ),
        (
            "ExtentGenerationSafety: admitting drain while writeback in flight",
            r#""kind":"extent_drain","range":[9,64,32],"barrier":"completed_retirement","next_gen":[5,10],"floor_gen":[0,0],"seen":[],"inflight":[[9,64,32]]"#,
            &[(
                r#""admitted":false,"returned_gen":null,"active_after":[0,0]"#,
                r#""admitted":true,"returned_gen":[5,10],"active_after":[5,10]"#,
            )],
            "extent_drain on { arena := 9, offset := 64, length := 32 } at { root := 5, seq := 10 }: Rust admitted=true, model=false",
        ),
        (
            "ExtentGenerationSafety: admitting incomplete barrier on drained range",
            r#""kind":"extent_drain","range":[9,64,32],"barrier":"retirement_incomplete","next_gen":[5,10],"floor_gen":[0,0],"seen":[],"inflight":[]"#,
            &[(
                r#""admitted":false,"returned_gen":null,"active_after":[0,0]"#,
                r#""admitted":true,"returned_gen":[5,10],"active_after":[5,10]"#,
            )],
            "extent_drain on { arena := 9, offset := 64, length := 32 } at { root := 5, seq := 10 }: Rust admitted=true, model=false",
        ),
        (
            "ReclamationSafety: admitting DerivedOnly candidate (M8)",
            r#""kind":"reclaim_verdict""#,
            &[(
                r#""coverage":"derived_only","verdict":"uncovered""#,
                r#""coverage":"derived_only","verdict":"accept""#,
            )],
            "planVerdict mismatch: Rust=StoreM4.ReclamationSafety.Verdict.accept, model=StoreM4.ReclamationSafety.Verdict.uncovered",
        ),
        (
            "ReclamationSafety: double-releasing published range on resume (M9)",
            r#""kind":"reclaim_resume","barrier":"free_space_published""#,
            &[(r#""release":false"#, r#""release":true"#)],
            "resumeRelease(StoreM4.ReclamationSafety.Barrier.freeSpacePublished) mismatch:",
        ),
    ];

    for (label, filter, replacements, expected_diag) in negative_controls {
        let (step_idx, tampered) = mutate_first_matching_line(lines, filter, replacements);
        let output = run_conformance_checker(bin, &tampered);
        assert!(
            !output.status.success(),
            "negative control `{label}` unexpectedly passed Lean conformance!\nstdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        let expected = format!("step {step_idx} fails: {expected_diag}");
        assert!(
            stderr.contains(&expected),
            "negative control `{label}` failed with unexpected diagnostic.\nexpected substring: {expected}\nactual stderr: {stderr}"
        );
    }
}
