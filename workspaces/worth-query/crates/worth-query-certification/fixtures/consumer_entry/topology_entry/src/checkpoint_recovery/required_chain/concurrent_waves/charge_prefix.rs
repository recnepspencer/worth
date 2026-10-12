//! Both histories have the same failed prefix, so their charges must agree.
use super::{computation::Failure, history};
use std::num::NonZeroUsize;
use worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest as Placement;

fn cell(suffix: usize) -> (u64, u64) {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "checkpoint_recovery::required_chain::concurrent_waves::charge_prefix::a_completion_failure_does_not_charge_a_suffix_kernel", "--nocapture"])
        .env("WORTH_FAILED_PREFIX_SUFFIX", suffix.to_string())
        .output().unwrap();
    assert!(
        output.status.success(),
        "the real failed advancement ran: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let metrics = stdout
        .lines()
        .find_map(|line| line.strip_prefix("FAILED_PREFIX_COST:"))
        .expect("the application returns its failed advancement's metrics");
    let (work, span) = metrics.split_once(',').unwrap();
    (work.parse().unwrap(), span.parse().unwrap())
}

#[test]
fn a_completion_failure_does_not_charge_a_suffix_kernel() {
    let _guard = super::checkpoint_recovery_test_guard();
    if let Ok(suffix) = std::env::var("WORTH_FAILED_PREFIX_SUFFIX") {
        let observed = history::run_costs(
            Placement::Leased(NonZeroUsize::MIN),
            Some((0, Failure::Complete)),
            [3, suffix.parse().unwrap()],
        );
        println!("FAILED_PREFIX_COST:{},{}", observed.work, observed.span);
        return;
    }
    // Fresh processes keep all issued binding identities and their encodings
    // equal. Only the second root's declared checkpoint units differ; it is
    // beyond the first root's completion failure in the canonical wave.
    let without_suffix_work = cell(0);
    let with_suffix_work = cell(7);
    assert_eq!(
        with_suffix_work, without_suffix_work,
        "a failed advancement cannot charge a suffix kernel or its span: prefix is unchanged"
    );
}
