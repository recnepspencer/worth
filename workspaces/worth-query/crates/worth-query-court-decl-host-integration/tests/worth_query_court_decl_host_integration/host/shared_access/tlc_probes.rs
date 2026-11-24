//! Positive TLC trace validation + 10 adversarial negative TLC trace probes
//! with strict semantic TLC rejection assertions (never generic `!status.success()`).

use std::path::{Path, PathBuf};
use std::process::Command;

use worth_query_host::facade::shared_access::ScopeAccess;

use super::dst::run_seed_ops;
use super::dst_oracle::DstOp;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("court crate sits 4 levels below repository root")
        .to_path_buf()
}

struct TlcRunResult {
    exit_code: Option<i32>,
    combined_output: String,
}

fn run_tlc_trace(ndjson_path: &Path) -> TlcRunResult {
    let root = repo_root();
    let script = root.join("tools/tla/run_tlc.py");
    let spec = root.join("specs/tla/shared_access/SharedAccessTrace.tla");
    let output = Command::new("python3")
        .arg(&script)
        .arg("trace")
        .arg(&spec)
        .arg(ndjson_path)
        .output()
        .expect("python3 run_tlc.py executes");
    let combined_output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    TlcRunResult {
        exit_code: output.status.code(),
        combined_output,
    }
}

/// Asserts that TLC rejected the probe for a genuine semantic trace/invariant/temporal
/// failure — NEVER a syntax error, parse error, missing tool, or runner crash.
fn assert_semantic_tlc_rejection(name: &str, result: &TlcRunResult) {
    let out = &result.combined_output;
    for forbidden in [
        "Parse Error",
        "Semantic error",
        "Syntax error",
        "Traceback",
        "ERROR:",
        "Exception in thread",
        "NoClassDefFoundError",
    ] {
        assert!(
            !out.contains(forbidden),
            "probe `{name}` failed with infrastructure/parse error ({forbidden}) instead of semantic TLC rejection!\n{out}"
        );
    }
    let valid_semantic_code = matches!(result.exit_code, Some(11..=13));
    let has_semantic_banner = out.contains("Error: Deadlock reached.")
        || out.contains("Error: Invariant ")
        || out.contains("Error: Temporal properties were violated.")
        || out.contains("Error: Action property ");
    assert!(
        valid_semantic_code && has_semantic_banner,
        "probe `{name}` did not produce a genuine semantic TLC rejection (exit={:?}):\n{out}",
        result.exit_code
    );
}

/// Exercises every single action in `SharedAccess.tla` in one real passive runtime trace.
fn full_coverage_script() -> Vec<DstOp> {
    let mut ops = grant_and_watch_prefix();
    ops.extend(revoke_and_reopen_suffix());
    ops
}

fn grant_and_watch_prefix() -> Vec<DstOp> {
    let read = |principal, scope| DstOp::RemoteRead { principal, scope };
    let watch = |principal, scope| DstOp::RemoteWatchOpen { principal, scope };
    let poll = |principal, scope| DstOp::RemoteWatchPoll { principal, scope };
    vec![
        read(0, 0),
        DstOp::GrantScopeAccess {
            principal: 0,
            scope: 0,
            access: ScopeAccess::ReadAndWatch,
        },
        DstOp::GrantScopeAccess {
            principal: 1,
            scope: 0,
            access: ScopeAccess::ReadOnly,
        },
        DstOp::OwnerWrite { scope: 0 },
        DstOp::OwnerWrite { scope: 1 },
        read(0, 0),
        read(0, 1),
        watch(1, 0),
        watch(0, 0),
        poll(0, 0),
        DstOp::OwnerWrite { scope: 0 },
        poll(0, 0),
    ]
}

fn revoke_and_reopen_suffix() -> Vec<DstOp> {
    let read = |principal, scope| DstOp::RemoteRead { principal, scope };
    let poll = |principal, scope| DstOp::RemoteWatchPoll { principal, scope };
    vec![
        DstOp::TamperAuthToken { principal: 0 },
        read(0, 0),
        DstOp::RestoreAuthToken { principal: 0 },
        DstOp::OwnerWrite { scope: 0 },
        DstOp::RevokeScopeAccess {
            principal: 0,
            scope: 0,
        },
        poll(0, 0),
        read(0, 0),
        DstOp::CloseHome,
        read(1, 0),
        DstOp::ReopenHome,
        read(1, 0),
        read(0, 0),
        poll(0, 0),
    ]
}

fn mutate_line(lines: &[&str], index: usize, from: &str, to: &str) -> String {
    let mut owned: Vec<String> = lines.iter().map(|s| (*s).to_owned()).collect();
    assert!(
        owned[index].contains(from),
        "probe target line {index} did not contain {from:?}: {}",
        owned[index]
    );
    owned[index] = owned[index].replacen(from, to, 1);
    owned.join("\n") + "\n"
}

fn probe_trace_dir() -> PathBuf {
    if let Ok(target) = std::env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target).join("shared-access-tlc-probes")
    } else {
        repo_root().join("target/shared-access-tlc-probes")
    }
}

#[test]
fn tlc_accepts_passive_runtime_trace_and_rejects_ten_adversarial_probes() {
    let run = run_seed_ops(9001, Some(&full_coverage_script()))
        .expect("full-coverage real-runtime script succeeds");
    let dir = probe_trace_dir();
    std::fs::create_dir_all(&dir).unwrap();

    let positive_path = dir.join("positive.ndjson");
    std::fs::write(&positive_path, &run.trace_ndjson).unwrap();
    let positive = run_tlc_trace(&positive_path);
    assert!(
        positive.exit_code == Some(0)
            && positive
                .combined_output
                .contains("Model checking completed. No error has been found."),
        "TLC rejected valid passive runtime trace (exit={:?}):\n{}",
        positive.exit_code,
        positive.combined_output
    );

    let lines: Vec<&str> = run.trace_ndjson.lines().collect();
    assert_eq!(lines.len(), 25, "expected all 25 scripted steps");

    for (name, mutated_ndjson) in build_ten_negative_probes(&lines) {
        let probe_path = dir.join(format!("negative-{name}.ndjson"));
        std::fs::write(&probe_path, &mutated_ndjson).unwrap();
        let probe_res = run_tlc_trace(&probe_path);
        assert_semantic_tlc_rejection(name, &probe_res);
    }

    let _ = std::fs::remove_dir_all(&dir);
}

const TEN_PROBE_SPECS: [(&str, usize, &str, &str); 10] = [
    (
        "scope_leak",
        6,
        "\"outcome\":\"AuthzCapabilityDenied\",\"rows\":[]",
        "\"outcome\":\"Delivered\",\"rows\":[{\"scope\":\"Project2\",\"version\":1}]",
    ),
    (
        "post_revoke_read",
        18,
        "\"outcome\":\"AuthzCapabilityDenied\",\"rows\":[]",
        "\"outcome\":\"Delivered\",\"rows\":[{\"scope\":\"Project1\",\"version\":3}]",
    ),
    (
        "post_revoke_watch_delivery",
        17,
        "\"outcome\":\"AuthzWatchCapabilityRevoked\",\"rows\":[]",
        "\"outcome\":\"Delivered\",\"rows\":[{\"scope\":\"Project1\",\"version\":3}]",
    ),
    (
        "unauthenticated_read",
        13,
        "\"outcome\":\"AuthInvalidSignature\",\"rows\":[]",
        "\"outcome\":\"Delivered\",\"rows\":[{\"scope\":\"Project1\",\"version\":2}]",
    ),
    (
        "wrong_refusal_category",
        18,
        "\"outcome\":\"AuthzCapabilityDenied\"",
        "\"outcome\":\"AuthInvalidSignature\"",
    ),
    (
        "wrong_refusal_reason_within_category",
        17,
        "\"outcome\":\"AuthzWatchCapabilityRevoked\"",
        "\"outcome\":\"AuthzCapabilityDenied\"",
    ),
    (
        "wrong_scope_row_delivered",
        5,
        "\"rows\":[{\"scope\":\"Project1\",\"version\":1}]",
        "\"rows\":[{\"scope\":\"Project2\",\"version\":1}]",
    ),
    (
        "extra_row_delivered",
        5,
        "\"rows\":[{\"scope\":\"Project1\",\"version\":1}]",
        "\"rows\":[{\"scope\":\"Project1\",\"version\":1},{\"scope\":\"Project2\",\"version\":1}]",
    ),
    (
        "read_only_watch_escalation",
        7,
        "\"outcome\":\"AuthzCapabilityDenied\",\"rows\":[]",
        "\"outcome\":\"Ok\",\"rows\":[{\"scope\":\"Project1\",\"version\":1}]",
    ),
    (
        "stale_watch_after_reopen",
        24,
        "\"outcome\":\"WatchNotFound\",\"rows\":[]",
        "\"outcome\":\"Delivered\",\"rows\":[{\"scope\":\"Project1\",\"version\":3}]",
    ),
];

fn build_ten_negative_probes(lines: &[&str]) -> Vec<(&'static str, String)> {
    TEN_PROBE_SPECS
        .iter()
        .map(|&(name, idx, from, to)| (name, mutate_line(lines, idx, from, to)))
        .collect()
}
