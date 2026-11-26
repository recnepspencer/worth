//! The single-harness seeded deterministic simulation of persistent naming
//! (`milestones/persistent-naming.md`, formal model, DST & trace contract).
//!
//! Each `u64` seed drives real installed commits: first births (and typed
//! `DuplicateBirth` refusals of live re-mints), commits leaving a created
//! record of the schemed kind unnamed (typed `MissingBirth` refusals of the
//! completeness guard), in-commit regeneration (`Preserve`), splits whose
//! pieces two distinct models own (#133), merges keeping a parent in place
//! whose partition the commit never touches (#314), the same merges through
//! a naming scope's `merge_least` with the parents in seeded order, merges
//! minting a new name, retirement, edit envelopes whose two authored stages
//! each birth a name in one composed commit (and typed `DuplicateBirth`
//! refusals when both mint the same one), cross-commit `Restore` on a
//! strictly fresh native identity, candidate-basis name reads (Slice 3C-1:
//! an envelope whose later stage reads a name on the candidate an earlier
//! stage mutated, answered or refused typed with zero effects, including
//! the name the envelope gives a record that earlier stage created, #426),
//! and
//! destruction of the derived name index
//! (a fresh world rebuilt from the same canonical commits). After every step
//! every ever-minted name is resolved `Exact` and `AllDescendants`, unscoped
//! and within each owner model, and must agree 100% with the reference
//! oracle transcribed from `specs/tla/persistent_naming/PersistentNaming.tla`.
//!
//! `WORTH_DST_SEEDS` (default 64) sets how many seeds run, `WORTH_DST_SEED`
//! runs one seed, and `WORTH_DST_TRACE_DIR` writes each seed's NDJSON trace
//! for `tools/tla/run_tlc.py trace`. The durable sweep (`durable`) runs the
//! same walks on a Directory Home crashed and reopened after every step.
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

mod candidate;
mod driver;
mod durable;
mod durable_step;
mod observe;
mod ops;
pub(super) use ops::Rng;
mod reference;
mod trace;

use driver::Driver;
use observe::Observed;
use ops::{next_op, Op};
use reference::{Event, Oracle};
use trace::Trace;

/// Actions per seed.
const STEPS: usize = 10;

/// How often each protocol case ran.
#[derive(Clone, Copy, Debug, Default)]
struct Coverage {
    multi_owner_splits: usize,
    untouched_keepers: usize,
    preserves: usize,
    restores: usize,
    rebuilds: usize,
    duplicates: usize,
    unnamed: usize,
    least_merges: usize,
    minted_merges: usize,
    coupled: usize,
    coupled_duplicates: usize,
    candidate_answers: usize,
    candidate_ended: usize,
    candidate_history: usize,
    candidate_pending: usize,
    reopens: usize,
    checkpointed_reopens: usize,
}

impl Coverage {
    fn count(&mut self, op: &Op, untouched_keeper: bool) {
        if let Op::MergeKeep {
            listed: Some(_), ..
        } = op
        {
            self.least_merges += 1;
        }
        match op {
            Op::Split { at, .. } if at[0] != at[1] => self.multi_owner_splits += 1,
            Op::MergeKeep { .. } if untouched_keeper => self.untouched_keepers += 1,
            Op::Preserve { .. } => self.preserves += 1,
            Op::Restore { .. } => self.restores += 1,
            Op::Rebuild => self.rebuilds += 1,
            Op::Duplicate { .. } => self.duplicates += 1,
            Op::Unnamed { .. } => self.unnamed += 1,
            Op::MergeMint { .. } => self.minted_merges += 1,
            Op::Coupled { .. } => self.coupled += 1,
            Op::CoupledDuplicate { .. } => self.coupled_duplicates += 1,
            Op::Candidate {
                read: candidate::Read::NameOfCreated,
                expected,
                ..
            } if !candidate::is_refusal(expected) => self.candidate_pending += 1,
            Op::Candidate { expected, .. } => match expected.status {
                "RefusedEnded" => self.candidate_ended += 1,
                "RefusedHistory" => self.candidate_history += 1,
                _ => self.candidate_answers += 1,
            },
            _ => {}
        }
    }

    fn add(&mut self, other: Self) {
        self.multi_owner_splits += other.multi_owner_splits;
        self.untouched_keepers += other.untouched_keepers;
        self.preserves += other.preserves;
        self.restores += other.restores;
        self.rebuilds += other.rebuilds;
        self.duplicates += other.duplicates;
        self.unnamed += other.unnamed;
        self.least_merges += other.least_merges;
        self.minted_merges += other.minted_merges;
        self.coupled += other.coupled;
        self.coupled_duplicates += other.coupled_duplicates;
        self.candidate_answers += other.candidate_answers;
        self.candidate_ended += other.candidate_ended;
        self.candidate_history += other.candidate_history;
        self.candidate_pending += other.candidate_pending;
        self.reopens += other.reopens;
        self.checkpointed_reopens += other.checkpointed_reopens;
    }
}

/// Applies a committed action to the reference model.
fn apply(model: &mut Oracle, op: &Op) {
    let event = match op {
        Op::Create { name, at } => Some((Event::Create(*name), vec![at.owner()])),
        Op::Preserve { name, at } => Some((Event::Preserve(*name), vec![at.owner()])),
        Op::Restore { name, at } => Some((Event::Restore(*name), vec![at.owner()])),
        Op::Split { parent, pieces, at } => Some((
            Event::Split {
                parent: *parent,
                pieces: *pieces,
            },
            at.iter().map(|placement| placement.owner()).collect(),
        )),
        Op::MergeKeep { keeper, other, .. } => Some((
            Event::MergeKeep {
                keeper: *keeper,
                other: *other,
            },
            Vec::new(),
        )),
        Op::MergeMint {
            parents,
            minted,
            at,
        } => Some((
            Event::MergeMint {
                parents: *parents,
                minted: *minted,
            },
            vec![at.owner()],
        )),
        Op::Retire { name } => Some((Event::Retire(*name), Vec::new())),
        Op::Coupled { names, at } => {
            // Both stages' births land in the one commit, in stage order.
            model.apply(Event::Create(names[0]), &[at[0].owner()]);
            Some((Event::Create(names[1]), vec![at[1].owner()]))
        }
        Op::Duplicate { .. }
        | Op::Unnamed { .. }
        | Op::MergeLeast { .. }
        | Op::Rebuild
        | Op::CoupledDuplicate { .. }
        | Op::Candidate { .. } => None,
    };
    if let Some((event, owners)) = event {
        model.apply(event, &owners);
    }
}

/// Every observed answer must equal the oracle's, for every born name.
fn agree(model: &Oracle, observed: &Observed, minted: &BTreeSet<usize>) -> Result<(), String> {
    let born: BTreeSet<usize> = model.born().collect();
    if &born != minted {
        return Err(format!("the world minted {minted:?}, the model {born:?}"));
    }
    if observed.live != model.live() {
        return Err(format!(
            "live {:?}, model {:?}",
            observed.live,
            model.live()
        ));
    }
    let expected = born.len() * reference::SCOPES.len() * reference::SELECTIONS.len();
    if observed.answers.len() != expected {
        return Err(format!(
            "{} answers, {expected} expected",
            observed.answers.len()
        ));
    }
    for (&(name, scope, all), answer) in &observed.answers {
        let modelled = model.resolve(name, scope, all);
        if *answer != modelled {
            return Err(format!(
                "n{} in {scope:?} (all: {all}) answered {answer:?}, model {modelled:?}",
                name + 1
            ));
        }
    }
    Ok(())
}

fn run_seed(
    seed: u64,
    trace_dir: Option<&std::path::Path>,
    durable: bool,
) -> Result<Coverage, String> {
    let mut rng = Rng::new(seed);
    let mut model = Oracle::default();
    let mut driver = Driver::new(seed, durable);
    let mut authored = BTreeSet::new();
    let mut minted = 0;
    let mut trace = Trace::default();
    let mut coverage = Coverage::default();
    let mut ops = Vec::new();
    let steps = if durable { durable::steps() } else { STEPS };
    for step in 0..steps {
        let op = next_op(&mut rng, &model, &authored, minted, coverage.rebuilds > 0);
        let op = driver.settle(op);
        ops.push(op.clone());
        let fail = |cause: String| format!("seed {seed} step {step}: {cause}\n  ops: {ops:?}");
        let untouched = match &op {
            Op::MergeKeep { keeper, other, .. } => {
                driver.partitions_differ(*keeper, *other) == Some(true)
            }
            _ => false,
        };
        coverage.count(&op, untouched);
        let created = driver.execute(&op, step).map_err(fail)?;
        let recovered = durable && durable_step::settles_by_reopen(&op);
        if recovered {
            driver.reopen(false);
        }
        driver.learn(step, &created).map_err(fail)?;
        // A candidate envelope lands its first stage only when the read
        // answered (an edit lands no naming event); every other op lands.
        let landed = match &op {
            Op::Candidate {
                first: candidate::First::Commit(first),
                expected,
                ..
            } if !candidate::is_refusal(expected) => Some(first.as_ref()),
            Op::Candidate { .. } => None,
            op => Some(op),
        };
        if let Some(landed) = landed {
            apply(&mut model, landed);
            match landed {
                Op::Create { name, .. } => {
                    authored.insert(*name);
                    minted += 1;
                }
                Op::Split { .. } => minted += 2,
                Op::MergeMint { .. } => minted += 1,
                Op::Coupled { names, .. } => {
                    authored.extend(names.iter().copied());
                    minted += 2;
                }
                _ => {}
            }
        }
        let observed = driver.observe(step).map_err(fail)?;
        let born: BTreeSet<usize> = driver.names.keys().copied().collect();
        agree(&model, &observed, &born).map_err(fail)?;
        trace.step(&op, &observed, &born, driver.naming_commits);
        if durable {
            let (trace, coverage) = (&mut trace, &mut coverage);
            durable_step::reopen_after(
                &mut driver,
                &model,
                &born,
                &observed,
                step,
                recovered,
                trace,
                coverage,
            )
            .map_err(|cause| format!("seed {seed} reopen after step {step}: {cause}"))?;
        }
    }
    if let Some(dir) = trace_dir {
        let file = format!("{}seed-{seed}", if durable { "durable-" } else { "" });
        trace
            .write(dir, &file)
            .map_err(|cause| format!("trace: {cause}"))?;
    }
    Ok(coverage)
}

fn seeds() -> Vec<u64> {
    let var = |name: &str| std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok());
    match var("WORTH_DST_SEED") {
        Some(seed) => vec![seed],
        None => (0..var("WORTH_DST_SEEDS").unwrap_or(64)).collect(),
    }
}

/// Runs `seeds` across worker threads; every failure fails the court.
fn sweep(seeds: &[u64], durable: bool) -> Coverage {
    let trace_dir = std::env::var_os("WORTH_DST_TRACE_DIR").map(std::path::PathBuf::from);
    let workers = std::thread::available_parallelism()
        .map_or(4, usize::from)
        .min(8)
        .min(seeds.len());
    let next = AtomicUsize::new(0);
    let failures = Mutex::new(Vec::new());
    let total = Mutex::new(Coverage::default());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                while let Some(&seed) = seeds.get(next.fetch_add(1, Ordering::Relaxed)) {
                    match run_seed(seed, trace_dir.as_deref(), durable) {
                        Ok(coverage) => total.lock().unwrap().add(coverage),
                        Err(failure) => failures.lock().unwrap().push(failure),
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    assert!(
        failures.is_empty(),
        "{} seeds failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
    total.into_inner().unwrap()
}

#[test]
fn seeded_walks_agree_with_the_reference_oracle() {
    let seeds = seeds();
    let total = sweep(&seeds, false);
    eprintln!(
        "persistent-naming DST: {} seeds x {STEPS} steps, {total:?}",
        seeds.len()
    );
    if seeds.len() >= 64 {
        let Coverage {
            multi_owner_splits,
            untouched_keepers,
            preserves,
            restores,
            rebuilds,
            duplicates,
            unnamed,
            least_merges,
            minted_merges,
            coupled,
            coupled_duplicates,
            candidate_answers,
            candidate_ended,
            candidate_history,
            candidate_pending,
            reopens: _,
            checkpointed_reopens: _,
        } = total;
        for (case, runs) in [
            ("#133 multi-owner splits", multi_owner_splits),
            ("#314 untouched keepers", untouched_keepers),
            ("Preserve", preserves),
            ("Restore", restores),
            ("index rebuilds", rebuilds),
            ("DuplicateBirth refusals", duplicates),
            ("unnamed-birth refusals", unnamed),
            ("merge_least merges", least_merges),
            ("MintNew merges", minted_merges),
            ("coupled envelopes", coupled),
            ("coupled DuplicateBirth refusals", coupled_duplicates),
            ("answered candidate reads", candidate_answers),
            ("candidate reads of ended records", candidate_ended),
            ("candidate reads over name history", candidate_history),
            ("candidate reads of earlier-stage births", candidate_pending),
        ] {
            assert!(runs > 0, "the seeds never exercised {case}");
        }
    }
}
