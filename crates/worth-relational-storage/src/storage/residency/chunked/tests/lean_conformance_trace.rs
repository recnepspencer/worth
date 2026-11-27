//! Executable Rust <-> Lean conformance traces for `StoreM4.ChunkedCoverage`
//! (`ChunkedCoverageAndSummarySound`), driven by `worth-relational-storage`'s
//! real `RegionChunkDirectory::admit`/`chunk_of`, `ChunkHistoryImage::flattened`
//! /`slot_versions`, `consolidate_history_chunk`/`verify_bound_chunk`,
//! `seal_chunk_image`, and `scan_chunks`.
mod directory_and_consolidation;
mod summary;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use super::super::canonical_read::ChunkResidency;
use super::super::chunk_directory::RegionChunkDirectory;
use super::super::summary_scan::ChunkScan;
use super::member_store::{accountant, LAYOUT};
use crate::identity::data::KindId;
use crate::storage::residency::working_set::ResidencyPoolKind;

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

    fn to_ndjson(&self) -> String {
        let mut out = self.lines.join("\n");
        out.push('\n');
        out
    }
}

pub(super) fn u64_list_json(values: &[u64]) -> String {
    let items: Vec<String> = values.iter().map(u64::to_string).collect();
    format!("[{}]", items.join(","))
}

pub(super) fn rows_json(rows: &[(u64, Vec<u64>)]) -> String {
    let items: Vec<String> = rows
        .iter()
        .map(|(s, vs)| format!("[{s},{}]", u64_list_json(vs)))
        .collect();
    format!("[{}]", items.join(","))
}

struct TempTraceFile(PathBuf);

impl TempTraceFile {
    fn write(ndjson: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "worth-rel-chunked-lean-{}-{id}.ndjson",
            std::process::id()
        ));
        std::fs::write(&path, ndjson).expect("write relational chunked conformance trace");
        Self(path)
    }
}

impl Drop for TempTraceFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("worth-relational-storage crate sits two levels below repository root")
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
                "WORTH_REQUIRE_LEAN_CONFORMANCE is set, but `lake` was not found"
            );
            eprintln!("relational_chunked_lean_conformance: skipping (`lake` not installed)");
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
        let output = Command::new(&lake)
            .current_dir(&lean_dir)
            .args(["build", "store_m4_core_conformance"])
            .output()
            .expect("invoke `lake build store_m4_core_conformance`");
        assert!(
            output.status.success(),
            "lake build store_m4_core_conformance failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(bin_path.is_file(), "missing {}", bin_path.display());
        Some(bin_path)
    })
    .as_ref()
}

fn run_checker(bin: &Path, ndjson: &str) -> Output {
    let temp = TempTraceFile::write(ndjson);
    Command::new(bin)
        .arg(&temp.0)
        .output()
        .expect("execute store_m4_core_conformance")
}

#[test]
fn relational_chunked_substrate_conforms_to_store_m4_chunked_coverage() {
    let mut trace = Trace::default();
    directory_and_consolidation::record_directory_cases(&mut trace);
    directory_and_consolidation::record_consolidation_cases(&mut trace);
    let (store, fixtures, mut tampered_sealed) = summary::record_summary_cases(&mut trace);
    assert!(
        trace.lines.len() >= 19,
        "expected >= 19 relational chunked steps, got {}",
        trace.lines.len()
    );
    let ndjson = trace.to_ndjson();
    if let Ok(path) = std::env::var("WORTH_RELATIONAL_CHUNKED_CONFORMANCE_TRACE") {
        std::fs::write(path, &ndjson).expect("write relational chunked trace");
    }
    let Some(bin) = conformance_binary() else {
        return;
    };
    let output = run_checker(bin, &ndjson);
    assert!(
        output.status.success(),
        "Lean conformance rejected genuine relational chunked trace:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let controls: [(&str, &str, &str, &str, &str); 4] = [
        (
            "directory gap admitted",
            r#""kind":"relational_chunk_directory","chunk_slots":64,"entity_slots":200,"relation_slots":70,"submitted":[[0,0,64],[0,128,192]"#,
            r#""admitted":false"#,
            r#""admitted":true"#,
            "relational_chunk_directory (c=64, ent=200, rel=70): Rust admitted=true, model=false",
        ),
        (
            "lossy consolidation accepted",
            r#""kind":"relational_chunk_consolidation""#,
            r#""is_flattened_candidate":false,"verify_ok":false,"consolidate_ok":false"#,
            r#""is_flattened_candidate":false,"verify_ok":true,"consolidate_ok":true"#,
            "relational_chunk_consolidation verify_ok=true, model meaning_preserved=false",
        ),
        (
            "seal omitted summary claim accepted",
            r#""kind":"relational_summary_claim""#,
            r#""claimed_summary":[1],"admitted":false"#,
            r#""claimed_summary":[1],"admitted":true"#,
            "relational_summary_claim: Rust admitted=true, model=false",
        ),
        (
            "pruned scan drops matching record",
            r#""kind":"relational_summary_scan","query_key":7"#,
            r#""pruned_hits":[74],"canonical_chunks_read":3,"pruned_chunks_read":1"#,
            r#""pruned_hits":[],"canonical_chunks_read":3,"pruned_chunks_read":0"#,
            "relational_summary_scan pruned_hits=[], model prunedScan=[74]",
        ),
    ];
    for (label, filter, from, to, expected_diag) in controls {
        let mut lines = trace.lines.clone();
        let (idx, line) = lines
            .iter_mut()
            .enumerate()
            .find(|(_, l)| l.contains(filter) && l.contains(from))
            .unwrap_or_else(|| panic!("missing trace line for `{label}`"));
        *line = line.replacen(from, to, 1);
        let tampered = format!("{}\n", lines.join("\n"));
        let res = run_checker(bin, &tampered);
        assert!(
            !res.status.success(),
            "control `{label}` unexpectedly passed"
        );
        let stderr = String::from_utf8_lossy(&res.stderr);
        let expected = format!("step {idx} fails: {expected_diag}");
        assert!(
            stderr.contains(&expected),
            "control `{label}`: expected `{expected}`, got `{stderr}`"
        );
    }

    // Real Mutant M3: construct an incomplete summary omitting KindId(7), run
    // real `scan_chunks`, observe pruned scan drops the record while canonical
    // finds it, and verify the Lean checker catches it.
    let mut entry_json = serde_json::to_value(&tampered_sealed[1].0).unwrap();
    entry_json["head_summary"]["record_kinds"] = serde_json::json!([1]);
    tampered_sealed[1].0 = serde_json::from_value(entry_json).unwrap();
    let tampered_dir = RegionChunkDirectory::admit(LAYOUT, 192, 0, tampered_sealed).unwrap();
    let residency =
        ChunkResidency::new(&accountant(1 << 20, 1 << 20, 4096), ResidencyPoolKind::Head);
    let mut m3_trace = Trace::default();
    summary::record_scan_step(
        &mut m3_trace,
        &tampered_dir,
        &residency,
        &store,
        &fixtures,
        ChunkScan::HeadKind(KindId::new(7)),
    );
    assert!(
        m3_trace.lines[0].contains(r#""canonical_hits":[74],"pruned_hits":[]"#),
        "expected pruned scan to drop slot 74 under incomplete summary: {}",
        m3_trace.lines[0]
    );
    let m3_res = run_checker(bin, &m3_trace.to_ndjson());
    assert!(
        !m3_res.status.success(),
        "Mutant M3 incomplete summary unexpectedly passed"
    );
    let m3_stderr = String::from_utf8_lossy(&m3_res.stderr);
    assert!(
        m3_stderr.contains(
            "step 0 fails: relational_summary_scan incomplete summary omits derived key 7"
        ),
        "unexpected Mutant M3 stderr: {m3_stderr}"
    );
}
