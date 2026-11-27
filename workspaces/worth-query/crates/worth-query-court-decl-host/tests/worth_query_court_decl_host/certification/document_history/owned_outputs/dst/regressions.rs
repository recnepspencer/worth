//! Seeds and shrunk choices a sweep once disagreed on, replayed on every run
//! after their cause is fixed. Each entry names the defect it pinned.
use super::workload::{Change, Op, StagePick::*};
use super::*;

/// (seed, shrunk choices, the defect).
const PINNED: &[(u64, &[Op], &str)] = &[
    (
        0x7b02_0000,
        &[Op::OutOfBand([Set(2), Set(1)]), Op::Stopped { stages: [Skip, Set(1)], at: 278 }],
        "the model refused a stop in the history read on a diverged head",
    ),
    (
        0x7b02_0039,
        &[MADE, Op::Undo, MADE, Op::OutOfBand([Skip, Set(2)])],
        "a write outside on a head recorded after an undo spun the settle to its deadline (public #353)",
    ),
    (
        0x7b02_004e,
        &[
            Op::Edit([Set(2), Skip]),
            Op::Stopped { stages: [Set(1), Set(1)], at: 685 },
            Op::Checkout { pick: 101 },
            Op::RetryStopped,
        ],
        "a retry after a checkout replayed the stopped attempt's calls, resolved where `e1` was live",
    ),
    (
        0x7b02_0035,
        &[Op::Begin { gesture: true }, Op::Edit([Set(1), Set(2)]), Op::Replay { pick: 109, change: Change::Same }],
        "an envelope under a gesture writer's key records a new step: the journal is keyed by namespace (refinement (v))",
    ),
    (
        0x7b02_0108,
        &[Op::Begin { gesture: false }, Op::Edit([Retire, Skip]), Op::Replay { pick: 122, change: Change::Stages }],
        "an envelope under an `.edit` writer's key records a new step: the journal is keyed by namespace (refinement (v))",
    ),
];

/// The public #445 pins. The paired support comparison refuses them
/// (`SourceDiverged`) until the #445 arm returns with public #465; with the
/// arm, Relational refuses the restoration (#465).
const PINNED_ON_465: &[(u64, &[Op], &str)] = &[
    (
        0x7b02_125f,
        &[
            Op::Edit([Set(1), Set(2)]),
            Op::Edit([Skip, Retire]),
            Op::Checkout { pick: 196 },
            Op::Checkout { pick: 52 },
            Op::Edit([Set(2), Set(2)]),
            Op::Edit([Skip, Set(1)]),
            Op::Checkout { pick: 106 },
            Op::Correction { pick: 220, entity: 0, on_line: true, cascade: false, stop: None },
        ],
        "a correction across the document's own navigation compared a re-retired output's commit version (public #445)",
    ),
    (
        0x7b02_181e,
        &[
            Op::Edit([Set(2), Skip]),
            Op::Edit([Retire, Skip]),
            Op::Checkout { pick: 163 },
            Op::Checkout { pick: 221 },
            Op::Correction { pick: 242, entity: 1, on_line: true, cascade: false, stop: None },
        ],
        "reversing a retirement after a checkout away and back compared its commit version (public #445)",
    ),
];

/// Each edit settles on a new product branch occurrence. The chain consumer
/// settled on the newer one keeps its consumed evidence on the upstream row
/// of the earlier one, whose source can only be disclosed on its own
/// occurrence. A write outside then executes the consumer ordinarily instead
/// of readmitting that row and refusing terminally `Superseded` (seed
/// `0x7b020024` with corrections, public #358). The write changed `dst-a`'s
/// source and the newer occurrence holds no `dst-a` row, so no equivalent
/// upstream row exists to reuse: execution is the only sound outcome, and
/// no chain is readmitted.
#[test]
fn a_chain_consumer_on_an_earlier_occurrences_upstream_settles() {
    let equivalence =
        worth_query_host::facade::primary_graph::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_chain();
    let ops = [Op::Edit([Set(2), Set(2)]), Op::Edit([Set(2), Set(2)]), Op::OutOfBand([Set(1), Set(1)])];
    if let Err(failure) = run(&mut world::Document::attach_chain(&host), &ops, None) {
        panic!("#358 chain ({ops:?}) disagrees:\n{failure:#?}");
    }
    assert!(equivalence.shadow_checks_for_test() > 0, "the armed shadow must check a certification");
    assert_eq!(
        worth_query_host::facade::primary_graph::readmitted_chains_on_this_thread_for_test(),
        0,
        "the earlier occurrence's row is never readmitted on the newer one"
    );
}

/// The first recorded edit of the forked-head cases: it creates both members.
const MADE: Op = Op::Edit([Set(2), Set(1)]);

fn replay(pins: &[(u64, &[Op], &str)]) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    for (seed, ops, defect) in pins {
        if let Err(failure) = run(&mut world::Document::attach(&host), ops, None) {
            panic!("seed {seed:#x} ({defect}) disagrees again:\n{failure:#?}");
        }
    }
}

#[test]
fn every_pinned_disagreement_stays_fixed() {
    replay(PINNED);
}

#[test]
#[ignore = "held on public #465 (with the deferred #445 arm): the paired support comparison refuses a source the navigation re-retired (SourceDiverged); with the arm back, restoration refuses because succession does not follow HistoricalAbsence"]
fn the_pins_held_on_465_stay_fixed() {
    replay(PINNED_ON_465);
}

/// Public #353: after an undo, a new record forks the history, and the
/// restored outputs' marks do not answer at the forked head. A write outside
/// the document there must still let the runtime settle at once, as without
/// the fork. Each shape once spun the program-output settle until the call
/// deadline refused it; the pinned seed above is the write to `e2`.
#[test]
fn a_write_outside_on_a_forked_head_settles() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let shapes: [&[Op]; 3] = [
        &[MADE, Op::Undo, MADE, Op::OutOfBand([Skip, Set(1)])],
        &[MADE, Op::Undo, MADE, Op::OutOfBand([Set(1), Skip])],
        &[MADE, Op::Edit([Set(1), Skip]), Op::Undo, Op::Edit([Skip, Set(2)]), Op::OutOfBand([Set(2), Skip])],
    ];
    for ops in shapes {
        if let Err(failure) = run(&mut world::Document::attach(&host), ops, None) {
            panic!("#353 ({ops:?}) disagrees:
{failure:#?}");
        }
    }
}

/// Public #358: [`a_write_outside_on_a_forked_head_settles`]'s first shape
/// with a `dst-use` output that reads the root through `current_output`.
/// Readmitted at the forked head with its chain, the consumer's evidence is
/// re-identified at its own selection commit, so it certifies there instead
/// of spinning to the call deadline on the abandoned line.
#[test]
fn a_consumer_on_a_forked_head_settles() {
    let equivalence =
        worth_query_host::facade::primary_graph::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_consumer();
    let ops = [MADE, Op::Undo, MADE, Op::OutOfBand([Skip, Set(1)])];
    if let Err(failure) = run(&mut world::Document::attach_consumer(&host), &ops, None) {
        panic!("#358 ({ops:?}) disagrees:
{failure:#?}");
    }
    assert!(equivalence.shadow_checks_for_test() > 0, "the armed shadow must check a certification");
    assert!(
        worth_query_host::facade::primary_graph::readmitted_chains_on_this_thread_for_test() > 0,
        "the consumer must be readmitted with its chain, not executed again"
    );
}

/// One envelope that sets `e1` then retires `e2`, or retires both, on the
/// document and outside it. The retirement is a later stage in each (child
/// first, a set before a retirement), which public #369 postpones: since
/// #348 (PR #374) each refuses typed `LaterStageRetirementPostponed { issue:
/// 369 }` where the model records, and `workload::held` keeps the pairs out
/// of the sweep.
#[test]
#[ignore = "held on public #369: a retirement in a later stage of one envelope refuses LaterStageRetirementPostponed { issue: 369 } (per-stage retirement events)"]
fn an_owner_retirement_beside_another_stage_in_one_envelope_agrees() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let created = Op::Edit([Set(1), Set(1)]);
    for pair in [
        Op::Edit([Set(2), Retire]),
        Op::OutOfBand([Set(2), Retire]),
        Op::Edit([Retire, Retire]),
        Op::OutOfBand([Retire, Retire]),
    ] {
        if let Err(failure) = run(&mut world::Document::attach(&host), &[created, pair], None) {
            panic!("#348 ({pair:?}) disagrees:
{failure:#?}");
        }
    }
}

/// The consumer's output settled on `main` is read again when an edit settles
/// on its product branch. The marks find that evidence unaligned and the
/// consumer executes again; the armed shadow's full verification once called
/// the same evidence foreign, and the equivalence check refused the edit
/// `RetainedBasisUnavailable` instead (found arming the #358 consumer courts).
#[test]
fn an_armed_consumer_settles_an_edit_on_its_product_branch() {
    let equivalence =
        worth_query_host::facade::primary_graph::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_consumer();
    let ops = [MADE];
    if let Err(failure) = run(&mut world::Document::attach_consumer(&host), &ops, None) {
        panic!("an armed consumer ({ops:?}) disagrees:
{failure:#?}");
    }
    assert!(equivalence.shadow_checks_for_test() > 0, "the armed shadow must check a certification");
}
