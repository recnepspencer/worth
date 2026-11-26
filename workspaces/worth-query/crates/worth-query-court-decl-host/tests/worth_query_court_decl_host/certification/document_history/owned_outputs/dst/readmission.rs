//! Public #358: consumer outputs readmitted at a forked head certify from
//! the readmitted marks. Each court runs the #358 consumer chain
//! (`dst-mid` reads `dst-a`'s current output for `e1`, `dst-leaf` reads
//! `dst-mid`'s) with the invalidation shadow armed, so a mark that accepts
//! what full verification rejects panics. The observation after every call
//! checks the chain against the outputs it reads, and the oracle checks
//! everything else.
use super::driver::System;
use super::reference::{Action, Observed, Outcome};
use super::workload::{Op, StagePick::*};
use super::*;
use fixture::program::Shape;
use worth_query_host::facade::primary_graph::{
    carried_historical_projections_on_this_thread_for_test as carried,
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
    /// Settlement registrations the second settle and its read made.
    second: Option<u64>,
}

impl System for SettledTwice<'_> {
    fn act(&mut self, action: &Action) -> Outcome {
        if matches!(action, Action::Close) {
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
}

/// Run `ops` on a fresh chain document, settle it a second time before close,
/// and require that the armed shadow checked a certification.
fn court(name: &str, ops: &[Op]) -> Counts {
    let equivalence = WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_as(Shape::WithChain);
    let mut system = SettledTwice { document: world::Document::attach_chain(&host), second: None };
    if let Err(failure) = run(&mut system, ops, None) {
        panic!("#358 {name} ({ops:?}) disagrees:\n{failure:#?}");
    }
    let Some(second_settle_registrations) = system.second else {
        panic!("#358 {name}: the court must settle twice");
    };
    assert!(equivalence.shadow_checks_for_test() > 0, "#358 {name}: the armed shadow must check a certification");
    Counts { acceptances: acceptances(), carried: carried(), second_settle_registrations, readmitted: readmitted() }
}

fn after_prefix(tail: &[Op]) -> Vec<Op> {
    PREFIX.iter().chain(tail).copied().collect()
}

/// Court 2: the two-hop chain on the forked head certifies from its
/// readmitted marks, and a second settle at the same head reuses them.
#[test]
fn chain_on_a_forked_head_reuses_marks() {
    let counts = court("chain_on_a_forked_head_reuses_marks", &PREFIX);
    assert!(counts.acceptances > 0, "the restored e1 fact must hold only through its selection evidence");
    assert_eq!(counts.second_settle_registrations, 0, "the second settle must certify from the readmitted marks");
}

/// Court 3(a): a write to `e1` after the readmission changes `dst-a(e1)`'s
/// parity, so `dst-mid` and `dst-leaf` re-execute to the new values; the
/// retire variant retires `e2`, then `e1`, and the chain reads 0 and 100.
#[test]
fn upstream_change_after_readmission_invalidates_consumer() {
    let _ = court("upstream change", &after_prefix(&[Op::OutOfBand([Set(2), Skip])]));
    let _ = court("upstream retirement", &after_prefix(&[Op::OutOfBand([Skip, Retire]), Op::OutOfBand([Retire, Skip])]));
}

/// Court 3(b), ABA: writes outside move `e1` 1 → 2 → 1 before the
/// readmitting undo. The revision moved without a selection restore, so the
/// readmitted chain's facts are not current and readmission must refuse:
/// the chain executes again and still reads its sources.
#[test]
fn diverged_upstream_refuses_readmission_with_zero_marks() {
    let ops = [
        Op::Edit([Set(1), Skip]),
        Op::Edit([Set(2), Skip]),
        Op::OutOfBand([Set(1), Skip]),
        Op::OutOfBand([Set(2), Skip]),
        Op::Undo,
        Op::Edit([Skip, Set(1)]),
        Op::OutOfBand([Skip, Set(2)]),
    ];
    let counts = court("diverged upstream", &ops);
    assert_eq!(counts.readmitted, 0, "a chain whose facts are not current is never readmitted");
}

/// Court 5: after the readmission, a write that keeps `dst-a(e1)`'s parity
/// re-executes `dst-mid` to an equal output, so its successor carries the
/// readmitted output projection; `dst-leaf` certifies through the equal
/// chain, and the second settle reuses it.
#[test]
#[ignore = "red-first: lands with S2 (public #358)"]
fn equal_adoption_after_readmission_keeps_the_restored_projection() {
    let counts = court("equal adoption", &after_prefix(&[Op::OutOfBand([Set(3), Skip])]));
    assert!(counts.carried > 0, "the equal successor must carry the readmitted projection");
    assert!(counts.acceptances > 0, "the carried projection must hold through its selection evidence");
}
