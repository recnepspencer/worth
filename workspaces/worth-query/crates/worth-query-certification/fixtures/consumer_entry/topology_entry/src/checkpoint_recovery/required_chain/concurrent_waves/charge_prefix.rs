//! Kept prefixes agree; dispatched suffix checkpoints are still charged.
use super::{computation::Failure, history};
use std::num::NonZeroUsize;
use worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest as Placement;

fn cell(suffix: usize) -> (u64, u64) {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "checkpoint_recovery::required_chain::concurrent_waves::charge_prefix::a_completion_failure_charges_a_dispatched_suffix_kernel", "--nocapture"])
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
fn a_completion_failure_charges_a_dispatched_suffix_kernel() {
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
    let costs = [[3_u64, 0], [3, 7]];
    let observed = [cell(costs[0][1] as usize), cell(costs[1][1] as usize)];
    let actual = (observed[1].0 - observed[0].0, observed[1].1 - observed[0].1);
    let model = (
        costs[1].iter().sum::<u64>() - costs[0].iter().sum::<u64>(),
        costs[1].iter().max().unwrap() - costs[0].iter().max().unwrap(),
    );
    let judge = |expected| {
        assert_eq!(
            actual, expected,
            "discarded suffix compute remains charged; span is the dispatched critical path"
        )
    };
    judge(model);
    assert!(
        std::panic::catch_unwind(|| judge((0, 0))).is_err(),
        "the same law assertion rejects the prefix-only mutant model"
    );
    let remainder = |row: usize| {
        (
            observed[row].0 - costs[row].iter().sum::<u64>(),
            observed[row].1 - costs[row].iter().max().unwrap(),
        )
    };
    assert_eq!(
        remainder(0),
        remainder(1),
        "unchanged owner work is one common remainder"
    );
}
