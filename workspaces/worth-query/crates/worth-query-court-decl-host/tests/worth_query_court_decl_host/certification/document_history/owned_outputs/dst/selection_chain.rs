//! Public #429 on chains of `current_output` readers: `dst-mid` reads
//! `dst-a`'s current output for `e1`, `dst-leaf` reads `dst-mid`'s, and
//! `dst-c3` to `dst-c8` each read the level below. A write outside the
//! document moves `e1`; the court measures what that settle executes.
use super::driver::System;
use super::reference::{Action, Observed, Outcome};
use super::workload::{Op, StagePick::*};
use super::*;
use fixture::program::Shape;
use worth_query_host::facade::primary_graph::WorthQueryInvalidationEquivalenceForTest;

/// The document, with the settle of its write outside measured.
struct Measured<'h> {
    document: world::Document<'h>,
    /// Chain executions, `dst-mid` first, while the write outside settled.
    builds: Option<[u64; 8]>,
}

impl System for Measured<'_> {
    fn act(&mut self, action: &Action) -> Outcome {
        if !matches!(action, Action::OutOfBand(_)) {
            return self.document.act(action);
        }
        let builds = family::dst::chain_builds();
        let outcome = self.document.act(action);
        let after = family::dst::chain_builds();
        self.builds = Some(std::array::from_fn(|level| after[level] - builds[level]));
        outcome
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

/// Set `e1 = 1` on a fresh chain `depth` levels deep, then write `e1 =
/// value` outside the document. Every observation checks the chain against
/// the outputs it reads. Returns the chain executions of the write outside.
fn moved_root(depth: usize, value: u64) -> [u64; 8] {
    let shape = Shape::chain_of(depth);
    let host = world::install_as(shape);
    let mut system = Measured { document: world::Document::attach_as(&host, shape), builds: None };
    let ops = [Op::Edit([Set(1), Skip]), Op::OutOfBand([Set(value), Skip])];
    if let Err(failure) = run(&mut system, &ops, None) {
        panic!("#429 chain of {depth} ({ops:?}) disagrees:\n{failure:#?}");
    }
    system.builds.expect("the court writes outside the document")
}

/// Court 14 (f): in the chain four levels deep, `e1` going 1 → 3 re-executes
/// `dst-mid` to an equal parity, so its equal adoption links `dst-mid_old →
/// dst-mid_new`. `dst-leaf` consumed `dst-mid_old` and is carried with its
/// sealed witness, so `dst-c3`, which recorded `Unique(dst-leaf)`, follows
/// the equal link below its producer and stays current, as a fresh read
/// would: nothing above `dst-mid` executes. With `e1` going 1 → 2 instead,
/// `dst-mid` itself changes: every level above it moves and converges in
/// one execution. The shadow is armed in both variants.
#[test]
fn a_reader_of_a_producer_carried_over_an_equal_adoption_stays_current() {
    let equivalence = WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let equal = moved_root(4, 3);
    assert!(equal[0] > 0, "dst-mid re-executes to an equal parity: {equal:?}");
    assert_eq!(equal[1..4], [0; 3], "every level above the equal adoption stays current: {equal:?}");
    let changed = moved_root(4, 2);
    assert!(changed[0] > 0, "dst-mid re-executes to a new parity: {changed:?}");
    assert_eq!(changed[1..4], [1; 3], "each level above a changed dst-mid executes once: {changed:?}");
    assert!(equivalence.shadow_checks_for_test() > 0, "the armed shadow must check a certification");
}
