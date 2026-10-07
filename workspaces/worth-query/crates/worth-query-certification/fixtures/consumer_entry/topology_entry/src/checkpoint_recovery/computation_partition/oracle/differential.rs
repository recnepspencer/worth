//! Partition reuse proven exact: one seeded sequence of edits runs through a
//! runtime that keeps its output, and every step's run is checked against a
//! full run of the same facts in a runtime installed for that step, which
//! holds no record to reuse: the same outcome, the same charged work, and the
//! same partition named where the work ceiling stops a run.

use worth_query_host::facade::application_contribution::{
    WorthQueryComputationPartitionStop as Stop, WorthQueryManagedComputationResourceDenial,
    WorthQueryPartitionedComputationFullCause as FullCause,
    WorthQueryPartitionedComputationRun as Run,
};

use super::super::region_output::{arm_own_write, OwnWrite};
use super::*;

mod alphabet;

use alphabet::{Change, Kind, Lcg, Model, KINDS};

/// Each round makes one edit of every kind, in a seeded order.
const ROUNDS: usize = 4;
const SEED: u64 = 0x9176_3c0b_5eed_0001;

/// The runs of a runtime installed over `model`, which holds no record. A
/// commit whose own write replaced a value its computation gathered is born
/// stale, so the demand runs the producer again over the written value; that
/// run writes the value it read, and the demand settles.
fn full_run(model: &Model, own_write: Option<OwnWrite>) -> Vec<OracleRun> {
    let application = install(|graph| model.seed(graph));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    if model.odd {
        adjust(&request, &application, ODD_Y, 1);
    }
    arm_own_write(own_write);
    let (contacts, runs) = demand(&request, &application);
    arm_own_write(None);
    let moved = own_write.is_some_and(|write| !model.holds(write));
    let rerun = moved && runs.first().is_some_and(|run| run.outcome.is_ok());
    assert_eq!(
        (contacts, runs.len()),
        (1, 1 + usize::from(rerun)),
        "a fresh runtime runs the producer once, and again after its own write"
    );
    let mut first = runs[0].runs.iter();
    assert!(
        first.all(|run| *run == Run::Full(FullCause::NoPriorRecord)),
        "a fresh runtime has no record to reuse: {:?}",
        runs[0]
    );
    runs
}

fn outcomes(
    runs: &[OracleRun],
) -> Vec<&Result<(u64, u64), WorthQueryPartitionedComputationDenial<u32>>> {
    runs.iter().map(|run| &run.outcome).collect()
}

/// How a step's run must run when it and the run before it completed: every
/// edit of the facts or the entries reuses, and only a new input runs in
/// full.
fn expected(kind: Kind) -> Option<Run> {
    match kind {
        Kind::Input => Some(Run::Full(FullCause::InputChanged)),
        Kind::Fault => None,
        _ => Some(Run::Incremental),
    }
}

/// One demand of the seeded sequence: the facts it was made over, the edit
/// before it, none for the first, whether that edit moved a fact, the value
/// its decision writes, and what it ran.
pub(super) struct Demanded<'model> {
    model: &'model Model,
    kind: Option<Kind>,
    moved: bool,
    own_write: Option<OwnWrite>,
    pub(super) contacts: usize,
    pub(super) runs: Vec<OracleRun>,
}

/// Runs the seeded sequence of edits through one runtime that keeps its
/// output, and hands `each` every demand in order. The caller holds the
/// checkpoint recovery guard.
pub(super) fn sequence(mut each: impl FnMut(&str, Demanded<'_>)) {
    let mut rng = Lcg(SEED);
    let mut model = Model::new(&mut rng);
    let application = install(|graph| model.seed(graph));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (contacts, runs) = demand(&request, &application);
    let first = Demanded {
        model: &model,
        kind: None,
        moved: true,
        own_write: None,
        contacts,
        runs,
    };
    each("the first demand", first);
    let mut command = 0_u64;
    for round in 0..ROUNDS {
        let mut kinds = KINDS;
        for place in (1..kinds.len()).rev() {
            kinds.swap(place, rng.below(place + 1));
        }
        let kinds = kinds.into_iter().flat_map(|kind| match kind {
            Kind::Ceiling => vec![Kind::Ceiling, Kind::Relief],
            Kind::Fault => vec![Kind::Fault, Kind::Repair],
            kind => vec![kind],
        });
        for kind in kinds {
            let before = model.clone();
            let step = model.step(kind, &mut rng);
            for change in step.changes {
                command += 1;
                match change {
                    Change::Entry(change) => edit(&request, &application, change, command),
                    Change::Ordinate(y) => adjust(&request, &application, y, command),
                }
            }
            arm_own_write(step.own_write);
            let (contacts, runs) = demand(&request, &application);
            arm_own_write(None);
            let demanded = Demanded {
                model: &model,
                kind: Some(kind),
                moved: model != before,
                own_write: step.own_write,
                contacts,
                runs,
            };
            each(&format!("round {round}, {kind:?}"), demanded);
            if let Some(write) = step.own_write {
                model.written(write);
            }
        }
    }
}

#[test]
fn every_reused_run_equals_a_full_run_of_the_same_facts() {
    let _guard = checkpoint_recovery_test_guard();
    let mut last: Option<OracleRun> = None;
    let (mut incremental, mut full, mut ceilings) = (0_usize, 0_usize, 0_usize);
    sequence(|at, mut demanded| {
        let reference = full_run(demanded.model, demanded.own_write);
        // A demand that runs nothing keeps the output of the last run, which
        // must equal a full run of the facts as they are now.
        if let (Some(last_run), true) = (last.as_ref(), demanded.runs.is_empty()) {
            let reference = reference.last().expect("a fresh runtime runs");
            assert_eq!(
                demanded.contacts, 0,
                "{at}: a demand that ran nothing kept its output"
            );
            assert!(!demanded.moved, "{at}: an edit that moved a fact runs");
            assert_eq!(
                reference.outcome, last_run.outcome,
                "{at}: the kept output is current"
            );
            return;
        }
        // Each run equals the full run at its place: the first over the
        // edited facts, and a rerun over the producer's own write.
        assert_eq!(
            outcomes(&demanded.runs),
            outcomes(&reference),
            "{at}: reuse equals a full run, run by run"
        );
        let Some(last_run) = last.as_ref() else {
            assert_eq!(demanded.contacts, 1);
            last = demanded.runs.pop();
            return;
        };
        for rerun in &demanded.runs[1..] {
            assert_eq!(rerun.runs, [Run::Incremental], "{at}: a rerun reuses");
        }
        let run = &demanded.runs[0];
        match run.runs.as_slice() {
            [Run::Incremental] => incremental += 1,
            [Run::Full(_)] => full += 1,
            _ => {}
        }
        // A stopped run is not shown to the observer. One that follows a
        // completed run had that run's record to reuse, and must stop at the
        // partition a full run stops at, which the outcome names.
        let exhausted = Stop::Resource(WorthQueryManagedComputationResourceDenial::WorkExhausted);
        if last_run.outcome.is_ok()
            && matches!(&run.outcome, Err(WorthQueryPartitionedComputationDenial::Partition { cause, .. }) if *cause == exhausted)
        {
            assert!(run.runs.is_empty(), "{at}: a stopped run is not shown");
            ceilings += 1;
        }
        if let (Some(expected), true) = (
            demanded.kind.and_then(expected),
            last_run.outcome.is_ok() && run.outcome.is_ok(),
        ) {
            assert_eq!(run.runs, [expected], "{at}: the run's kind");
        }
        last = demanded.runs.pop();
    });
    assert!(
        incremental > full,
        "most steps reuse partitions: {incremental} incremental, {full} full"
    );
    assert!(
        ceilings > 0,
        "a reused run stops at the work ceiling where a full run stops"
    );
}
