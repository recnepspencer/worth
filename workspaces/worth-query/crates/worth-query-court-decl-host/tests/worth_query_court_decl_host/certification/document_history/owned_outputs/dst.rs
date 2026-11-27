//! The seeded deterministic simulation of the document history (Phase 1 and
//! Phase 2, public #53-#57): one harness, one reference oracle translated
//! from `specs/tla/tree_based_undo/TreeBasedUndo.tla`, agreement after every
//! call.
//!
//! - `WORTH_DST_SEEDS` (default 64), `WORTH_DST_SEED_BASE`, `WORTH_DST_OPS`
//!   (default 16) choose the sweep.
//! - `WORTH_DST_SCENARIO` (`branch-after-delete`, `replay`, `out-of-band`,
//!   `stop-recovery`) puts that scenario's scripted skeleton first in every
//!   seed (`workload::Scenario`); the seeded choices follow it.
//! - `WORTH_DST_TRACE_DIR` writes one NDJSON trace per seed for
//!   `run_tlc.py trace` against `TreeBasedUndoTrace.tla`.
//! - `WORTH_DST_CORRECTIONS=1` adds correction workloads to the document's
//!   sweep. Until the correction envelope exists (S2) they run through
//!   today's route, outside the document, and the oracle disagrees.
//! - A disagreement prints the seed and its shrunk choices; a panic in the
//!   real system names its seed.
mod corrective_cascade;
mod driver;
mod history_correction;
mod history_navigation;
mod history_retirement;
mod history_reversal_facts;
pub(super) mod fixture;
mod readmission;
mod reference;
mod regressions;
mod trace;
mod workload;
mod world;

use super::*;
use driver::{run, shrink, Failure, ModelSystem, System};
use reference::{Disagreement, Environment};
use std::panic::{catch_unwind, AssertUnwindSafe};
use workload::{generate, Op, Scenario, ENVIRONMENT};

const DEFAULT_SEED_BASE: u64 = 0x7b02_0000;

/// Document seeds held on a traced public cause, with the corrections
/// setting they fail under. Each is skipped by name, never silently.
/// `0x7b02000f` (corrections on): a `current_output` read that returned
/// `Missing` is not a verifiable consumer fact, so the chain keeps a stale
/// `dst-mid` once `dst-a` is readmitted after it (held on public #429).
/// `0x7b02162a` (corrections on): re-correcting a retirement after its
/// correction was undone is refused at Relational publication, "exact route
/// has no canonical publication" (held on public #450). `0x7b02206b`
/// (corrections on) refuses the same way, at the same succession site
/// (held on public #450).
/// `0x7b02205d` (corrections on): a retirement reversed after its
/// re-creation was corrected records, because the performed correction pair
/// cancels a re-creation the model counts as live on the line (held on
/// public #456).
/// `0x7b02125f` and `0x7b02181e` (corrections on): reversing a retirement
/// after the document's navigation revived and retired it again refuses
/// `IncompleteRestoration`, because native transition succession does not
/// follow the `HistoricalAbsence` the navigation wrote (held on public #465).
/// `WORTH_DST_RUN_HELD=1` runs the held seeds too, to trace one by name.
const HELD_DOCUMENT_SEEDS: [(u64, bool, &str); 6] = [
    (0x7b02_000f, true, "public #429"),
    (0x7b02_162a, true, "public #450"),
    (0x7b02_206b, true, "public #450"),
    (0x7b02_205d, true, "public #456"),
    (0x7b02_125f, true, "public #465"),
    (0x7b02_181e, true, "public #465"),
];

fn variable(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .map(|value| value.parse().unwrap_or_else(|_| panic!("{name} must be a number")))
        .unwrap_or(default)
}

/// The seeds of this sweep and the choices per seed.
fn sweep() -> (std::ops::Range<u64>, usize) {
    let base = variable("WORTH_DST_SEED_BASE", DEFAULT_SEED_BASE);
    let seeds = variable("WORTH_DST_SEEDS", 64);
    (base..base + seeds, variable("WORTH_DST_OPS", 16) as usize)
}

/// The scenario named by `WORTH_DST_SCENARIO`, if any.
fn scenario() -> Option<Scenario> {
    std::env::var("WORTH_DST_SCENARIO").ok().map(|name| Scenario::from_name(&name))
}

/// The workloads a sweep draws from: which environment refusals the system
/// can meet, and whether corrections run.
struct Workloads<'e> {
    environment: &'e [Environment],
    corrections: bool,
}

/// Run `seeds` on systems from `system`; on the first disagreement, shrink
/// it and fail with the seed and its shortest choices. Traces go to
/// `<WORTH_DST_TRACE_DIR>/<label>/`.
fn agree<S: System>(
    label: &str,
    seeds: std::ops::Range<u64>,
    workloads: Workloads<'_>,
    mut system: impl FnMut(u64) -> S,
) {
    let count = sweep().1;
    let directory = trace::directory().map(|directory| directory.join(label));
    for seed in seeds {
        if let Some((_, _, issue)) = HELD_DOCUMENT_SEEDS
            .iter()
            .filter(|_| std::env::var_os("WORTH_DST_RUN_HELD").is_none())
            .find(|(held, corrections, _)| label == "document" && *held == seed && *corrections == workloads.corrections)
        {
            eprintln!("DST seed {seed:#x} held on {issue}");
            continue;
        }
        let ops = generate(seed, count, workloads.corrections, scenario(), workloads.environment);
        let mut trace = trace::Trace::new();
        let result = catch_unwind(AssertUnwindSafe(|| run(&mut system(seed), &ops, Some(&mut trace))));
        // A seed that panics keeps the steps before its panic, for tracing.
        if let Some(directory) = &directory {
            trace.write(directory, seed);
        }
        let result = result
            .unwrap_or_else(|_| panic!("DST seed {seed:#x} (scenario {:?}) panicked; its message is above", scenario()));
        if let Err(failure) = result {
            eprintln!("DST disagreement: seed {seed:#x} at step {}, shrinking:
{failure:#?}", failure.step);
            // A replay that panics is another failure, not this one.
            let fails = |ops: &[Op]| {
                catch_unwind(AssertUnwindSafe(|| run(&mut system(seed), ops, None).is_err())).unwrap_or(false)
            };
            report(seed, ops, failure, fails);
        }
    }
}

fn report(seed: u64, ops: Vec<Op>, failure: Failure, fails: impl FnMut(&[Op]) -> bool) -> ! {
    let shrunk = shrink(ops, fails);
    let Disagreement { action, outcome, observed, allowed, invariant } = *failure.disagreement;
    panic!(
        "DST disagreement: seed {seed:#x} (scenario {:?}) at step {}\n\
         action {action:?} returned {outcome:?}; the model allows {allowed:?}\n\
         broken invariant: {invariant:?}\nobserved {observed:#?}\nshrunk to {} choices:\n{shrunk:?}",
        scenario(),
        failure.step,
        shrunk.len(),
    );
}

/// The oracle's bookkeeping holds the model's invariants and agrees with its
/// own transitions over ten times the sweep; its traces are what TLC checks
/// the translation with.
#[test]
fn the_oracle_agrees_with_the_model_it_translates() {
    let seeds = sweep().0;
    let wide = seeds.start..seeds.start + 10 * (seeds.end - seeds.start);
    let workloads = Workloads { environment: &ENVIRONMENT, corrections: true };
    agree("model", wide, workloads, ModelSystem::new);
}

/// The real document agrees with the oracle after every call. One host per
/// tree shape is installed before the sweep, and each seed attaches its own
/// document on the host its seed picks: the members alone, with the `dst-use`
/// consumer of the root, or with the #358 consumer chain. The invalidation
/// shadow is armed throughout, so every marked certification is checked
/// against full verification.
#[test]
fn the_document_agrees_with_the_oracle() {
    use fixture::program::Shape;
    let equivalence = worth_query_host::facade::primary_graph::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    const SHAPES: [Shape; 3] = [Shape::Members, Shape::WithConsumer, Shape::WithChain];
    let hosts = SHAPES.map(world::install_as);
    let workloads = Workloads {
        environment: &world::ENVIRONMENT,
        corrections: variable("WORTH_DST_CORRECTIONS", 0) == 1,
    };
    agree("document", sweep().0, workloads, |seed| {
        let shape = (seed % SHAPES.len() as u64) as usize;
        world::Document::attach_as(&hosts[shape], SHAPES[shape])
    });
    assert!(equivalence.shadow_checks_for_test() > 0, "the armed shadow must check the sweep's certifications");
}
