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

pub(crate) use {std::num::NonZeroU64};
pub(crate) use {std::panic, std::panic::AssertUnwindSafe};
pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering};
pub(crate) use {std::sync::Mutex};

pub(crate) use {worth_execution::ExecutionRequest};
pub(crate) use {worth_store::physical_runtime::certification::MediaOperationRole};

pub(crate) use {crate::durable_run::apply, crate::durable_run::served_schema};
pub(crate) use {crate::media_case::run_case, crate::media_case::Ended, crate::media_case::Target};
pub(crate) use {crate::media_faults};
pub(crate) use {crate::workload, crate::workload::Script, crate::workload::Step};


pub(crate) use {crate::media_sweep::prepared_sweep_home::PreparedSweepHome};

pub(crate) const SEEDS: [u64; 2] = [0x5eed_0301, 0x5eed_0302];
pub(crate) const COMMITS: usize = 9;
pub(crate) const CHECKPOINT_EVERY: usize = 3;
/// Retain at least eight workers because timeout budgets were derived at eight.
/// Cap simultaneous homes at 32 to bound NTFS/endpoint-scanner contention and
/// the fixture memory footprint.
pub(crate) fn worker_count() -> usize {
    std::thread::available_parallelism()
        .map_or(8, std::num::NonZeroUsize::get)
        .clamp(8, 32)
}
/// Libtest runs target sweeps concurrently. Keep that from multiplying the
/// worker allowance and starving process-kill children of their Store seam.
pub(crate) static SWEEP_EXECUTION: Mutex<()> = std::sync::Mutex::new(());

/// How many cases ran and how the target ended in them.
#[derive(Default, Debug)]
pub(crate) struct Tally {
    pub(crate) cases: usize,
    pub(crate) skipped: usize,
    pub(crate) acknowledged: usize,
    pub(crate) refused: usize,
    pub(crate) unresolved_absent: usize,
    pub(crate) unresolved_recovered: usize,
}

impl Tally {
    pub(crate) fn add(&mut self, ended: Ended) {
        match ended {
            crate::media_case::Ended::Skipped => self.skipped += 1,
            crate::media_case::Ended::Acknowledged => self.acknowledged += 1,
            crate::media_case::Ended::Refused => self.refused += 1,
            crate::media_case::Ended::Unresolved { recovered: false } => self.unresolved_absent += 1,
            crate::media_case::Ended::Unresolved { recovered: true } => self.unresolved_recovered += 1,
        }
        if !matches!(ended, crate::media_case::Ended::Skipped) {
            self.cases += 1;
        }
    }
}

/// One seed's script and the steps around its target.
pub(crate) struct SeedSweep {
    pub(crate) seed: u64,
    pub(crate) script: Script,
    pub(crate) target: Target,
    /// Steps before the target that a closed home holds.
    pub(crate) prefix: usize,
}

impl SeedSweep {
    pub(crate) fn new(seed: u64, select: &impl Fn(&Script) -> Target) -> Self {
        let script = crate::workload::Script::generate(seed, COMMITS, CHECKPOINT_EVERY);
        let target = select(&script);
        let prefix = target.index();
        Self {
            seed,
            script,
            target,
            prefix,
        }
    }

    pub(crate) fn prefix(&self) -> &[Step] {
        &self.script.steps[..self.prefix]
    }

    pub(crate) fn lead(&self) -> &[Step] {
        &self.script.steps[self.prefix..self.target.index()]
    }

    /// Compare the original fresh-prefix schedule with the restored prefix's
    /// complete role/ordinal matrix before scheduling either directive.
    pub(crate) fn measure(&self, execution: &ExecutionRequest<'_, '_>) -> Vec<(MediaOperationRole, u64)> {
        let mut prepared = crate::media_sweep::prepared_sweep_home::PreparedSweepHome::prepare(self.prefix(), execution);
        let before = self.measure_at(prepared.location(), execution);
        prepared.restore();
        let after = self.measure_at(prepared.location(), execution);
        assert_eq!(before, after, "restored prefix changed the media schedule");
        before
    }

    pub(crate) fn measure_at(
        &self,
        location: &std::path::Path,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Vec<(MediaOperationRole, u64)> {
        let (home, activations) = crate::media_faults::open_with_faults(
            location,
            served_schema(self.prefix()),
            &crate::media_faults::counters(),
            execution,
        )
        .expect("fail-before is admissible for every role");
        apply(&home, execution, self.lead());
        if matches!(self.target, crate::media_case::Target::BranchTransition(_)) {
            let _ = crate::branch_lifecycle::state_of(home.runtime(), &self.script.records);
        }
        let _ = crate::branch_lifecycle::artifacts_of_home(&home);
        if matches!(self.target, crate::media_case::Target::RegionLoad(_)) {
            crate::residency_media::prepare_cold_media_read(&home);
        }
        crate::media_faults::arm(&activations);
        match self.target {
            crate::media_case::Target::Commit(index) => {
                self.script.steps[index]
                    .commit(home.runtime(), execution)
                    .expect("the measured commit");
            }
            crate::media_case::Target::Checkpoint(_) => {
                home.checkpoint(execution).expect("the measured checkpoint");
            }
            crate::media_case::Target::RegionLoad(_) => {
                crate::residency_media::read_first(&home, &self.script, execution).unwrap();
            }
            crate::media_case::Target::BranchTransition(index) => {
                self.script.steps[index]
                    .lifecycle(home.runtime(), execution)
                    .expect("the measured transition");
            }
        }
        let counts = crate::media_faults::counts(&activations);
        home.close();
        counts
    }
}

pub(crate) fn sweep(select: impl Fn(&Script) -> Target) {
    let _workers = SWEEP_EXECUTION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let started = std::time::Instant::now();
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let seeds: Vec<_> = SEEDS
        .iter()
        .map(|seed| crate::media_sweep::SeedSweep::new(*seed, &select))
        .collect();
    let mut cases = Vec::new();
    let mut schedules = Vec::new();
    for (index, sweep) in seeds.iter().enumerate() {
        let counts = sweep.measure(&execution);
        assert!(!counts.is_empty(), "seed {:#x}: no media", sweep.seed);
        for &(role, count) in &counts {
            for nth in 1..=count {
                for directive in crate::media_faults::directives() {
                    cases.push((index, (role, std::num::NonZeroU64::new(nth).unwrap(), directive)));
                }
            }
        }
        schedules.push(counts);
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let workers = worker_count();
    let failures = std::sync::Mutex::new(Vec::new());
    let tallies = std::sync::Mutex::new(seeds.iter().map(|_| crate::media_sweep::Tally::default()).collect::<Vec<_>>());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                let request = crate::workload::serial_request();
                let execution = worth_execution::ExecutionRequest::serial(&request);
                let mut prepared: Vec<Option<PreparedSweepHome>> =
                    seeds.iter().map(|_| None).collect();
                while let Some((index, fault)) = cases.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed)) {
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
                    let ended = std::panic::catch_unwind(AssertUnwindSafe(|| {
                        // Keep inapplicable directives in the matrix, without
                        // preparing a home they would never open (as before).
                        let Some(media) =
                            crate::media_faults::FaultedMedia::admit(std::slice::from_ref(fault))
                        else {
                            return crate::media_case::Ended::Skipped;
                        };
                        let prepared = prepared[*index].get_or_insert_with(|| {
                            crate::media_sweep::prepared_sweep_home::PreparedSweepHome::prepare(sweep.prefix(), &execution)
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
    let failures = failures.into_inner().unwrap();
    assert!(
        failures.is_empty(),
        "{} of {} cases failed:\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
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
    eprintln!(
        "media sweep: {} scheduled cases, {workers} workers, elapsed {:?}",
        cases.len(),
        started.elapsed()
    );
}

pub(crate) fn first_commit(script: &Script) -> Target {
    let index = script.steps.iter().position(crate::workload::Step::is_commit).unwrap();
    crate::media_case::Target::Commit(index)
}

pub(crate) fn first_feature_commit(script: &Script) -> Target {
    let fork = script
        .steps
        .iter()
        .position(|step| matches!(step, crate::workload::Step::Fork { .. }))
        .expect("the script forks");
    crate::media_case::Target::Commit(fork + 1)
}

pub(crate) fn transition_commit(script: &Script) -> Target {
    let index = script
        .steps
        .iter()
        .position(|step| matches!(step, crate::workload::Step::Transition { .. }))
        .expect("the script transitions");
    crate::media_case::Target::Commit(index)
}

/// The script's second checkpoint, which replaces a selected one.
pub(crate) fn replacement_checkpoint(script: &Script) -> Target {
    let index = script
        .steps
        .iter()
        .enumerate()
        .filter(|(_, step)| matches!(step, crate::workload::Step::Checkpoint))
        .nth(1)
        .expect("the script checkpoints twice")
        .0;
    crate::media_case::Target::Checkpoint(index)
}