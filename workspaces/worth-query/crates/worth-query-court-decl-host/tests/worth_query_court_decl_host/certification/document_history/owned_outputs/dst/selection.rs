//! Public #429: a consumer's `current_output` selection is a verifiable
//! dependency. In the #358 chain `dst-mid` reads `dst-a`'s current output
//! for `e1`; a read that selected `Missing` must not stay current once
//! `dst-a` is readmitted. The observation after every call checks the chain
//! against the outputs it reads: `dst-mid` is the parity of `dst-a`'s output
//! for `e1` and `dst-leaf` that plus 100.
use super::workload::{Op, StagePick::*};
use super::*;

/// Seed `0x7b02000f` at corrections 1, shrunk from 16 choices to 11 with the
/// #358 shrinker on its stale chain read. Before #429 an observation reads
/// the chain as `(0, 100)` where `dst-a`'s output gives `(1, 101)`: `dst-mid`
/// keeps a `Missing` selection that `dst-a`'s readmission moved.
#[test]
fn a_missing_read_is_invalidated_by_a_later_readmission() {
    let equivalence =
        worth_query_host::facade::primary_graph::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_chain();
    let ops = [
        Op::Redo { pick: 101 },
        Op::Edit([Set(1), Set(2)]),
        Op::Undo,
        Op::Undo,
        Op::Edit([Skip, Skip]),
        Op::Edit([Set(2), Skip]),
        Op::Correction { pick: 48, entity: 1, on_line: true, cascade: false, stop: None },
        Op::Correction { pick: 188, entity: 1, on_line: false, cascade: false, stop: Some(679) },
        Op::Correction { pick: 42, entity: 1, on_line: true, cascade: false, stop: None },
        Op::Correction { pick: 14, entity: 0, on_line: false, cascade: false, stop: None },
        Op::OutOfBand([Set(1), Set(2)]),
    ];
    if let Err(failure) = run(&mut world::Document::attach_chain(&host), &ops, None) {
        panic!("#429 chain ({ops:?}) disagrees:\n{failure:#?}");
    }
    assert!(equivalence.shadow_checks_for_test() > 0, "the armed shadow must check a certification");
}

/// Court 10, condition 3: `dst-mid`'s `current_output` read is a
/// `FamilySelection` fact, which marks cannot certify (`UnsupportedFact`).
/// Readmission compared the chain in full at the forked head, so it still
/// readmits the chain there.
#[test]
fn readmission_accepts_unsupported_only_when_compared() {
    let counts = super::readmission::court("unsupported readmission", &super::readmission::PREFIX, &[]);
    assert!(counts.readmitted > 0, "a fully compared chain with an UnsupportedFact row must readmit");
}

/// Adds `dst-b`'s bindings to `dst-a`'s output family before the first call
/// after the undo, so the family `dst-mid` reads for `e1` has a second
/// member, `dst-b(e1)`, at the forked head.
struct SiblingAfterUndo<'h> {
    document: world::Document<'h>,
    host: &'h super::fixture::reads::Host,
    sibling: bool,
    undone: bool,
    /// Consumer readmissions verified current on this thread when the first
    /// call after the undo began.
    from: Option<u64>,
}

impl System for SiblingAfterUndo<'_> {
    fn act(&mut self, action: &super::reference::Action) -> super::reference::Outcome {
        if self.undone && self.from.is_none() {
            if self.sibling {
                let graph = self.host.runtime().runtime_for_test().primary_provider_for_test().graph_for_test();
                let mut lineage = graph.output_lineage.lock().unwrap();
                let families = lineage.output_families_mut_for_test();
                let sibling = families.get("dst-b").expect("dst-b's family is installed").clone();
                families.get_mut("dst-a").expect("dst-a's family is installed").extend(sibling);
            }
            self.from = Some(verified_consumers());
        }
        self.undone |= matches!(action, super::reference::Action::Undo);
        self.document.act(action)
    }

    fn observe(&mut self, limit: u32) -> super::reference::Observed {
        self.document.observe(limit)
    }

    fn nodes(&self) -> u32 {
        self.document.nodes()
    }

    fn release(&mut self) {
        self.document.release()
    }
}

/// Consumer readmissions that verified current; the armed decline door
/// counts each one and declines it, so the consumer executes afresh.
fn verified_consumers() -> u64 {
    worth_query_host::facade::primary_graph::declined_historical_readmissions_on_this_thread_for_test()
}

/// The consumer readmissions verified current from the first call after the
/// undo of `ops`, and the message of a panic that stopped the run.
fn verified_after_undo(ops: &[Op], sibling: bool) -> (u64, Option<String>) {
    let host = world::install_chain();
    let mut system = SiblingAfterUndo {
        document: world::Document::attach_chain(&host),
        host: &host,
        sibling,
        undone: false,
        from: None,
    };
    let stopped = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&mut system, ops, None))) {
        Ok(Ok(())) => None,
        Ok(Err(failure)) => panic!("#429 sibling={sibling} ({ops:?}) disagrees:
{failure:#?}"),
        Err(payload) => Some(
            payload
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_else(|| panic!("#429 sibling={sibling}: the run panicked without a message")),
        ),
    };
    (verified_consumers() - system.from.expect("the court calls after its undo"), stopped)
}

/// S1 STOP 3, negative: a chain row its readmission verified stands in only
/// for itself. With `dst-b(e1)` in `dst-a`'s family at the forked head, the
/// family `dst-mid` read for `e1` is `Ambiguous` there, so its recorded
/// `Unique` selection is `Moved`: no consumer readmission verifies current
/// and `dst-mid` executes afresh. Without the sibling the same calls verify
/// the chain, and the run agrees with the model. The decline door counts each
/// verified consumer readmission and declines it.
///
/// The family is widened by the test door after install, so the program's
/// demand order does not know `dst-mid` now reads `dst-b`: `dst-mid`'s fresh
/// execution reads `dst-b(e1)`'s transported row, which that row's own
/// readmission supersedes in the same settle. The edit then answers the
/// retryable `PublicationStale`, which the model does not name; the court
/// asserts that refusal and nothing after it. An overlay that answered for the
/// whole family instead lets native selection take the transported `dst-mid`,
/// and the edit is refused with `dst-leaf`'s `StaleSource` on it.
#[test]
fn readmission_refuses_an_ambiguous_producer_family() {
    let equivalence =
        worth_query_host::facade::primary_graph::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    worth_query_host::facade::primary_graph::decline_historical_readmissions_on_this_thread_for_test(true);
    let ops = [
        Op::Edit([Set(2), Skip]),
        Op::Edit([Set(1), Skip]),
        Op::Undo,
        Op::Edit([Skip, Set(1)]),
        Op::OutOfBand([Skip, Set(2)]),
    ];
    let (control, stopped) = verified_after_undo(&ops, false);
    assert_eq!(stopped, None, "without a sibling the run agrees with the model");
    assert!(control > 0, "without a sibling the forked head verifies the chain current");
    assert!(equivalence.shadow_checks_for_test() > 0, "the armed shadow must check a certification");
    let (ambiguous, stopped) = verified_after_undo(&ops, true);
    assert_eq!(ambiguous, 0, "an Ambiguous producer family at the forked head must not verify current");
    if let Some(stopped) = stopped {
        assert!(
            stopped.contains("PublicationStale") && stopped.contains("superseded before publication"),
            "the only refusal is the retryable stale upstream: {stopped}"
        );
    }
}
