//! Public #429 on chains of `current_output` readers: `dst-mid` reads
//! `dst-a`'s current output for `e1`, `dst-leaf` reads `dst-mid`'s, and
//! `dst-c3` to `dst-c8` each read the level below. A write outside the
//! document moves `e1`; the court measures what that settle executes and
//! what each certification frame re-resolved.
use super::driver::System;
use super::reference::{Action, Observed, Outcome};
use super::workload::{Op, StagePick::*};
use super::*;
use fixture::program::Shape;
use worth_query_host::facade::primary_graph::{
    family_selection_frame_costs_on_this_thread_for_test as frame_costs, WorthQueryFamilySelectionCostForTest as Cost,
    WorthQueryInvalidationEquivalenceForTest,
};

/// The document, with the settle of its write outside measured.
struct Measured<'h> {
    document: world::Document<'h>,
    /// Chain executions, `dst-mid` first, while the write outside settled.
    builds: Option<[u64; 8]>,
    /// The cost of each certification frame that settle closed.
    frames: Vec<Cost>,
}

impl System for Measured<'_> {
    fn act(&mut self, action: &Action) -> Outcome {
        if !matches!(action, Action::OutOfBand(_)) {
            return self.document.act(action);
        }
        let builds = family::dst::chain_builds();
        let frames = frame_costs().len();
        let outcome = self.document.act(action);
        let after = family::dst::chain_builds();
        self.builds = Some(std::array::from_fn(|level| after[level] - builds[level]));
        self.frames = frame_costs()[frames..].to_vec();
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
/// the outputs it reads. Returns the chain executions and the certification
/// frames of the write outside.
fn moved_root(depth: usize, value: u64) -> ([u64; 8], Vec<Cost>) {
    settled(depth, [Op::Edit([Set(1), Skip]), Op::OutOfBand([Set(value), Skip])])
}

/// Run `ops` on a fresh chain `depth` levels deep; the last is the write
/// outside whose settle is measured.
fn settled(depth: usize, ops: [Op; 2]) -> ([u64; 8], Vec<Cost>) {
    let shape = Shape::chain_of(depth);
    let host = world::install_as(shape);
    let mut system = Measured { document: world::Document::attach_as(&host, shape), builds: None, frames: Vec::new() };
    if let Err(failure) = run(&mut system, &ops, None) {
        panic!("#429 chain of {depth} ({ops:?}) disagrees:\n{failure:#?}");
    }
    (system.builds.expect("the court writes outside the document"), system.frames)
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
    let (equal, _) = moved_root(4, 3);
    assert!(equal[0] > 0, "dst-mid re-executes to an equal parity: {equal:?}");
    assert_eq!(equal[1..4], [0; 3], "every level above the equal adoption stays current: {equal:?}");
    let (changed, _) = moved_root(4, 2);
    assert!(changed[0] > 0, "dst-mid re-executes to a new parity: {changed:?}");
    assert_eq!(changed[1..4], [1; 3], "each level above a changed dst-mid executes once: {changed:?}");
    assert!(equivalence.shadow_checks_for_test() > 0, "the armed shadow must check a certification");
}

/// Plan §5, blocker 4: certifying the top reader of a chain costs at most
/// two re-resolutions and two compared candidates per level: the nested
/// re-resolution of each producer, and the marked walk's own visit of that
/// level, which takes the closure memo. So the costliest certification frame
/// of the settle stays within `2 × depth`. Without the memo each level
/// compares its producer's closure again, and the cost doubles per level.
#[test]
fn selection_chain_verification_does_not_grow_superlinearly() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let costs = [1, 2, 4, 8].map(|depth| {
        let (builds, frames) = moved_root(depth, 3);
        assert!(builds[0] > 0, "dst-mid re-executes to an equal parity: {builds:?}");
        assert!(builds[1..depth].iter().all(|&built| built == 0), "the chain above dst-mid is carried: {builds:?}");
        let top = frames.iter().copied().max_by_key(|cost| (cost.rewalks, cost.candidates_compared)).unwrap_or_default();
        eprintln!("#429 chain of {depth}: costliest certification {top:?} of {} frames", frames.len());
        let bound = 2 * depth as u64;
        assert!(
            top.rewalks <= bound && top.candidates_compared <= bound,
            "the chain of {depth} costs more than two re-resolutions per level: {top:?}"
        );
        top
    });
    assert!(costs[0].rewalks > 0, "the top reader's certification re-resolves its producer");
}

/// Court 14 (h)(2): a write outside the document that moves only `e2`, which
/// no level of the chain reads, readmits the chain's settlements through
/// `require_current_settlements_in` (site 9). An earlier settlement's
/// readmission carries its read basis, which moves the root payload, before
/// a later settlement's `FamilySelection` fact is certified; that fact still
/// certifies `Unmoved`, with no retry, so nothing in the chain executes.
#[test]
fn a_chain_readmitted_after_an_unrelated_write_stays_current() {
    let equivalence = WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let (builds, frames) = settled(4, [Op::Edit([Set(1), Set(1)]), Op::OutOfBand([Skip, Set(5)])]);
    eprintln!("#429 chain of 4 after an unrelated write: builds {builds:?}, {} certification frames", frames.len());
    assert_eq!(builds[..4], [0; 4], "no level of the chain reads e2: {builds:?}");
    assert!(frames.iter().any(|cost| cost.rewalks > 0), "the readmission re-resolves a selection: {frames:?}");
    assert!(equivalence.shadow_checks_for_test() > 0, "the armed shadow must check a certification");
}
