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
#[ignore = "held on public #429: a Missing current_output read records no fact, so the readmission cannot dirty dst-mid"]
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
