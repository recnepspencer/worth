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
use worth_query_host::facade::primary_graph::WorthQueryInvalidationEquivalenceForTest;

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
    settled: bool,
}

impl System for SettledTwice<'_> {
    fn act(&mut self, action: &Action) -> Outcome {
        if matches!(action, Action::Close) {
            // The second settle at the head the last read saw; the read after
            // it checks the chain again before close retires the branch.
            self.document.settle_again();
            self.document.observe(1);
            self.settled = true;
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

/// Run `ops` on a fresh chain document, settle it a second time before close,
/// and require that the armed shadow checked a certification.
fn court(name: &str, ops: &[Op]) {
    let equivalence = WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_as(Shape::WithChain);
    let mut system = SettledTwice { document: world::Document::attach_chain(&host), settled: false };
    if let Err(failure) = run(&mut system, ops, None) {
        panic!("#358 {name} ({ops:?}) disagrees:\n{failure:#?}");
    }
    assert!(system.settled, "#358 {name}: the court must settle twice");
    assert!(equivalence.shadow_checks_for_test() > 0, "#358 {name}: the armed shadow must check a certification");
}

fn after_prefix(tail: &[Op]) -> Vec<Op> {
    PREFIX.iter().chain(tail).copied().collect()
}

/// Court 2: the two-hop chain on the forked head certifies from its
/// readmitted marks, and a second settle at the same head reuses them.
#[test]
#[ignore = "red-first: lands with S1 (public #358)"]
fn chain_on_a_forked_head_reuses_marks() {
    court("chain_on_a_forked_head_reuses_marks", &PREFIX);
}

/// Court 3(a): a write to `e1` after the readmission changes `dst-a(e1)`'s
/// parity, so `dst-mid` and `dst-leaf` re-execute to the new values; the
/// retire variant retires `e2`, then `e1`, and the chain reads 0 and 100.
#[test]
#[ignore = "red-first: lands with S1 (public #358)"]
fn upstream_change_after_readmission_invalidates_consumer() {
    court("upstream change", &after_prefix(&[Op::OutOfBand([Set(2), Skip])]));
    court("upstream retirement", &after_prefix(&[Op::OutOfBand([Skip, Retire]), Op::OutOfBand([Retire, Skip])]));
}

/// Court 3(b), ABA: writes outside move `e1` 1 → 2 → 1 before the
/// readmitting undo. The revision moved without a selection restore, so the
/// readmitted chain's facts are not current and readmission must refuse:
/// the chain executes again and still reads its sources.
#[test]
#[ignore = "red-first: lands with S1 (public #358)"]
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
    court("diverged upstream", &ops);
}

/// Court 5: after the readmission, a write that keeps `dst-a(e1)`'s parity
/// re-executes `dst-mid` to an equal output, so its successor carries the
/// readmitted output projection; `dst-leaf` certifies through the equal
/// chain, and the second settle reuses it.
#[test]
#[ignore = "red-first: lands with S1 (public #358)"]
fn equal_adoption_after_readmission_keeps_the_restored_projection() {
    court("equal adoption", &after_prefix(&[Op::OutOfBand([Set(3), Skip])]));
}
