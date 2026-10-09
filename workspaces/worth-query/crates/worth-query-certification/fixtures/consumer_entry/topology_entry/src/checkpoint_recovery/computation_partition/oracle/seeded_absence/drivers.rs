use super::*;

#[test]
fn seeded_eviction_steps_equal_fresh_state_and_work_boundaries() {
    let _guard = checkpoint_recovery_test_guard();
    for seed in super::courtroom::SEEDS {
        run::<true, TOTALS_WORK, 1, 0>(Boundary::Evict, seed);
    }
}
#[test]
fn seeded_observation_steps_equal_fresh_state_and_work_boundaries() {
    let _guard = checkpoint_recovery_test_guard();
    for seed in super::courtroom::SEEDS {
        // Two holders contribute two incoming scans and each shared set's
        // outgoing scan. Their summed bounds exceed eight units per set;
        // that ceiling still admits the two-item preparation.
        run::<false, { 8 * OBSERVATION_SETS + 256 }, 1, 1>(Boundary::Observation, seed);
    }
}
#[test]
fn seeded_several_steps_equal_fresh_state_and_work_boundaries() {
    let _guard = checkpoint_recovery_test_guard();
    for seed in super::courtroom::SEEDS {
        run::<false, TOTALS_WORK, 2, 0>(Boundary::Several, seed);
    }
}
