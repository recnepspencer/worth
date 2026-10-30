#[allow(unused_imports)]
use crate::phase_eight_process::child_lifecycle::tests::*;
use std::io::Write;
use std::time::Duration;
use {std::process::Command, std::process::Stdio};
use {
    worth_store_testkit_recovery_runtime::child_lifecycle::output_capture::MAXIMUM_PIPE_BYTES,
    worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard,
};

#[test]
fn output_child() {
    let Ok(mode) = std::env::var(CHILD_ROLE) else {
        return;
    };
    let bytes = vec![
        b'x';
        if mode == "overflow" {
            MAXIMUM_PIPE_BYTES + 8192
        } else {
            128 * 1024
        }
    ];
    let _ = std::io::stderr().write_all(&bytes);
    if mode != "overflow" {
        std::io::stdout().write_all(&bytes).unwrap();
    }
}

#[test]
fn both_full_pipes_drain_before_child_exit_without_losing_bytes() {
    let output = child("both")
        .wait_with_output_within(std::time::Duration::from_secs(10))
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stderr, vec![b'x'; 128 * 1024]);
    assert!(output
        .stdout
        .windows(128 * 1024)
        .any(|bytes| bytes.iter().all(|byte| *byte == b'x')));
}

#[test]
fn pipe_capture_has_an_enforced_byte_bound() {
    let denial = child("overflow")
        .wait_with_output_within(std::time::Duration::from_secs(10))
        .unwrap_err();
    assert_eq!(
        denial,
        format!("Phase 8 stderr exceeded {MAXIMUM_PIPE_BYTES}-byte capture bound")
    );
}
