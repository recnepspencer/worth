//! Public #358: consumer outputs readmitted at a forked head certify from
//! the readmitted marks. Each court runs the #358 consumer chain
//! (`dst-mid` reads `dst-a`'s current output for `e1`, `dst-leaf` reads
//! `dst-mid`'s) with the invalidation shadow armed, so a mark that accepts
//! what full verification rejects panics. The observation after every call
//! checks the chain against the outputs it reads, and the oracle checks
//! everything else.
use super::driver::System;
use super::reference::{Action, Observed, Outcome, Stage, Stages};
use super::workload::{Op, StagePick::*};
use super::*;
use fixture::program::Shape;
use worth_query_host::facade::primary_graph::{
    carried_historical_projections_on_this_thread_for_test as carried,
    decline_historical_readmissions_on_this_thread_for_test as decline_readmissions,
    declined_historical_readmissions_on_this_thread_for_test as declined,
    historical_acceptances_on_this_thread_for_test as acceptances,
    readmitted_chains_on_this_thread_for_test as readmitted,
    settlement_registrations_on_this_thread_for_test as registrations,
    WorthQueryInvalidationEquivalenceForTest,
};

/// Restores `e1 = 1` by an undo, then forks the head with an `e2` record and
/// settles it with a write outside: the readmission of the chain on the
/// forked head. Every later court starts here.
const PREFIX: [Op; 5] = [
    Op::Edit([Set(1), Skip]),
    Op::Edit([Set(2), Skip]),
    Op::Undo,
    Op::Edit([Skip, Set(1)]),
    Op::OutOfBand([Skip, Set(2)]),
];

/// The document, settled once more at its head before it closes.
struct SettledTwice<'h> {
    document: world::Document<'h>,
    /// Writes outside the document after the readmission. The model's
    /// diverged head refuses them, so they run unmodelled before the second
    /// settle; the chain observation checks what they settle.
    tail: &'h [Stages],
    /// Settlement registrations the second settle and its read made.
    second: Option<u64>,
}

impl System for SettledTwice<'_> {
    fn act(&mut self, action: &Action) -> Outcome {
        if matches!(action, Action::Close) {
            for stages in self.tail {
                self.document.write_outside_unmodelled(stages);
                self.document.observe(1);
            }
            // The second settle at the head the last read saw; the read after
            // it checks the chain again before close retires the branch.
            let before = registrations();
            self.document.settle_again();
            self.document.observe(1);
            self.second = Some(registrations() - before);
        }
        self.document.act(action)
    }

    fn observe(&mut self, limit: u32) -> Observed {
        self.document.observe(limit)
    }

    fn nodes(&self) -> u32 {
        self.document.nodes()
    }

    fn release(&mut self) {
        self.document.release()
    }
}

/// What a court observed on this thread.
struct Counts {
    /// Moved facts or aspects accepted only by their row's selection evidence.
    acceptances: u64,
    /// Equal successors that took a projection carrying a coordinate.
    carried: u64,
    /// Settlement registrations made by the second settle at the same head.
    second_settle_registrations: u64,
    /// Consumer chains readmitted with marks at their selected head.
    readmitted: u64,
    /// Verified readmissions the armed decline door refused.
    declined: u64,
}

/// Run `ops` on a fresh chain document, write `tail` outside it, settle it
/// a second time before close, and require that the armed shadow checked a
/// certification.
fn court(name: &str, ops: &[Op], tail: &[Stages]) -> Counts {
    court_as(name, Shape::WithChain, None, ops, tail)
}

/// [`court`] on `shape`, with every verified readmission of a consumer
/// (`Some(true)`) or a root (`Some(false)`) declined.
fn court_as(name: &str, shape: Shape, decline: Option<bool>, ops: &[Op], tail: &[Stages]) -> Counts {
    let equivalence = WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    if let Some(consumers) = decline {
        decline_readmissions(consumers);
    }
    let host = world::install_as(shape);
    let mut system = SettledTwice { document: world::Document::attach_as(&host, shape), tail, second: None };
    if let Err(failure) = run(&mut system, ops, None) {
        panic!("#358 {name} ({ops:?}) disagrees:\n{failure:#?}");
    }
    let Some(second_settle_registrations) = system.second else {
        panic!("#358 {name}: the court must settle twice");
    };
    // Members bind no consumer, so nothing certifies consumed outputs there.
    assert!(
        shape == Shape::Members || equivalence.shadow_checks_for_test() > 0,
        "#358 {name}: the armed shadow must check a certification"
    );
    Counts { acceptances: acceptances(), carried: carried(), second_settle_registrations, readmitted: readmitted(), declined: declined() }
}

/// Court 2: the two-hop chain on the forked head certifies from its
/// readmitted marks, and a second settle at the same head reuses them.
#[test]
fn chain_on_a_forked_head_reuses_marks() {
    let counts = court("chain_on_a_forked_head_reuses_marks", &PREFIX, &[]);
    assert!(counts.acceptances > 0, "the restored e1 fact must hold only through its selection evidence");
    assert!(counts.readmitted > 0, "the forked head must readmit the chain, not execute it afresh");
    assert_eq!(counts.second_settle_registrations, 0, "the second settle must certify from the readmitted marks");
}

/// Court 3(a): a write to `e1` after the readmission changes `dst-a(e1)`'s
/// parity, so `dst-mid` and `dst-leaf` re-execute to the new values; the
/// retire variant retires `e2`, then `e1`, and the chain reads 0 and 100.
#[test]
fn upstream_change_after_readmission_invalidates_consumer() {
    let change = court("upstream change", &PREFIX, &[[Stage::Set(2), Stage::Skip]]);
    assert!(change.readmitted > 0, "the change court must readmit before it invalidates");
    let retirement = court(
        "upstream retirement",
        &PREFIX,
        &[[Stage::Skip, Stage::Retire], [Stage::Retire, Stage::Skip]],
    );
    assert!(retirement.readmitted > 0, "the retirement court must readmit before it invalidates");
}

/// Court 3(b), the selection-refusal court. The settle that would readmit
/// the chain on the forked head also writes `e1 = 2` outside, so the
/// transported rows that read `e1 = 1` are not current there. Native
/// selection's exact predicate (`restored_output_is_current`) refuses them,
/// so no chain is readmitted and none posts marks: the chain executes again
/// to the new values. Readmission's own currentness check stays behind it as
/// defense in depth.
#[test]
fn stale_transported_chain_is_refused_at_selection_with_zero_marks() {
    let ops = [
        Op::Edit([Set(1), Skip]),
        Op::Edit([Set(2), Skip]),
        Op::Undo,
        Op::Edit([Skip, Set(1)]),
        Op::OutOfBand([Set(2), Set(2)]),
    ];
    let counts = court("stale transported chain", &ops, &[]);
    assert_eq!(counts.readmitted, 0, "a chain whose facts are not current is never readmitted");
}

/// Liveness: a consumer readmission that verified current and then declined,
/// as any later stop would, degrades to ordinary execution. The chain
/// executes again, and every read is checked against the reference values.
#[test]
fn declined_consumer_readmission_executes_and_settles() {
    let counts = court_as("declined consumer readmission", Shape::WithChain, Some(true), &PREFIX, &[]);
    assert!(counts.declined > 0, "the door must decline a verified consumer readmission");
    assert_eq!(counts.readmitted, 0, "a declined chain is never readmitted");
}

/// Liveness: a declined root readmission executes its producer again and
/// settles the reference values; it never stops as a missing producer.
#[test]
fn declined_root_readmission_executes_and_settles() {
    let counts = court_as("declined root readmission", Shape::Members, Some(false), &PREFIX, &[]);
    assert!(counts.declined > 0, "the door must decline a verified root readmission");
}

/// The readmission work of [`PREFIX`], counted after its history.
#[derive(Clone, Copy, Debug)]
struct Work {
    acceptances: u64,
    carried: u64,
    registrations: u64,
    readmitted: u64,
}

impl Work {
    fn now() -> Self {
        Self { acceptances: acceptances(), carried: carried(), registrations: registrations(), readmitted: readmitted() }
    }
}

/// The document, its counters taken once its first `history` calls ran.
struct AfterHistory<'h> {
    document: world::Document<'h>,
    history: usize,
    calls: usize,
    before: Option<Work>,
}

impl System for AfterHistory<'_> {
    fn act(&mut self, action: &Action) -> Outcome {
        if self.calls == self.history {
            self.before = Some(Work::now());
        }
        self.calls += 1;
        self.document.act(action)
    }

    fn observe(&mut self, limit: u32) -> Observed {
        self.document.observe(limit)
    }

    fn nodes(&self) -> u32 {
        self.document.nodes()
    }

    fn release(&mut self) {
        self.document.release()
    }
}

/// [`PREFIX`]'s readmission work after `edits` steps of unrelated history:
/// `e2` edits whose records and outputs the `e1` chain never reads. The
/// last one leaves `e2 = 2`, so the fork's `e2 = 1` record still changes it.
fn readmission_work(edits: usize) -> Work {
    let equivalence = WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_as(Shape::WithChain);
    let mut ops: Vec<Op> = (0..edits).map(|n| Op::Edit([Skip, Set(2 - ((edits - 1 - n) % 2) as u64)])).collect();
    ops.extend(PREFIX);
    let mut system =
        AfterHistory { document: world::Document::attach_as(&host, Shape::WithChain), history: edits, calls: 0, before: None };
    if let Err(failure) = run(&mut system, &ops, None) {
        panic!("#358 scaling ({edits} edits) disagrees:\n{failure:#?}");
    }
    let before = system.before.expect("the history precedes the readmission");
    assert!(equivalence.shadow_checks_for_test() > 0, "the armed shadow must check a certification");
    let after = Work::now();
    Work {
        acceptances: after.acceptances - before.acceptances,
        carried: after.carried - before.carried,
        registrations: after.registrations - before.registrations,
        readmitted: after.readmitted - before.readmitted,
    }
}

/// Scaling court: readmitting the forked-head chain does the same work
/// after ten times the unrelated history (within 1.1x). The per-branch
/// re-rooted map is bounded at its only writer (`install_rerooted`).
#[test]
fn readmission_work_does_not_grow_with_unrelated_history() {
    let small = readmission_work(3);
    let large = readmission_work(30);
    assert!(small.readmitted > 0 && small.registrations > 0, "the fork must readmit the chain: {small:?}");
    for (name, small, large) in [
        ("acceptances", small.acceptances, large.acceptances),
        ("carried projections", small.carried, large.carried),
        ("settlement registrations", small.registrations, large.registrations),
        ("readmitted chains", small.readmitted, large.readmitted),
    ] {
        assert!(large * 10 <= small * 11, "{name} grew with unrelated history: 1x {small}, 10x {large}");
    }
}

/// Court 5: after the readmission, a write that keeps `dst-a(e1)`'s parity
/// re-executes `dst-mid` to an equal output, so its successor carries the
/// readmitted output projection; `dst-leaf` certifies through the equal
/// chain, and the second settle reuses it.
#[test]
fn equal_adoption_after_readmission_keeps_the_restored_projection() {
    let counts = court("equal adoption", &PREFIX, &[[Stage::Set(3), Stage::Skip]]);
    assert!(counts.carried > 0, "the equal successor must carry the readmitted projection");
    assert!(counts.acceptances > 0, "the carried projection must hold through its selection evidence");
}
