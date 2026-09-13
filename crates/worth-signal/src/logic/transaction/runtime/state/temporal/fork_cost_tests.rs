use std::env;
use std::hint::black_box;
use std::process::Command;

use stats_alloc::{Region, INSTRUMENTED_SYSTEM};

use super::TemporalRuntimeState;
use crate::data::temporal::{ClockTick, TemporalCondition, TemporalWakeOwner};

const TEST_NAME: &str = "logic::transaction::runtime::state::temporal::fork_cost_tests::same_tick_same_owner_first_write_is_bounded_by_inner_frontier_roots";

#[test]
fn same_tick_same_owner_first_write_is_bounded_by_inner_frontier_roots() {
    const CHILD_PROCESS: &str = "WORTH_SIGNAL_TEMPORAL_FORK_COST_CHILD";
    if env::var_os(CHILD_PROCESS).is_none() {
        let output = Command::new(env::current_exe().expect("test executable resolves"))
            .arg("--exact")
            .arg(TEST_NAME)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CHILD_PROCESS, "1")
            .output()
            .expect("isolated temporal allocation probe starts");
        let stdout = String::from_utf8_lossy(&output.stdout);
        print!("{stdout}");
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        assert!(
            output.status.success(),
            "isolated temporal allocation probe failed"
        );
        assert!(
            stdout.contains(TEST_NAME) && stdout.contains("test result: ok. 1 passed; 0 failed;"),
            "isolated temporal probe must execute exactly one named test"
        );
        return;
    }

    let due_tick = ClockTick::new(7);
    let condition = TemporalCondition::after(7).expect("positive temporal delay");
    let mut samples = Vec::new();
    let mut eager_copy_bytes = None;

    for wake_count in [64_usize, 4_096, 65_536] {
        let mut source = TemporalRuntimeState::default();
        for _ in 0..wake_count {
            source
                .schedule_wake(condition.clone(), due_tick, None)
                .expect("same-tick wake schedules");
        }

        if wake_count == 65_536 {
            let eager_region = Region::new(&INSTRUMENTED_SYSTEM);
            black_box(source.clone());
            eager_copy_bytes = Some(eager_region.change().bytes_allocated);
        }

        let mut fork = source.fork_persistent();
        assert!(
            source.shares_storage_with(&fork),
            "allocation sample must begin from a genuine shared fork"
        );

        let region = Region::new(&INSTRUMENTED_SYSTEM);
        fork.schedule_wake(condition.clone(), due_tick, None)
            .expect("fork same-tick wake schedules");
        let allocation = region.change();
        samples.push((
            wake_count,
            allocation.allocations,
            allocation.bytes_allocated,
        ));

        assert_eq!(source.scheduled_wakes.len(), wake_count);
        assert_eq!(fork.scheduled_wakes.len(), wake_count + 1);
        assert_eq!(
            source.scheduled_frontier.get(&due_tick).unwrap().len(),
            wake_count
        );
        assert_eq!(
            fork.scheduled_frontier.get(&due_tick).unwrap().len(),
            wake_count + 1
        );
        assert_eq!(
            source
                .owner_frontier
                .get(&TemporalWakeOwner::Manual)
                .unwrap()
                .len(),
            wake_count
        );
        assert_eq!(
            fork.owner_frontier
                .get(&TemporalWakeOwner::Manual)
                .unwrap()
                .len(),
            wake_count + 1
        );
    }

    let minimum_calls = samples.iter().map(|(_, calls, _)| *calls).min().unwrap();
    let minimum_bytes = samples.iter().map(|(_, _, bytes)| *bytes).min().unwrap();
    for (wake_count, calls, bytes) in &samples {
        assert!(
            *calls <= minimum_calls + 96,
            "temporal first-write calls slope with {wake_count} same-frontier wakes: {calls} vs {minimum_calls}"
        );
        assert!(
            *bytes <= minimum_bytes + 96 * 1_024,
            "temporal first-write bytes slope with {wake_count} same-frontier wakes: {bytes} vs {minimum_bytes}"
        );
    }
    let maximum_first_write_bytes = samples.iter().map(|(_, _, bytes)| *bytes).max().unwrap();
    assert!(
        eager_copy_bytes.expect("largest sensitivity sample exists")
            > maximum_first_write_bytes.saturating_mul(8),
        "probe must distinguish a whole temporal-state copy from bounded frontier mutation"
    );
}

#[test]
fn inherited_frontier_retirement_keeps_next_wake_observation_local() {
    const RETIRED_PREFIX: usize = 4_096;
    let condition = TemporalCondition::after(1).expect("positive temporal delay");

    let mut scheduled_source = TemporalRuntimeState::default();
    let scheduled_wakes = (0..=RETIRED_PREFIX)
        .map(|offset| {
            scheduled_source
                .schedule_wake(condition.clone(), ClockTick::new(offset as u64 + 1), None)
                .expect("inherited scheduled frontier populates")
        })
        .collect::<Vec<_>>();
    let mut scheduled_fork = scheduled_source.fork_persistent();
    assert!(scheduled_source.shares_storage_with(&scheduled_fork));
    for wake in &scheduled_wakes[1..RETIRED_PREFIX] {
        scheduled_fork
            .retire_wake(
                wake.id(),
                crate::data::temporal::TemporalWakeRetirementReason::Cancelled,
                None,
            )
            .expect("inherited scheduled prefix retires");
    }
    scheduled_fork
        .retire_wake(
            scheduled_wakes[0].id(),
            crate::data::temporal::TemporalWakeRetirementReason::Cancelled,
            None,
        )
        .expect("earliest inherited wake retires after later wakes");
    let scheduled_snapshot = scheduled_fork.frontier_snapshot();
    assert_eq!(
        scheduled_snapshot.next_due_wake_id(),
        Some(scheduled_wakes[RETIRED_PREFIX].id())
    );
    assert_eq!(
        scheduled_snapshot.next_due_tick(),
        Some(ClockTick::new(RETIRED_PREFIX as u64 + 1))
    );
    assert_eq!(
        scheduled_source.frontier_snapshot().next_due_wake_id(),
        Some(scheduled_wakes[0].id()),
        "retiring the destination prefix must not alter the live source"
    );

    let mut ready_source = TemporalRuntimeState::default();
    let ready_wakes = (0..=RETIRED_PREFIX)
        .map(|_| {
            let scheduled = ready_source
                .schedule_wake(condition.clone(), ClockTick::new(0), None)
                .expect("ready frontier fixture schedules");
            ready_source
                .promote_wake_ready(scheduled.id(), None)
                .expect("due wake promotes")
        })
        .collect::<Vec<_>>();
    let mut ready_fork = ready_source.fork_persistent();
    assert!(ready_source.shares_storage_with(&ready_fork));
    for wake in &ready_wakes[1..RETIRED_PREFIX] {
        ready_fork
            .retire_wake(
                wake.id(),
                crate::data::temporal::TemporalWakeRetirementReason::Consumed,
                None,
            )
            .expect("inherited ready prefix retires");
    }
    ready_fork
        .retire_wake(
            ready_wakes[0].id(),
            crate::data::temporal::TemporalWakeRetirementReason::Consumed,
            None,
        )
        .expect("earliest inherited ready wake retires after later wakes");
    assert_eq!(
        ready_fork.frontier_snapshot().next_ready_wake_id(),
        Some(ready_wakes[RETIRED_PREFIX].id())
    );
    assert_eq!(
        ready_source.frontier_snapshot().next_ready_wake_id(),
        Some(ready_wakes[0].id()),
        "retiring the destination ready prefix must not alter the live source"
    );
}
