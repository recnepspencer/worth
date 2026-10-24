//! Media sweep: every identified media operation of a home commit and of a
//! home checkpoint, faulted once each with `FailBefore` and
//! `IndeterminateAfterEffect`.
//!
//! For each seed and target the sweep counts every role's operations while
//! the target runs on a home holding the script before it, then runs one
//! case per (role, ordinal, directive) (`media_case`): the in-flight commit
//! is recovered exactly when the journal holds it, and then it is the
//! artifact its unresolved fate names; a proven-no-effect commit is absent; a
//! checkpoint fault leaves the old or the new checkpoint selected; the rest
//! of the script, a second recovery and fresh identities follow.
//!
//! Targets: the first commit, the first commit on a fork (which makes the
//! branch durable), the schema-transition commit, and a checkpoint that
//! replaces a selected one; two seeds each.

use std::num::NonZeroU64;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_store::physical_runtime::certification::MediaOperationRole;

use super::durable_run::{apply, served_schema};
use super::media_case::{run_case, Ended, Target};
use super::media_faults;
use super::prepared_home::PreparedHome;
use super::workload::{self, Script, Step};

const SEEDS: [u64; 2] = [0x5eed_0301, 0x5eed_0302];
const COMMITS: usize = 9;
const CHECKPOINT_EVERY: usize = 3;
/// Cases are independent homes in their own directories, so they run on a
/// few workers; each case's fault schedule belongs to its own Store.
const WORKERS: usize = 6;

/// How many cases ran and how the target ended in them.
#[derive(Default, Debug)]
struct Tally {
    cases: usize,
    skipped: usize,
    acknowledged: usize,
    refused: usize,
    unresolved_absent: usize,
    unresolved_recovered: usize,
}

impl Tally {
    fn add(&mut self, ended: Ended) {
        match ended {
            Ended::Skipped => self.skipped += 1,
            Ended::Acknowledged => self.acknowledged += 1,
            Ended::Refused => self.refused += 1,
            Ended::Unresolved { recovered: false } => self.unresolved_absent += 1,
            Ended::Unresolved { recovered: true } => self.unresolved_recovered += 1,
        }
        if !matches!(ended, Ended::Skipped) {
            self.cases += 1;
        }
    }
}

/// One seed's script and the steps around its target.
struct SeedSweep {
    seed: u64,
    script: Script,
    target: Target,
    /// Steps before the target that a closed home holds.
    prefix: usize,
}

impl SeedSweep {
    fn new(seed: u64, select: &impl Fn(&Script) -> Target) -> Self {
        let script = Script::generate(seed, COMMITS, CHECKPOINT_EVERY);
        let target = select(&script);
        // A fork is not durable until its branch's first commit, so forks
        // just before the target run again on each reopened home.
        let mut prefix = target.index();
        while prefix > 0 && matches!(script.steps[prefix - 1], Step::Fork { .. }) {
            prefix -= 1;
        }
        Self {
            seed,
            script,
            target,
            prefix,
        }
    }

    fn prefix(&self) -> &[Step] {
        &self.script.steps[..self.prefix]
    }

    fn lead(&self) -> &[Step] {
        &self.script.steps[self.prefix..self.target.index()]
    }

    /// Every role's operation count while the target runs.
    fn measure(&self, execution: &ExecutionRequest<'_, '_>) -> Vec<(MediaOperationRole, u64)> {
        let prepared = PreparedHome::prepare(self.prefix(), execution);
        let (home, activations) = media_faults::open_with_faults(
            prepared.location(),
            served_schema(self.prefix()),
            &media_faults::counters(),
            execution,
        )
        .expect("fail-before is admissible for every role");
        apply(&home, execution, self.lead());
        media_faults::arm(&activations);
        match self.target {
            Target::Commit(index) => {
                self.script.steps[index]
                    .commit(home.runtime(), execution)
                    .expect("the measured commit");
            }
            Target::Checkpoint(_) => {
                home.checkpoint(ExecutionAllocationPolicy::SystemAllocation)
                    .expect("the measured checkpoint");
            }
        }
        let counts = media_faults::counts(&activations);
        home.close();
        counts
    }
}

fn sweep(select: impl Fn(&Script) -> Target) {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let seeds: Vec<_> = SEEDS
        .iter()
        .map(|seed| SeedSweep::new(*seed, &select))
        .collect();
    let mut cases = Vec::new();
    for (index, sweep) in seeds.iter().enumerate() {
        let counts = sweep.measure(&execution);
        assert!(!counts.is_empty(), "seed {:#x}: no media", sweep.seed);
        for (role, count) in counts {
            for nth in 1..=count {
                for directive in media_faults::directives() {
                    cases.push((index, (role, NonZeroU64::new(nth).unwrap(), directive)));
                }
            }
        }
    }
    let next = AtomicUsize::new(0);
    let failures = Mutex::new(Vec::new());
    let tallies = Mutex::new(seeds.iter().map(|_| Tally::default()).collect::<Vec<_>>());
    std::thread::scope(|scope| {
        for _ in 0..WORKERS {
            scope.spawn(|| {
                let request = workload::serial_request();
                let execution = ExecutionRequest::serial(&request);
                while let Some((index, fault)) = cases.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let sweep = &seeds[*index];
                    let case = format!(
                        "seed {:#x} step {} {:?}#{} {:?}",
                        sweep.seed,
                        sweep.target.index(),
                        fault.0,
                        fault.1,
                        fault.2
                    );
                    // A failing case is recorded and the sweep goes on, so
                    // one run reports every failing case.
                    let ended = panic::catch_unwind(AssertUnwindSafe(|| {
                        run_case(
                            &sweep.script,
                            sweep.prefix(),
                            sweep.lead(),
                            sweep.target,
                            fault.clone(),
                            &execution,
                            &case,
                        )
                    }));
                    match ended {
                        Ok(ended) => tallies.lock().unwrap()[*index].add(ended),
                        Err(_) => failures.lock().unwrap().push(case),
                    }
                }
            });
        }
    });
    for (sweep, tally) in seeds.iter().zip(tallies.into_inner().unwrap()) {
        eprintln!(
            "media sweep seed {:#x} step {}: {tally:?}",
            sweep.seed,
            sweep.target.index()
        );
        assert!(tally.cases > 0, "seed {:#x}: no case ran", sweep.seed);
    }
    let failures = failures.into_inner().unwrap();
    assert!(
        failures.is_empty(),
        "{} of {} cases failed:\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
}

fn first_commit(script: &Script) -> Target {
    let index = script.steps.iter().position(Step::is_commit).unwrap();
    Target::Commit(index)
}

fn first_feature_commit(script: &Script) -> Target {
    let fork = script
        .steps
        .iter()
        .position(|step| matches!(step, Step::Fork { .. }))
        .expect("the script forks");
    Target::Commit(fork + 1)
}

fn transition_commit(script: &Script) -> Target {
    let index = script
        .steps
        .iter()
        .position(|step| matches!(step, Step::Transition { .. }))
        .expect("the script transitions");
    Target::Commit(index)
}

/// The script's second checkpoint, which replaces a selected one.
fn replacement_checkpoint(script: &Script) -> Target {
    let index = script
        .steps
        .iter()
        .enumerate()
        .filter(|(_, step)| matches!(step, Step::Checkpoint))
        .nth(1)
        .expect("the script checkpoints twice")
        .0;
    Target::Checkpoint(index)
}

#[test]
fn every_media_operation_of_the_first_commit_recovers() {
    sweep(first_commit);
}

#[test]
fn every_media_operation_of_the_first_commit_on_a_fork_recovers() {
    sweep(first_feature_commit);
}

#[test]
fn every_media_operation_of_a_schema_transition_commit_recovers() {
    sweep(transition_commit);
}

#[test]
fn every_media_operation_of_a_checkpoint_replacement_recovers() {
    sweep(replacement_checkpoint);
}
