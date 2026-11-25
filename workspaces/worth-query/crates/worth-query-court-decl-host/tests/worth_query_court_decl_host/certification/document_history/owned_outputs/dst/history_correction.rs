//! S2 courts: a correction recorded by the document as one step
//! (`document.correction_envelope`, plan §S2, §10.1 and §11.1).
use super::reference::{Action, Intent, Outcome, Stage, Stages, E1, E2};
use super::world::Corrected;
use super::*;

const SKIP: Stages = [Stage::Skip; 2];

/// A document whose steps create each of `creations` in turn, one member per
/// step (`e1` at step 1, `e2` at step 2).
fn created<'h>(host: &'h fixture::reads::Host, creations: &[usize]) -> world::Document<'h> {
    let mut document = world::Document::attach(host);
    for (step, &entity) in creations.iter().enumerate() {
        let mut stages = SKIP;
        stages[entity] = Stage::Set(1);
        let action = Action::Envelope { key: step as u32 + 1, intent: Intent::Edit(stages), stop: None };
        assert_eq!(document.act(&action), Outcome::Recorded { node: step as u32 + 1 });
    }
    document
}

/// Reversing a recorded creation through the document is one step: the head
/// moves to it, does not diverge, and the document keeps editing.
#[test]
fn a_recorded_creation_reversal_is_one_step_and_the_document_keeps_editing() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = created(&host, &[E1, E2]);
    let corrected = document.correct_recorded(3, 2, E2, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(3)), "{corrected:?}");
    let observed = document.observe(4);
    assert_eq!(observed.current, Some(3));
    assert_eq!(observed.head_diverged, Some(false));
    assert_eq!(observed.branch.as_ref().map(|branch| branch.live), Some([true, false]));
    let edit = Action::Envelope { key: 4, intent: Intent::Edit([Stage::Set(2), Stage::Skip]), stop: None };
    assert_eq!(document.act(&edit), Outcome::Recorded { node: 4 });
    document.release();
}

/// The same key and the same correction answer the recorded step, at once
/// and after later steps and navigation away and back (plan §10.1).
#[test]
fn a_correction_replays_after_later_steps_and_navigation() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = created(&host, &[E1, E2]);
    assert!(matches!(document.correct_recorded(3, 2, E2, &SKIP), Corrected::Recorded(3)));
    let replay = document.correct_recorded(3, 2, E2, &SKIP);
    assert!(matches!(replay, Corrected::AlreadyRecorded(3)), "at once: {replay:?}");
    let edit = Action::Envelope { key: 4, intent: Intent::Edit([Stage::Set(2), Stage::Skip]), stop: None };
    assert_eq!(document.act(&edit), Outcome::Recorded { node: 4 });
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    assert_eq!(document.act(&Action::Redo(4)), Outcome::Moved);
    let replay = document.correct_recorded(3, 2, E2, &SKIP);
    assert!(matches!(replay, Corrected::AlreadyRecorded(3)), "after navigation: {replay:?}");
    assert_eq!(document.observe(4).current, Some(4), "a replay leaves the head where it was");
    document.release();
}

/// An undone step is not on the selected line, so it is no correction
/// source: refused before any reservation, with no new step (plan §10.1a).
#[test]
fn an_undone_step_is_not_a_correction_source() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = created(&host, &[E1, E2]);
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    let refused = document.correct_recorded(3, 2, E2, &SKIP);
    assert!(
        matches!(&refused, Corrected::Refused(denial) if denial.contains("SourceNotOnSelectedLine")),
        "{refused:?}"
    );
    let observed = document.observe(4);
    assert_eq!(observed.current, Some(1));
    assert_eq!(observed.head_diverged, Some(false));
    let rows = observed.history.map(|history| history.pages.concat().len());
    assert_eq!(rows, Some(3), "no step was reserved or recorded: the root and steps 1 and 2");
    document.release();
}

/// A recorded correction key with other coupled stages is another meaning:
/// `IntentDrift`, before anything runs (plan §10.1, kept from r1).
#[test]
fn a_correction_with_other_coupled_stages_drifts() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = created(&host, &[E1, E2]);
    let retire = Action::Envelope { key: 3, intent: Intent::Edit([Stage::Skip, Stage::Retire]), stop: None };
    assert_eq!(document.act(&retire), Outcome::Recorded { node: 3 });
    assert!(matches!(document.correct_recorded(4, 3, E2, &SKIP), Corrected::Recorded(4)));
    let drift = document.correct_recorded(4, 3, E2, &[Stage::Set(2), Stage::Skip]);
    assert!(matches!(&drift, Corrected::Refused(denial) if denial.contains("IntentDrift")), "{drift:?}");
    assert_eq!(document.observe(4).current, Some(4));
    document.release();
}
