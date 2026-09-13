use super::{output_capture::MAXIMUM_PIPE_BYTES, ProcessChildGuard};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

const CHILD_ROLE: &str = "WORTH_C8_PIPE_CAPTURE_TEST";

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

fn child(mode: &str) -> ProcessChildGuard {
    ProcessChildGuard::new(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "child_lifecycle::tests::output_child",
                "--nocapture",
            ])
            .env(CHILD_ROLE, mode)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    )
}

#[test]
fn both_full_pipes_drain_before_child_exit_without_losing_bytes() {
    let output = child("both")
        .wait_with_output_within(Duration::from_secs(10))
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
        .wait_with_output_within(Duration::from_secs(10))
        .unwrap_err();
    assert_eq!(
        denial,
        format!("Phase 8 stderr exceeded {MAXIMUM_PIPE_BYTES}-byte capture bound")
    );
}
