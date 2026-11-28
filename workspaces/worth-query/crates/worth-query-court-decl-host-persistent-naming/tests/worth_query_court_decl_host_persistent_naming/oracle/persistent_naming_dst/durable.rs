//! Phase 7B: the seeded walks on a real Directory Home, crashed (dropped
//! still owned) and reopened as a fresh installation after every step:
//! alternately from a checkpoint captured at that boundary and by replaying
//! the Journal since the last one. After each reopen every retained name the
//! walk ever held is presented to the fresh installation and must answer
//! exactly as the reference oracle (and as it did before the crash): genuine
//! names readmit through their persisted births, and the derived index is
//! rebuilt from persisted events alone.
//!
//! Every Directory Home open and checkpoint costs about a second in the
//! unoptimized test profile, so the default run is 2 seeds of 4 steps;
//! `WORTH_DST_DURABLE_SEEDS` and `WORTH_DST_DURABLE_STEPS` (at most the
//! in-memory walk's 10) widen the sweep, and `WORTH_DST_SEED` runs one seed.
use super::{seeds, sweep, var, Coverage, STEPS};

/// Steps per durable seed: each one committed, crashed and reopened.
pub(super) fn steps() -> usize {
    var("WORTH_DST_DURABLE_STEPS").map_or(4, |steps| (steps as usize).clamp(1, STEPS))
}

#[test]
fn seeded_walks_survive_store_reopen_after_every_step() {
    let seeds = seeds("WORTH_DST_DURABLE_SEEDS", 2);
    let total: Coverage = sweep(&seeds, true);
    eprintln!(
        "persistent-naming durable DST: {} seeds x {} steps, {total:?}",
        seeds.len(),
        steps()
    );
    assert_eq!(
        total.reopens,
        seeds.len() * steps(),
        "a reopen after every step"
    );
    assert!(
        total.checkpointed_reopens > 0,
        "some reopens restore a checkpoint"
    );
    assert!(
        total.checkpointed_reopens < total.reopens,
        "some reopens replay the Journal tail"
    );
}
