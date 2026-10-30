#![cfg_attr(test, allow(clippy::disallowed_methods))]
#[allow(dead_code)]
mod phase_three_support;
#[path = "phase_seven_cleanup/process.rs"]
mod process;

use process::{assert_child_succeeded, ProcessWorld};

#[test]
fn post_publication_cleanup_removes_only_the_exact_checkpoint_covered_wal() {
    let world = ProcessWorld::write("settled");
    let candidate = world.oldest_wal();
    let bytes = std::fs::metadata(&candidate).unwrap().len();
    let cleanup = world.cleanup(bytes, "complete", &candidate);
    assert_child_succeeded("cleanup", &cleanup);
    assert!(!candidate.exists());
    assert!(world.newest_wal().exists());
}

#[test]
fn cleanup_limit_defers_the_exact_candidate_without_invalidating_recovery() {
    let world = ProcessWorld::write("settled");
    let candidate = world.oldest_wal();
    let bytes = std::fs::metadata(&candidate).unwrap().len();
    let cleanup = world.cleanup(bytes - 1, "byte-limit", &candidate);
    assert_child_succeeded("cleanup", &cleanup);
    assert!(candidate.exists());
}

#[test]
fn unresolved_operation_fate_retains_checkpoint_covered_wal() {
    let world = ProcessWorld::write("unresolved");
    let candidate = world.oldest_wal();
    let bytes = std::fs::metadata(&candidate).unwrap().len();
    let cleanup = world.cleanup(bytes, "unresolved", &candidate);
    assert_child_succeeded("cleanup", &cleanup);
    assert!(candidate.exists());
}

#[test]
fn cleanup_candidate_limit_removes_only_the_admitted_prefix() {
    let world = ProcessWorld::write("multiple-settled");
    let wal = world.wal_files();
    assert!(
        wal.len() >= 3,
        "ordinary mutations must rotate three WAL artifacts"
    );
    let bytes = std::fs::metadata(&wal[0]).unwrap().len();
    let cleanup = world.cleanup_with_candidate_limit(bytes * 2, 1, "candidate-limit", &wal[0]);
    assert_child_succeeded("cleanup", &cleanup);
    assert!(!wal[0].exists());
    assert!(wal[1].exists());
    assert!(wal.last().unwrap().exists());
}

#[test]
fn cleanup_cancellation_before_first_effect_defers_every_candidate() {
    let world = ProcessWorld::write("settled");
    let candidate = world.oldest_wal();
    let bytes = std::fs::metadata(&candidate).unwrap().len();
    let cleanup = world.cleanup_with_cancellation(bytes, 0, &candidate);
    assert_child_succeeded("cleanup cancellation before first", &cleanup);
    assert!(candidate.exists());
}

#[test]
fn cleanup_cancellation_between_effects_retains_the_exact_completed_prefix() {
    let world = ProcessWorld::write("multiple-settled");
    let wal = world.wal_files();
    assert!(wal.len() >= 3);
    let bytes = wal
        .iter()
        .map(|path| std::fs::metadata(path).unwrap().len())
        .sum();
    let cleanup = world.cleanup_with_cancellation(bytes, 1, &wal[0]);
    assert_child_succeeded("cleanup cancellation after one", &cleanup);
    assert!(!wal[0].exists());
    assert!(wal[1].exists());
    assert!(wal.last().unwrap().exists());
}
