//! Media sweep: every identified media operation of a home commit and of a
//! home checkpoint, faulted once each with `FailBefore` and
//! `IndeterminateAfterEffect`.
//!
//! For each seed and target the sweep counts every role's operations while
//! the target runs on a freshly prepared home holding the script before it,
//! then runs one case per (role, ordinal, directive): every single cut executes
//! fault injection, recovery, and full crash/transition oracle assertions,
//! while deduplicating reopen-based `assert_continues_world` tails once per
//! distinct byte-exact `PostRecoveryEquivalenceKey` (and running `RegionLoad`
//! continuation directly on the first recovered owner).

pub(crate) use std::num::NonZeroU64;
pub(crate) use std::panic::{self, AssertUnwindSafe};
pub(crate) use std::sync::atomic::{AtomicUsize, Ordering};
pub(crate) use std::sync::{Arc, Mutex};

pub(crate) use worth_execution::ExecutionRequest;
pub(crate) use worth_store::physical_runtime::certification::MediaOperationRole;

pub(crate) use crate::durable_run::served_schema;
pub(crate) use crate::media_case::{Ended, Target};
pub(crate) use crate::media_faults;
pub(crate) use crate::media_sweep::equivalence::EquivalenceLedger;
pub(crate) use crate::media_sweep::prepared_sweep_home::PreparedSweepHome;
pub(crate) use crate::media_sweep::targets::PhaseTimings;
pub(crate) use crate::media_sweep::world_case::run_sweep_case;
pub(crate) use crate::media_sweep::world_continuation::{
    prepare_native_target_home, prepare_world_checkpoint_home,
};
pub(crate) use crate::media_sweep::Tally;
pub(crate) use crate::workload::{self, Script, Step};

pub(crate) const SEEDS: [u64; 2] = [0x5eed_0301, 0x5eed_0302];
pub(crate) const COMMITS: usize = 9;
pub(crate) const CHECKPOINT_EVERY: usize = 3;

/// Libtest runs target sweeps concurrently. Keep that from multiplying the
/// worker allowance and starving process-kill children of their Store seam.
pub(crate) static SWEEP_EXECUTION: Mutex<()> = std::sync::Mutex::new(());

/// One seed's script and the steps around its target.
pub(crate) struct SeedSweep {
    pub(crate) seed: u64,
    pub(crate) script: Arc<Script>,
    pub(crate) target: Target,
    /// Steps before the target that a closed home holds.
    pub(crate) prefix: usize,
    /// Canonical commit artifacts held by the closed prefix (verified during measure).
    pub(crate) prefix_artifacts: Vec<Vec<u8>>,
    /// Durable branch-transition artifacts held by the closed prefix.
    pub(crate) prefix_transitions: Vec<Vec<u8>>,
    /// Selected checkpoint generation held by the closed prefix.
    pub(crate) prefix_checkpoint_generation: u64,
}

impl SeedSweep {
    pub(crate) fn from_script(
        seed: u64,
        script: Arc<Script>,
        select: &dyn Fn(&Script) -> Target,
    ) -> Self {
        let target = select(&script);
        // Keep the first-checkpoint sweep starting from an empty prefix so it
        // exercises fresh member emission from WAL without a prior checkpoint.
        let prefix = match target {
            Target::Checkpoint(index)
                if !script.steps[..index]
                    .iter()
                    .any(|step| matches!(step, Step::Checkpoint | Step::Transition { .. })) =>
            {
                0
            }
            _ => target.index(),
        };
        let prefix_commits = script.steps[..prefix]
            .iter()
            .filter(|step| step.is_commit())
            .count();
        let prefix_artifacts = script.artifacts[..prefix_commits].to_vec();
        Self {
            seed,
            script,
            target,
            prefix,
            prefix_artifacts,
            prefix_transitions: Vec::new(),
            prefix_checkpoint_generation: 0,
        }
    }

    pub(crate) fn prefix(&self) -> &[Step] {
        &self.script.steps[..self.prefix]
    }

    pub(crate) fn lead(&self) -> &[Step] {
        &self.script.steps[self.prefix..self.target.index()]
    }

    /// Compare the untouched freshly prepared prefix schedule with the
    /// differentially restored prefix's complete role/ordinal matrix, then
    /// capture prefix baseline metadata so worker 0 reuses this prepared home.
    pub(crate) fn measure_with_home(
        &mut self,
        prepared: &mut PreparedSweepHome,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Vec<(MediaOperationRole, u64)> {
        let before = self.measure_at(prepared.location(), execution);
        prepared.restore();
        let after = self.measure_at(prepared.location(), execution);
        assert_eq!(before, after, "restored prefix changed the media schedule");
        prepared.restore();
        {
            let native = crate::durable_run::open_declaring(
                prepared.location(),
                served_schema(self.prefix()),
                execution,
            )
            .expect("prepared prefix opens cleanly for baseline capture");
            self.prefix_checkpoint_generation = crate::selected_checkpoint::generation(&native);
            self.prefix_transitions = crate::branch_lifecycle::artifacts_of_home(&native);
            drop(native);
            prepared.restore();
        }
        before
    }

    pub(crate) fn measure_at(
        &self,
        location: &std::path::Path,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Vec<(MediaOperationRole, u64)> {
        let media = crate::media_faults::FaultedMedia::admit(&crate::media_faults::counters())
            .expect("fail-before is admissible for every role");
        let schema = served_schema(self.prefix());
        if matches!(self.target, Target::Checkpoint(_)) {
            let (fixture, activations, _, _) =
                prepare_world_checkpoint_home(self, location, schema, media, execution);
            crate::media_faults::arm(&activations);
            fixture
                .home
                .checkpoint(execution)
                .expect("the measured checkpoint");
            let counts = crate::media_faults::counts(&activations);
            let observed =
                crate::workload::artifacts_of(execution, fixture.home.relational().runtime());
            assert_eq!(
                &observed[..self.prefix_artifacts.len()],
                self.prefix_artifacts.as_slice(),
                "seed {:#x}: prepared prefix commit artifacts diverge from reference",
                self.seed
            );
            drop(fixture);
            return counts;
        }
        let (home, activations, _, _, _) =
            prepare_native_target_home(self, location, schema, media, execution);
        crate::media_faults::arm(&activations);
        match self.target {
            Target::Commit(index) => {
                self.script.steps[index]
                    .commit(home.relational().runtime(), execution)
                    .expect("the measured commit");
            }
            Target::RegionLoad(_) => {
                crate::residency_media::read_first(&home, &self.script, execution).unwrap();
            }
            Target::BranchTransition(index) => {
                self.script.steps[index]
                    .lifecycle(home.relational().runtime(), execution)
                    .expect("the measured transition");
            }
            Target::Checkpoint(_) => unreachable!(),
        }
        let counts = crate::media_faults::counts(&activations);
        let observed = crate::workload::artifacts_of(execution, home.relational().runtime());
        assert_eq!(
            &observed[..self.prefix_artifacts.len()],
            self.prefix_artifacts.as_slice(),
            "seed {:#x}: prepared prefix commit artifacts diverge from reference",
            self.seed
        );
        // Deliberate abrupt drop: bypasses close checkpoint after measurement.
        drop(home);
        counts
    }
}

pub(crate) fn sweep(select: impl Fn(&Script) -> Target) {
    let _workers = SWEEP_EXECUTION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let started = std::time::Instant::now();
    let mut seeds: Vec<SeedSweep> = SEEDS
        .iter()
        .map(|seed| {
            let script = Arc::new(Script::generate(*seed, COMMITS, CHECKPOINT_EVERY));
            SeedSweep::from_script(*seed, script, &select)
        })
        .collect();

    // Prepare worker 0's fresh home for each seed in parallel and measure the
    // exact media operation schedules before and after differential restore.
    let (mut seed_homes, schedules): (
        Vec<Option<PreparedSweepHome>>,
        Vec<Vec<(MediaOperationRole, u64)>>,
    ) = std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .iter_mut()
            .map(|seed_sweep| {
                scope.spawn(move || {
                    let request = crate::workload::serial_request();
                    let execution = ExecutionRequest::serial(&request);
                    let mut prepared = PreparedSweepHome::prepare(seed_sweep.prefix(), &execution);
                    let counts = seed_sweep.measure_with_home(&mut prepared, &execution);
                    assert!(!counts.is_empty(), "seed {:#x}: no media", seed_sweep.seed);
                    (Some(prepared), counts)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .unzip()
    });

    let seed_cases: Vec<Vec<crate::media_faults::Fault>> = schedules
        .iter()
        .map(|counts| {
            let mut list = Vec::new();
            for &(role, count) in counts {
                for nth in 1..=count {
                    for directive in crate::media_faults::directives() {
                        list.push((role, NonZeroU64::new(nth).unwrap(), directive));
                    }
                }
            }
            list
        })
        .collect();
    let total_scheduled: usize = seed_cases.iter().map(Vec::len).sum();
    let nexts: Vec<AtomicUsize> = seeds.iter().map(|_| AtomicUsize::new(0)).collect();
    let failures = Mutex::new(Vec::new());
    let tallies: Vec<Mutex<Tally>> = seeds.iter().map(|_| Mutex::new(Tally::default())).collect();
    let ledgers: Vec<EquivalenceLedger> =
        seeds.iter().map(|_| EquivalenceLedger::default()).collect();
    let timings = PhaseTimings::default();
    let mut total_active_workers = 0usize;

    std::thread::scope(|scope| {
        for (seed_index, seed_sweep) in seeds.iter().enumerate() {
            let cases = &seed_cases[seed_index];
            let next = &nexts[seed_index];
            let tally = &tallies[seed_index];
            let ledger = &ledgers[seed_index];
            let active =
                crate::media_sweep::workers_per_seed(SEEDS.len(), cases.len(), seed_sweep.prefix);
            total_active_workers += active;
            let mut first_home = seed_homes[seed_index].take();
            for _ in 0..active {
                let mut worker_home = first_home.take();
                let failures = &failures;
                let timings = &timings;
                scope.spawn(move || {
                    let request = crate::workload::serial_request();
                    let execution = ExecutionRequest::serial(&request);
                    loop {
                        let Some(fault) = cases.get(next.fetch_add(1, Ordering::Relaxed)) else {
                            break;
                        };
                        let case = format!(
                            "seed {:#x} step {} {:?}#{} {:?}",
                            seed_sweep.seed,
                            seed_sweep.target.index(),
                            fault.0,
                            fault.1,
                            fault.2
                        );
                        let ended = panic::catch_unwind(AssertUnwindSafe(|| {
                            let Some(media) = crate::media_faults::FaultedMedia::admit(
                                std::slice::from_ref(fault),
                            ) else {
                                return Ended::Skipped;
                            };
                            let tp = std::time::Instant::now();
                            let prepared = worker_home.get_or_insert_with(|| {
                                PreparedSweepHome::prepare(seed_sweep.prefix(), &execution)
                            });
                            timings
                                .prep_us
                                .fetch_add(tp.elapsed().as_micros() as u64, Ordering::Relaxed);
                            let tr = std::time::Instant::now();
                            prepared.restore();
                            timings
                                .restore_us
                                .fetch_add(tr.elapsed().as_micros() as u64, Ordering::Relaxed);
                            run_sweep_case(
                                seed_sweep, prepared, media, &execution, ledger, timings, &case,
                            )
                        }));
                        match ended {
                            Ok(ended) => tally.lock().unwrap().add(ended),
                            Err(_) => {
                                // Discard the worker's fixture in case the panic left it
                                // mid-mutation, then keep draining the remaining cuts so
                                // every failing cut across both seeds is reported.
                                worker_home = None;
                                failures.lock().unwrap().push(case);
                            }
                        }
                    }
                });
            }
        }
    });

    let failures = failures.into_inner().unwrap();
    assert!(
        failures.is_empty(),
        "{} of {} cases failed:\n{}",
        failures.len(),
        total_scheduled,
        failures.join("\n")
    );
    for (index, (seed_sweep, tally_lock)) in seeds.iter().zip(tallies).enumerate() {
        let tally = tally_lock.into_inner().unwrap();
        let eq = ledgers[index].stats();
        eprintln!(
            "media sweep seed {:#x} step {}: {tally:?}, eq_classes={} (total={}, deduped={})",
            seed_sweep.seed,
            seed_sweep.target.index(),
            eq.distinct_classes,
            eq.total_cases,
            eq.continuations_deduped
        );
        assert!(tally.cases > 0, "seed {:#x}: no case ran", seed_sweep.seed);
        let before: u64 = schedules[index].iter().map(|(_, count)| count * 2).sum();
        let after = tally.cases + tally.skipped;
        assert_eq!(
            after as u64, before,
            "seed {:#x}: lost cases",
            seed_sweep.seed
        );
        assert_eq!(
            eq.total_cases, tally.cases,
            "seed {:#x}: every executed cut must be accounted for in the equivalence ledger",
            seed_sweep.seed
        );
        assert!(
            eq.distinct_classes > 0,
            "seed {:#x}: at least one continuation must execute",
            seed_sweep.seed
        );
    }
    eprintln!(
        "media sweep: {} scheduled cases, {total_active_workers} workers, elapsed {:?}, {}",
        total_scheduled,
        started.elapsed(),
        timings.summary()
    );
}
