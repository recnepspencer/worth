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
//! `WORTH_DST_SEEDS` (default 64) sets how many seeds run, `WORTH_DST_SEED`
//! one seed, and `WORTH_DST_GROUP_TRACE_DIR` writes each seed's NDJSON trace
//! for `run_tlc.py trace PersistentNamingGroupsTrace.tla`.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

mod driver;
mod model;
mod trace;

use driver::Driver;
use model::{Model, Op};

use super::persistent_naming_dst::Rng;

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
    let mut rng = Rng::new(seed ^ 0x6772_6f75_7073);
    let mut trace = trace::Trace::default();
    let mut coverage = BTreeMap::new();
    let mut ops = Vec::new();
    for step in 0..STEPS {
        let Some(op) = model.next_op(&mut rng) else {
            break;
        };
        ops.push(op.clone());
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
        for kind in trace::KINDS {
            assert!(
                total.get(kind).copied().unwrap_or(0) > 0,
                "the seeds never exercised {kind}"
            );
        }
    }
}
