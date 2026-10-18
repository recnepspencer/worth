//! Parallel-dispatch proofs need an uncontended process-owned worker authority.
//! The child must report native evidence after its assertions, never just exit green.
pub(crate) enum NativeExecutionEvidence {
    PlacementRendezvous,
    ThroughputParity,
}

pub(crate) fn run_with_private_authority(
    test_name: &str,
    evidence: NativeExecutionEvidence,
) -> bool {
    let receipt_prefix = match evidence {
        NativeExecutionEvidence::PlacementRendezvous => "PLACEMENT_RENDEZVOUS",
        NativeExecutionEvidence::ThroughputParity => "THROUGHPUT_PARITY",
    };
    const CHILD_TEST: &str = "WORTH_SIGNAL_PRIVATE_EXECUTION_AUTHORITY";
    if std::env::var(CHILD_TEST).as_deref() == Ok(test_name) {
        return false;
    }
    let output = crate::process_deadline::output(
        std::process::Command::new(
            std::env::current_exe().expect("current test binary should be discoverable"),
        )
        .args(["--exact", test_name, "--nocapture"])
        .env(CHILD_TEST, test_name),
    )
    .expect("private authority test child should start");
    assert!(
        output.status.success(),
        "private authority test {test_name} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let receipt = stdout
        .lines()
        .find(|line| line.starts_with(receipt_prefix))
        .expect(
            "private authority child must execute the named test and record its native evidence",
        );
    println!("PRIVATE_AUTHORITY {test_name}: {receipt}");
    true
}
