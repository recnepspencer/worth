//! S3 ruling B, refined: a correction's restored support across the
//! document's own navigation commits. A navigation restores the full
//! recorded state, so the latest navigation commit before the target decides
//! and supersedes earlier ones; its occurrences count as cancelled only if
//! the basis it selected is on the selected line and descends from the
//! correction's source. Every other occurrence still needs its witness.
use super::reference::{Action, Intent, Outcome, Stage, Stages, E1, E2};
use super::world::Corrected;
use super::*;
use worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test as fact_verification;

const SKIP: Stages = [Stage::Skip; 2];

fn edit(key: u32, e1: Stage, e2: Stage) -> Action {
    Action::Envelope { key, intent: Intent::Edit([e1, e2]), stop: None }
}

/// Record `steps` as edits from key 1, each expected to be the next node.
fn recorded<'h>(host: &'h fixture::reads::Host, steps: &[[Stage; 2]]) -> world::Document<'h> {
    let mut document = world::Document::attach(host);
    for (index, [e1, e2]) in steps.iter().enumerate() {
        let node = index as u32 + 1;
        assert_eq!(document.act(&edit(node, *e1, *e2)), Outcome::Recorded { node }, "step {node}");
    }
    document
}

fn refused(corrected: &Corrected, denial: &str) -> bool {
    matches!(corrected, Corrected::Refused(text) if text.contains(denial))
}

/// The restored-support stage that refused this thread's last correction.
fn refusing_stage() -> Option<String> {
    fact_verification().correction_restored_support_rejection
}

/// e2 created, retired, created again and retired again on one line: the
/// source (step 2) and the selected basis (step 4) bracket two line writes
/// that change e2's liveness (review r1 N2).
const RELIVED: [[Stage; 2]; 4] = [
    [Stage::Set(1), Stage::Set(1)],
    [Stage::Skip, Stage::Retire],
    [Stage::Skip, Stage::Set(1)],
    [Stage::Skip, Stage::Retire],
];

/// Record [`RELIVED`], run `moves`, then reverse step 2's retirement of e2.
/// The model refuses (e2 is live at node 3, in `Since(2)`), and the platform
/// must refuse at the canonical cancellation, never by the facts comparison.
fn relived_and_reretired_refuses(moves: &[Action]) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &RELIVED);
    for (index, action) in moves.iter().enumerate() {
        assert_eq!(document.act(action), Outcome::Moved, "move {index}");
    }
    fact_verification();
    let corrected = document.correct_recorded(5, 2, E2, &SKIP);
    let stage = refusing_stage();
    assert!(refused(&corrected, "SourceDiverged"), "{corrected:?}; {stage:?}");
    assert!(stage.as_deref().is_some_and(|stage| stage.starts_with("stage=canonical-cancellation")), "{stage:?}");
    document.release();
}

// ---- #453: supersession never passes over a line write ----

/// H: an undo and a redo of the latest step. The redo's basis descends from
/// the source, but the retirement's re-creation and re-retirement are line
/// writes between them, so they keep needing their witnesses.
#[test]
fn a_retirement_relived_and_reretired_on_the_line_refuses() {
    relived_and_reretired_refuses(&[Action::Undo, Action::Redo(4)]);
}

/// C1: the same line, then a checkout behind the source and back.
#[test]
fn a_retirement_relived_and_reretired_on_the_line_refuses_after_a_checkout_behind_it_and_back() {
    relived_and_reretired_refuses(&[Action::Checkout(1), Action::Checkout(4)]);
}

/// C2: the same line, then three undos and three redos.
#[test]
fn a_retirement_relived_and_reretired_on_the_line_refuses_after_three_undos_and_redos() {
    let undos = [Action::Undo, Action::Undo, Action::Undo];
    relived_and_reretired_refuses(&[undos.as_slice(), &[Action::Redo(2), Action::Redo(3), Action::Redo(4)]].concat());
}

/// P-off: step 2's retirement of e1 descends from the source (step 1) but is
/// not on the line up to the selected basis (the undo selects step 1), so the
/// undo supersedes it and e1's creation is reversed.
#[test]
fn a_retirement_off_the_restored_line_does_not_block_reversing_the_creation() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &[[Stage::Set(1), Stage::Skip], [Stage::Retire, Stage::Skip]]);
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    let corrected = document.correct_recorded(3, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(3)), "{corrected:?}");
    document.release();
}

/// A lineage a correction took can outlive its document: after the real
/// close it answers without panicking, vouches for nothing, and keeps every
/// write (it cannot tell a line write from another).
#[test]
fn a_lineage_held_past_its_documents_close_answers_without_panicking() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &[[Stage::Set(1), Stage::Skip]]);
    assert_eq!(document.vouch_after_close(), (false, true));
}

// ---- positive courts ----

/// The genuinely changed case: undo a set of `e1`, then reverse `e1`'s
/// creation. The navigation wrote `e1` back to its created value.
#[test]
fn undoing_a_set_then_reversing_the_creation_records() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &[[Stage::Set(1), Stage::Skip], [Stage::Set(2), Stage::Skip]]);
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    let corrected = document.correct_recorded(3, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(3)), "{corrected:?}");
    let observed = document.observe(4);
    assert_eq!(observed.branch.as_ref().map(|branch| branch.live), Some([false, false]));
    assert_eq!(observed.head_diverged, Some(false));
    document.release();
}

/// Undo, then redo to a step whose value equals the created one, then
/// reverse the creation: both navigation commits selected descendants.
#[test]
fn a_creation_is_reversed_after_an_undo_and_a_redo() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let steps = [[Stage::Set(1), Stage::Skip], [Stage::Set(2), Stage::Skip], [Stage::Set(1), Stage::Skip]];
    let mut document = recorded(&host, &steps);
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    assert_eq!(document.act(&Action::Redo(3)), Outcome::Moved);
    let corrected = document.correct_recorded(4, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(4)), "{corrected:?}");
    document.release();
}

/// A checkout to the source step itself, then the reversal of its creation.
#[test]
fn a_creation_is_reversed_after_a_checkout_to_its_step() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let steps = [[Stage::Set(1), Stage::Skip], [Stage::Set(2), Stage::Skip], [Stage::Set(1), Stage::Skip]];
    let mut document = recorded(&host, &steps);
    assert_eq!(document.act(&Action::Checkout(1)), Outcome::Moved);
    let corrected = document.correct_recorded(4, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(4)), "{corrected:?}");
    document.release();
}

// ---- negative courts: each must still refuse ----

/// (a) After an undo, a write outside the document diverges its head: the
/// correction refuses `HeadDiverged`, whatever its restored support says.
#[test]
fn a_write_outside_after_an_undo_still_refuses_the_correction() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &[[Stage::Set(1), Stage::Skip], [Stage::Set(2), Stage::Skip]]);
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    assert_eq!(document.act(&Action::OutOfBand([Stage::Set(2), Stage::Skip])), Outcome::OutOfBand);
    let corrected = document.correct_recorded(3, 1, E1, &SKIP);
    assert!(refused(&corrected, "HeadDiverged"), "{corrected:?}");
    document.release();
}

/// (c) ABA: navigate away, write outside, navigate back. The write outside
/// diverges the head, so the navigation back and the correction refuse.
#[test]
fn navigating_away_writing_outside_and_back_still_refuses_the_correction() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &[[Stage::Set(1), Stage::Skip], [Stage::Skip, Stage::Set(1)]]);
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    assert_eq!(document.act(&Action::OutOfBand([Stage::Set(2), Stage::Skip])), Outcome::OutOfBand);
    assert_eq!(document.act(&Action::Redo(2)), Outcome::HeadDiverged);
    let corrected = document.correct_recorded(3, 1, E1, &SKIP);
    assert!(refused(&corrected, "HeadDiverged"), "{corrected:?}");
    document.release();
}

/// (d) A navigation commit that restored a state other than the one the
/// source made at the compared facts: the navigation selects a step where
/// `e1` is a later occurrence (retired, then created again), so the
/// occurrence the source created is not the one present. The navigation is
/// vouched for, but the retirement and re-creation on the way are line
/// writes (public #453), so they keep needing their witnesses and the
/// cancellation refuses `SourceDiverged` before the final comparison.
#[test]
fn a_navigation_that_restored_another_occurrence_still_refuses_the_correction() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let steps = [
        [Stage::Set(1), Stage::Skip],
        [Stage::Retire, Stage::Skip],
        [Stage::Set(1), Stage::Skip],
        [Stage::Set(2), Stage::Skip],
    ];
    let mut document = recorded(&host, &steps);
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    let corrected = document.correct_recorded(5, 1, E1, &SKIP);
    let stage = refusing_stage();
    assert!(refused(&corrected, "SourceDiverged"), "{corrected:?}; {stage:?}");
    assert!(stage.as_deref().is_some_and(|stage| stage.starts_with("stage=canonical-cancellation")), "{stage:?}");
    document.release();
}

// ---- refined rule: the latest navigation decides ----

/// Create the source, undo it, redo it, then reverse it: CAD's create,
/// undo, redo, delete. The undo's commit is superseded by the redo's.
#[test]
fn the_source_is_reversed_after_its_undo_and_redo() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &[[Stage::Set(1), Stage::Skip]]);
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    assert_eq!(document.act(&Action::Redo(1)), Outcome::Moved);
    let corrected = document.correct_recorded(2, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(2)), "{corrected:?}");
    assert_eq!(document.observe(4).branch.as_ref().map(|branch| branch.live), Some([false, false]));
    document.release();
}

/// Twice undone and redone, then reversed.
#[test]
fn the_source_is_reversed_after_two_undos_and_redos() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &[[Stage::Set(1), Stage::Skip]]);
    for _ in 0..2 {
        assert_eq!(document.act(&Action::Undo), Outcome::Moved);
        assert_eq!(document.act(&Action::Redo(1)), Outcome::Moved);
    }
    let corrected = document.correct_recorded(2, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(2)), "{corrected:?}");
    document.release();
}

/// A checkout away from the source and back to it, then its reversal: the
/// checkout back decides and supersedes the checkout away.
#[test]
fn the_source_is_reversed_after_a_checkout_away_and_back() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &[[Stage::Set(1), Stage::Skip], [Stage::Skip, Stage::Set(1)]]);
    assert_eq!(document.act(&Action::Checkout(1)), Outcome::Moved);
    assert_eq!(document.act(&Action::Checkout(2)), Outcome::Moved);
    let corrected = document.correct_recorded(3, 2, E2, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(3)), "{corrected:?}");
    document.release();
}

/// Undo, redo, undo: the document ends away from the source, which is off
/// the selected line.
#[test]
fn the_source_ending_undone_is_not_reversed() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &[[Stage::Set(1), Stage::Skip]]);
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    assert_eq!(document.act(&Action::Redo(1)), Outcome::Moved);
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    let corrected = document.correct_recorded(2, 1, E1, &SKIP);
    assert!(refused(&corrected, "SourceNotOnSelectedLine"), "{corrected:?}");
    document.release();
}

/// Away, an application write that creates `e2` again on the other branch,
/// then back to the source: the navigation back decides the restored state,
/// and the witnessed write is covered, so the reversal records.
#[test]
fn a_write_on_the_branch_away_does_not_block_the_reversal_after_coming_back() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &[[Stage::Set(1), Stage::Skip], [Stage::Skip, Stage::Set(1)]]);
    assert_eq!(document.act(&Action::Checkout(1)), Outcome::Moved);
    assert_eq!(document.act(&edit(3, Stage::Skip, Stage::Set(2))), Outcome::Recorded { node: 3 });
    assert_eq!(document.act(&Action::Checkout(2)), Outcome::Moved);
    let corrected = document.correct_recorded(4, 2, E2, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(4)), "{corrected:?}");
    document.release();
}

/// Back at the source, an application write after the latest navigation that
/// retires `e2`: the occurrence the source created is gone, and the reversal
/// refuses.
#[test]
fn a_write_after_the_latest_navigation_still_decides() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &[[Stage::Set(1), Stage::Skip], [Stage::Skip, Stage::Set(1)]]);
    assert_eq!(document.act(&Action::Checkout(1)), Outcome::Moved);
    assert_eq!(document.act(&Action::Checkout(2)), Outcome::Moved);
    assert_eq!(document.act(&edit(3, Stage::Skip, Stage::Retire)), Outcome::Recorded { node: 3 });
    let corrected = document.correct_recorded(4, 2, E2, &SKIP);
    // The retirement after the latest navigation is a write that still needs
    // its witness and is not a reversal pair: `SourceDiverged`. (Were it
    // classified as a navigation, the navigations would go unvouched and
    // refuse `CanonicalEvidenceUnavailable` instead.)
    assert!(refused(&corrected, "SourceDiverged"), "{corrected:?}");
    document.release();
}
