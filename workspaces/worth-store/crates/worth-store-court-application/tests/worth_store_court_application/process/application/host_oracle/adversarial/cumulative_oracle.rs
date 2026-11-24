//! C4b.4: Deterministic seeded multi-family cumulative adversarial court oracle.
//!
//! Generates reproducible sequences from `seed: u64` via SplitMix64 mixing every
//! supported owner family (Core Query Host, C2.8 Durable World Branches &
//! Retirement, Signal + Bridge + Authored Branches, shared `DocumentRetentionSchema`
//! Governed Workflow + Branch Program Adoption, Authored Correction, and
//! External Dispatch Custody), injecting real OS kills across alternating build
//! identities and D14 derived-state discards, verified in fresh OS processes
//! against `MemoryOnly` under run-wide durable identity correspondence.
mod families_child;
mod families_lane;
mod host_lane;
mod signal_lane;
mod unified_home;
mod world_branch_child;
mod world_branch_lane;

use super::dimensions_matrix::{
    self, recovery::run_all_recovery_stages_exhaustive, run_parallel_items,
};
use super::{BUILD_PRIMARY, BUILD_SECONDARY, BUILD_TERTIARY};
use std::fmt;

pub(super) const BUILDS: [&str; 3] = [BUILD_PRIMARY, BUILD_SECONDARY, BUILD_TERTIARY];

/// Deterministic SplitMix64 PRNG (zero OS randomness, 100% seed-reproducible).
#[derive(Clone, Copy, Debug)]
pub(super) struct SplitMix64(u64);

impl SplitMix64 {
    pub(super) fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    pub(super) fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub(super) fn range(&mut self, bound: usize) -> usize {
        assert!(bound > 0, "SplitMix64::range requires bound > 0");
        (self.next_u64() as usize) % bound
    }

    pub(super) fn choose<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.range(items.len())]
    }

    pub(super) fn next_other_build(&mut self, current: &'static str) -> &'static str {
        let others: Vec<&'static str> = BUILDS.iter().copied().filter(|&b| b != current).collect();
        *self.choose(&others)
    }
}

/// Reproducible diagnostic context emitted on every oracle assertion failure.
#[derive(Clone, Debug)]
pub(super) struct StepCtx {
    pub(super) seed: u64,
    pub(super) step: usize,
    pub(super) family: &'static str,
    pub(super) op: String,
    pub(super) cut: String,
    pub(super) build: &'static str,
    pub(super) derived: &'static str,
}

impl fmt::Display for StepCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(seed={}, step={}, family={}, op={}, cut={}, build={}, derived={})",
            self.seed, self.step, self.family, self.op, self.cut, self.build, self.derived
        )
    }
}

fn focused_seeds() -> [u64; 4] {
    [0, 1, 2, 3]
}

fn parse_exhaustive_seeds() -> Vec<u64> {
    let raw = std::env::var("WORTH_ADV_ORACLE_SEEDS").unwrap_or_else(|_| "0..16".to_owned());
    let trimmed = raw.trim();
    if let Some((a, b)) = trimmed.split_once("..=") {
        let start: u64 = a.trim().parse().expect("valid start seed");
        let end: u64 = b.trim().parse().expect("valid inclusive end seed");
        assert!(start <= end, "empty seed range {trimmed}");
        return (start..=end).collect();
    }
    if let Some((a, b)) = trimmed.split_once("..") {
        let start: u64 = a.trim().parse().expect("valid start seed");
        let end: u64 = b.trim().parse().expect("valid end seed");
        assert!(start < end, "empty seed range {trimmed}");
        return (start..end).collect();
    }
    let seeds: Vec<u64> = trimmed
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().parse().expect("valid comma-separated seed"))
        .collect();
    assert!(
        !seeds.is_empty(),
        "WORTH_ADV_ORACLE_SEEDS must not be empty"
    );
    seeds
}

/// Execute all 7 owner families in a true operation-level PRNG-interleaved
/// schedule for one seed, mixing Core Query Host (mutate/checkpoint/close Store
/// seams + custody release), C2.8 Durable World Branches (independent reference
/// payloads + exact reclaim sets), Signal+Bridge+Authored Branches (root AND
/// forked branch barrier kills), shared `DocumentRetentionSchema` (Governed
/// Workflow + Branch Program Adoption on one durable Home), Authored
/// Correction, and External Dispatch Custody.
fn run_interleaved_cumulative_seed(seed: u64) {
    let mut rng = SplitMix64::new(seed ^ 0x494E_5452);
    let mut unified = unified_home::UnifiedHomeStepper::new(seed);
    let mut host = host_lane::HostStepper::new(seed);
    let mut wb = world_branch_lane::WorldBranchStepper::new(seed);
    let mut sig = signal_lane::SignalStepper::new(seed);
    let mut fam = families_lane::FamiliesStepper::new(seed);
    loop {
        let mut active = Vec::with_capacity(5);
        if !unified.is_done() {
            active.push(0usize);
        }
        if !host.is_done() {
            active.push(1usize);
        }
        if !wb.is_done() {
            active.push(2usize);
        }
        if !sig.is_done() {
            active.push(3usize);
        }
        if !fam.is_done() {
            active.push(4usize);
        }
        if active.is_empty() {
            break;
        }
        match *rng.choose(&active) {
            0 => unified.step(),
            1 => host.step(),
            2 => wb.step(),
            3 => sig.step(),
            4 => fam.step(),
            _ => unreachable!(),
        }
    }
}

#[test]
fn cumulative_adversarial_oracle_unified_single_home_seeded() {
    let seeds = focused_seeds();
    run_parallel_items(
        "C4b.4 unified 5-family single-home seeded",
        &seeds,
        |&seed| {
            unified_home::run_unified_single_home_seed(seed);
        },
    );
}

#[test]
fn cumulative_adversarial_oracle_core_host_seeded() {
    let seeds = focused_seeds();
    run_parallel_items("C4b.4 core host seeded", &seeds, |&seed| {
        host_lane::run_host_cumulative_seed(seed);
    });
}

#[test]
fn cumulative_adversarial_oracle_world_branches_seeded() {
    let seeds = focused_seeds();
    run_parallel_items("C4b.4 world branch retirement seeded", &seeds, |&seed| {
        world_branch_lane::run_world_branch_cumulative_seed(seed);
    });
}

#[test]
fn cumulative_adversarial_oracle_signal_bridge_seeded() {
    let seeds = focused_seeds();
    run_parallel_items("C4b.4 signal+bridge seeded", &seeds, |&seed| {
        signal_lane::run_signal_cumulative_seed(seed);
    });
}

#[test]
fn cumulative_adversarial_oracle_remaining_families_seeded() {
    let seeds = focused_seeds();
    run_parallel_items(
        "C4b.4 workflow+adoption/correction/dispatch seeded",
        &seeds,
        |&seed| {
            families_lane::run_remaining_families_cumulative_seed(seed);
        },
    );
}

/// Tier-3 exhaustive cumulative oracle controlled by `WORTH_ADV_ORACLE_SEEDS`
/// (e.g. `WORTH_ADV_ORACLE_SEEDS=0..16` or `WORTH_ADV_ORACLE_SEEDS=7,19,42`).
///
/// Combines:
/// 1. Operation-level PRNG-interleaved cumulative runs across ALL 7 owner families,
/// 2. Exhaustive enumeration of EVERY Store seam (`mutate`, `checkpoint`, AND
///    `close` x all 4 seams) across alternating builds and D14 discards,
/// 3. Exhaustive enumeration of EVERY recovery stage (all 13 physical stages
///    including `CleanupFreshnessRead`, `CleanupRemoval`, `ObligationDischarge`
///    + all 12 opening edges + 4-deep chained kills), AND
/// 4. Exhaustive `SynchronizeFileState` barrier sweep on a **forked** Signal
///    authored branch publication across alternating builds and D14 discards.
#[test]
#[ignore]
fn cumulative_adversarial_oracle_exhaustive_seeded() {
    let seeds = parse_exhaustive_seeds();
    run_parallel_items("C4b.4 exhaustive multi-family seeded", &seeds, |&seed| {
        run_interleaved_cumulative_seed(seed);
    });
    dimensions_matrix::run_all_store_seams_exhaustive();
    run_all_recovery_stages_exhaustive();
    signal_lane::fork_sweep::run_forked_signal_barrier_sweep();
}
