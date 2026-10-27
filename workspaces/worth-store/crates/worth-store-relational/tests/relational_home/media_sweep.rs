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
//! Targets: the first commit, each of fork/archive/deletion-requested/delete,
//! the first commit on a durably forked branch, the schema-transition commit,
//! and a checkpoint that replaces a selected one; two seeds each.

use std::num::NonZeroU64;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::certification::MediaOperationRole;

use super::durable_run::{apply, served_schema};
use super::media_case::{run_case, Ended, Target};
use super::media_faults;
use super::workload::{self, Script, Step};

#[path = "prepared_sweep_home.rs"]
pub(super) mod prepared_sweep_home;
use prepared_sweep_home::PreparedSweepHome;

const SEEDS: [u64; 2] = [0x5eed_0301, 0x5eed_0302];
const COMMITS: usize = 9;
const CHECKPOINT_EVERY: usize = 3;
/// Keep small hosts busy through I/O waits, but cap simultaneous homes at 32
/// to bound NTFS/endpoint-scanner contention and the fixture memory footprint.
fn worker_count() -> usize {
    std::thread::available_parallelism()
        .map_or(4, std::num::NonZeroUsize::get)
        .clamp(4, 32)
}
/// Libtest runs target sweeps concurrently. Keep that from multiplying the
/// worker allowance and starving process-kill children of their Store seam.
static SWEEP_EXECUTION: Mutex<()> = Mutex::new(());

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
        let prefix = target.index();
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

    /// Compare the original fresh-prefix schedule with the restored prefix's
    /// complete role/ordinal matrix before scheduling either directive.
    fn measure(&self, execution: &ExecutionRequest<'_, '_>) -> Vec<(MediaOperationRole, u64)> {
        let mut prepared = PreparedSweepHome::prepare(self.prefix(), execution);
        let before = self.measure_at(prepared.location(), execution);
        prepared.restore();
        let after = self.measure_at(prepared.location(), execution);
        assert_eq!(before, after, "restored prefix changed the media schedule");
        before
    }

    fn measure_at(
        &self,
        location: &std::path::Path,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Vec<(MediaOperationRole, u64)> {
        let (home, activations) = media_faults::open_with_faults(
            location,
            served_schema(self.prefix()),
            &media_faults::counters(),
            execution,
        )
        .expect("fail-before is admissible for every role");
        apply(&home, execution, self.lead());
        if matches!(self.target, Target::BranchTransition(_)) {
            let _ = super::branch_lifecycle::state_of(home.runtime(), &self.script.records);
        }
        let _ = super::branch_lifecycle::artifacts_of_home(&home);
        if matches!(self.target, Target::RegionLoad(_)) {
            super::residency_media::prepare_cold_media_read(&home);
        }
        media_faults::arm(&activations);
        match self.target {
            Target::Commit(index) => {
                self.script.steps[index]
                    .commit(home.runtime(), execution)
                    .expect("the measured commit");
            }
            Target::Checkpoint(_) => {
                home.checkpoint(execution).expect("the measured checkpoint");
            }
            Target::RegionLoad(_) => {
                super::residency_media::read_first(&home, &self.script, execution).unwrap();
            }
            Target::BranchTransition(index) => {
                self.script.steps[index]
                    .lifecycle(home.runtime(), execution)
                    .expect("the measured transition");
            }
        }
        let counts = media_faults::counts(&activations);
        home.close();
        counts
    }
}

fn sweep(select: impl Fn(&Script) -> Target) {
    let _workers = SWEEP_EXECUTION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let started = std::time::Instant::now();
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let seeds: Vec<_> = SEEDS
        .iter()
        .map(|seed| SeedSweep::new(*seed, &select))
        .collect();
    let mut cases = Vec::new();
    let mut schedules = Vec::new();
    for (index, sweep) in seeds.iter().enumerate() {
        let counts = sweep.measure(&execution);
        assert!(!counts.is_empty(), "seed {:#x}: no media", sweep.seed);
        for &(role, count) in &counts {
            for nth in 1..=count {
                for directive in media_faults::directives() {
                    cases.push((index, (role, NonZeroU64::new(nth).unwrap(), directive)));
                }
            }
        }
        schedules.push(counts);
    }
    let next = AtomicUsize::new(0);
    let workers = worker_count();
    let failures = Mutex::new(Vec::new());
    let tallies = Mutex::new(seeds.iter().map(|_| Tally::default()).collect::<Vec<_>>());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                let request = workload::serial_request();
                let execution = ExecutionRequest::serial(&request);
                let mut prepared: Vec<Option<PreparedSweepHome>> =
                    seeds.iter().map(|_| None).collect();
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
                        // Keep inapplicable directives in the matrix, without
                        // preparing a home they would never open (as before).
                        let Some(media) =
                            media_faults::FaultedMedia::admit(std::slice::from_ref(fault))
                        else {
                            return Ended::Skipped;
                        };
                        let prepared = prepared[*index].get_or_insert_with(|| {
                            PreparedSweepHome::prepare(sweep.prefix(), &execution)
                        });
                        prepared.restore();
                        run_case(
                            &sweep.script,
                            prepared,
                            sweep.lead(),
                            sweep.target,
                            media,
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
    for (index, (sweep, tally)) in seeds.iter().zip(tallies.into_inner().unwrap()).enumerate() {
        eprintln!(
            "media sweep seed {:#x} step {}: {tally:?}",
            sweep.seed,
            sweep.target.index()
        );
        assert!(tally.cases > 0, "seed {:#x}: no case ran", sweep.seed);
        let before: u64 = schedules[index].iter().map(|(_, count)| count * 2).sum();
        let after = tally.cases + tally.skipped;
        assert_eq!(after as u64, before, "seed {:#x}: lost cases", sweep.seed);
        eprintln!(
            "media schedule seed {:#x}: before {before}, after {after}",
            sweep.seed
        );
    }
    let failures = failures.into_inner().unwrap();
    assert!(
        failures.is_empty(),
        "{} of {} cases failed:\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
    eprintln!(
        "media sweep: {} scheduled cases, {workers} workers, elapsed {:?}",
        cases.len(),
        started.elapsed()
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

#[test]
fn every_media_operation_of_a_branch_transition_recovers() {
    sweep(|script| {
        Target::BranchTransition(
            script
                .steps
                .iter()
                .position(|step| matches!(step, Step::Fork { .. }))
                .unwrap(),
        )
    });
}

#[test]
fn every_media_operation_of_archive_and_delete_transitions_recovers() {
    for select in [
        (|step: &Step| matches!(step, Step::Archive { .. })) as fn(&Step) -> bool,
        |step| matches!(step, Step::RequestDeletion { .. }),
        |step| matches!(step, Step::Delete { .. }),
    ] {
        sweep(|script| Target::BranchTransition(script.steps.iter().position(select).unwrap()));
    }
}

#[test]
fn every_media_operation_of_the_first_checkpoint_recovers() {
    sweep(|script| {
        Target::Checkpoint(
            script
                .steps
                .iter()
                .position(|step| matches!(step, Step::Checkpoint))
                .unwrap(),
        )
    });
}

#[test]
fn every_media_operation_of_a_cold_region_load_recovers() {
    sweep(|script| {
        Target::RegionLoad(
            script
                .steps
                .iter()
                .position(|step| matches!(step, Step::Checkpoint))
                .unwrap()
                + 1,
        )
    });
}
