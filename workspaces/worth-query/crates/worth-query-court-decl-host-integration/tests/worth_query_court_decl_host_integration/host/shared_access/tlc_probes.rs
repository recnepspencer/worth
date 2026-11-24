//! Positive TLC trace validation + 7 adversarial negative TLC trace probes.
//!
//! Runs `tools/tla/run_tlc.py trace specs/tla/shared_access/SharedAccessTrace.tla`
//! against a passively emitted real-runtime trace covering all 10 protocol actions,
//! then verifies that 7 distinct adversarial mutations are all REJECTED by TLC.

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

fn run_tlc_trace(ndjson_path: &Path) -> (bool, String) {
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
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), combined)
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

#[test]
fn tlc_accepts_passive_runtime_trace_and_rejects_seven_adversarial_probes() {
    let run = run_seed_ops(9001, Some(&full_coverage_script()))
        .expect("full-coverage real-runtime script succeeds");
    let dir = worth_query_testkit_host::facade::host::application_home::location()
        .with_extension("shared-tlc-probes");
    std::fs::create_dir_all(&dir).unwrap();

    let positive_path = dir.join("positive.ndjson");
    std::fs::write(&positive_path, &run.trace_ndjson).unwrap();
    let (accepted, log) = run_tlc_trace(&positive_path);
    assert!(accepted, "TLC rejected valid passive runtime trace:\n{log}");

    let lines: Vec<&str> = run.trace_ndjson.lines().collect();
    assert_eq!(lines.len(), 25, "expected all 25 scripted steps");

    for (name, mutated_ndjson) in build_seven_negative_probes(&lines) {
        let probe_path = dir.join(format!("negative-{name}.ndjson"));
        std::fs::write(&probe_path, &mutated_ndjson).unwrap();
        let (probe_accepted, probe_log) = run_tlc_trace(&probe_path);
        assert!(
            !probe_accepted,
            "TLC MUST reject negative probe `{name}`, but accepted it!\n{probe_log}"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

fn build_seven_negative_probes(lines: &[&str]) -> Vec<(&'static str, String)> {
    const SPECS: [(&str, usize, &str, &str); 7] = [
        (
            "scope_leak",
            6,
            "\"outcome\":\"AuthorizationRefused\",\"value\":0",
            "\"outcome\":\"Delivered\",\"value\":1",
        ),
        (
            "post_revoke_read",
            18,
            "\"outcome\":\"AuthorizationRefused\",\"value\":0",
            "\"outcome\":\"Delivered\",\"value\":3",
        ),
        (
            "post_revoke_watch_delivery",
            17,
            "\"outcome\":\"AuthorizationRefused\",\"value\":0",
            "\"outcome\":\"Delivered\",\"value\":3",
        ),
        (
            "unauthenticated_read",
            13,
            "\"outcome\":\"AuthenticationRefused\",\"value\":0",
            "\"outcome\":\"Delivered\",\"value\":2",
        ),
        (
            "wrong_refusal_variant",
            18,
            "\"outcome\":\"AuthorizationRefused\"",
            "\"outcome\":\"AuthenticationRefused\"",
        ),
        (
            "read_only_watch_escalation",
            7,
            "\"outcome\":\"AuthorizationRefused\",\"value\":0",
            "\"outcome\":\"Ok\",\"value\":1",
        ),
        (
            "stale_watch_after_reopen",
            24,
            "\"outcome\":\"WatchRefused\",\"value\":0",
            "\"outcome\":\"Delivered\",\"value\":3",
        ),
    ];
    SPECS
        .iter()
        .map(|&(name, idx, from, to)| (name, mutate_line(lines, idx, from, to)))
        .collect()
}
