//! Kept prefixes agree; dispatched suffix checkpoints are still charged.
mod cost_cell;

fn cell(suffix: usize) -> (u64, u64) {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "checkpoint_recovery::required_chain::concurrent_waves::charge_prefix::cost_cell::failed_advancement_cost_cell", "--nocapture"])
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
    println!("FAILED_PREFIX_DELTA:{actual:?}");
    let prefix = |row: usize| &costs[row][..=0];
    let prefix_only = (
        prefix(1).iter().sum::<u64>() - prefix(0).iter().sum::<u64>(),
        prefix(1).iter().max().unwrap() - prefix(0).iter().max().unwrap(),
    );
    assert!(
        std::panic::catch_unwind(|| judge(prefix_only)).is_err(),
        "the same law assertion rejects the prefix-only mutant model"
    );
}
