//! The seeded deterministic simulation of runtime group persistent names
//! (Phase 5E, Slice 3C-2, #334) under `Anchor` continuity.
//!
//! Each `u64` seed drives real installed commits: loop births and revivals
//! around their hosts, splits keeping the host's piece, a host deleted while
//! the rest re-partition (an ended loop with two successors), merges kept by
//! the least host, dissolutions, member deletions that shrink their loop
//! undeclared, host deletions whose remainder is born as the successor, and
//! typed `OverlappingMember` refusals with zero effects. After every step
//! one tracked decision reads every live member's loop and resolves every
//! loop name ever born, `Exact` and `AllDescendants`, and must agree 100%
//! with the reference model transcribed from
//! `specs/tla/persistent_naming/PersistentNamingGroups.tla`.
//!
//! About one step in three instead runs as an edit envelope (#361): its
//! first stage applies the action (or a field edit) to the unpublished
//! candidate and its second stage reads a member's loop and resolves one
//! loop there. Every read must equal Query's candidate rule transcribed from
//! `PersistentNamingGroupsCandidate.tla` (`candidate`), the envelope must
//! commit exactly when every read answered, and each answer must then equal
//! the committed world.
//!
//! `WORTH_DST_SEEDS` (default 64) sets how many seeds run, `WORTH_DST_SEED`
//! one seed, and `WORTH_DST_GROUP_TRACE_DIR` writes each seed's NDJSON trace
//! for `run_tlc.py trace PersistentNamingGroupsTrace.tla`.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use worth_dst::{Rng, Seed};

mod candidate;
mod driver;
mod model;
mod trace;

use driver::Driver;
use model::{Model, Op};

const STEPS: usize = 10;
const MEMBERS: usize = 6;

fn agree(model: &Model, observed: &driver::Observed) -> Result<(), String> {
    for (m, owner) in &observed.owners {
        if model.owner(*m) != *owner {
            return Err(format!(
                "m{} is in {owner:?}, the model says {:?}",
                m + 1,
                model.owner(*m)
            ));
        }
    }
    for ((g, all), answer) in &observed.answers {
        let expected = model.resolve(*g, *all);
        if *answer != expected {
            return Err(format!(
                "g{} (all: {all}) answered {answer:?}, the model says {expected:?}",
                g + 1
            ));
        }
    }
    Ok(())
}

fn run_seed(
    seed: u64,
    trace_dir: Option<&std::path::Path>,
) -> Result<BTreeMap<&'static str, usize>, String> {
    let mut driver = Driver::new(seed, MEMBERS);
    let rank = driver.members.iter().map(|name| *name.as_bytes()).collect();
    let mut model = Model::new(rank);
    let mut rng = Seed::new(seed ^ 0x6772_6f75_7073).rng();
    let mut trace = trace::Trace::default();
    let mut coverage = BTreeMap::new();
    let mut ops = Vec::new();
    for step in 0..STEPS {
        let Some(op) = model.next_op(&mut rng) else {
            break;
        };
        if rng.below(3) == 0 {
            let first = candidate_first(op, &model, &mut rng);
            ops.push(format!("envelope {first:?}"));
            let fail = |cause: String| format!("seed {seed} step {step}: {cause}\n  ops: {ops:?}");
            let outcome = envelope(&mut driver, &mut model, &first, &mut rng).map_err(fail)?;
            for kind in trace::candidate_kinds(&first, &outcome.1) {
                *coverage.entry(kind).or_default() += 1;
            }
            let observed = driver.observe(&model).map_err(fail)?;
            agree(&model, &observed).map_err(fail)?;
            trace.candidate(&first, &outcome, &model, &observed);
            continue;
        }
        ops.push(format!("{op:?}"));
        let fail = |cause: String| format!("seed {seed} step {step}: {cause}\n  ops: {ops:?}");
        *coverage.entry(trace::kind(&op)).or_default() += 1;
        let before_anchor = match &op {
            Op::SplitAway { g, .. } | Op::DeleteAnchor { g, .. } => Some(model.groups[g].anchor),
            _ => None,
        };
        driver.execute(&op, &model).map_err(fail)?;
        model.apply(&op);
        driver.learn(&op, &model).map_err(fail)?;
        let observed = driver.observe(&model).map_err(fail)?;
        agree(&model, &observed).map_err(fail)?;
        trace.step(&op, before_anchor, &model, &observed);
    }
    if let Some(dir) = trace_dir {
        trace
            .write(dir, seed)
            .map_err(|cause| format!("trace: {cause}"))?;
    }
    Ok(coverage)
}

/// The envelope's first stage: the chosen action, or a field edit of a
/// live member in place of an overlap refusal (which never reaches a read).
fn candidate_first(op: Op, model: &Model, rng: &mut Rng) -> candidate::First {
    let live: Vec<usize> = (0..MEMBERS).filter(|m| model.live_member[*m]).collect();
    match op {
        Op::RefuseOverlap { m } => candidate::First::Edit(m),
        op if live.is_empty() || rng.below(4) != 0 => candidate::First::Op(op),
        _ => candidate::First::Edit(rng.pick(&live)),
    }
}

/// Runs `first` and a later reading stage as one envelope, checks every
/// read against the candidate rule, and on commit applies `first` and
/// checks every answer against the committed model.
fn envelope(
    driver: &mut Driver,
    model: &mut Model,
    first: &candidate::First,
    rng: &mut Rng,
) -> Result<(Vec<candidate::Read>, Vec<model::Answer>, bool), String> {
    let mut reads = vec![candidate::Read::MemberOf(rng.below(MEMBERS))];
    let labels: Vec<usize> = model.groups.keys().copied().collect();
    if !labels.is_empty() {
        let g = rng.pick(&labels);
        reads.extend([
            candidate::Read::Resolve(g, false),
            candidate::Read::Resolve(g, true),
        ]);
    }
    let expected: Vec<_> = reads
        .iter()
        .map(|read| candidate::expected(model, first, *read))
        .collect();
    let (committed, answers) = driver.envelope(first, &reads, model)?;
    if answers != expected {
        return Err(format!(
            "{first:?} {reads:?} answered {answers:?}, the rule says {expected:?}"
        ));
    }
    if committed == answers.iter().any(candidate::refused) {
        return Err(format!(
            "{first:?} {reads:?} {answers:?}: committed {committed}"
        ));
    }
    if committed {
        if let candidate::First::Op(op) = first {
            model.apply(op);
            driver.learn(op, model)?;
        }
        for (read, answer) in reads.iter().zip(&answers) {
            let truth = candidate::truth(model, *read);
            if truth != *answer {
                return Err(format!(
                    "{read:?} answered {answer:?}, the commit made {truth:?}"
                ));
            }
        }
    }
    Ok((reads, answers, committed))
}

fn seeds() -> Vec<u64> {
    let var = |name: &str| std::env::var(name).ok().and_then(|v| v.parse::<u64>().ok());
    match var("WORTH_DST_SEED") {
        Some(seed) => vec![seed],
        None => (0..var("WORTH_DST_SEEDS").unwrap_or(64)).collect(),
    }
}

#[test]
fn seeded_group_walks_agree_with_the_reference_model() {
    let seeds = seeds();
    let trace_dir = std::env::var_os("WORTH_DST_GROUP_TRACE_DIR").map(std::path::PathBuf::from);
    let workers = std::thread::available_parallelism()
        .map_or(4, usize::from)
        .min(8)
        .min(seeds.len());
    let next = AtomicUsize::new(0);
    let failures = Mutex::new(Vec::new());
    let total = Mutex::new(BTreeMap::<&'static str, usize>::new());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                while let Some(&seed) = seeds.get(next.fetch_add(1, Ordering::Relaxed)) {
                    match run_seed(seed, trace_dir.as_deref()) {
                        Ok(coverage) => {
                            let mut total = total.lock().unwrap();
                            for (kind, runs) in coverage {
                                *total.entry(kind).or_default() += runs;
                            }
                        }
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
    let total = total.into_inner().unwrap();
    eprintln!(
        "persistent-naming group DST: {} seeds x {STEPS} steps, {total:?}",
        seeds.len()
    );
    if seeds.len() >= 64 {
        for kind in trace::KINDS.iter().chain(&trace::CANDIDATE_KINDS) {
            assert!(
                total.get(kind).copied().unwrap_or(0) > 0,
                "the seeds never exercised {kind}"
            );
        }
    }
}
