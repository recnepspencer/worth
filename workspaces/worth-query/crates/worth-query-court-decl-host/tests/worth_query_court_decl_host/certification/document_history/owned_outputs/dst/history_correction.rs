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

/// Cost law 2 for the selected-line check (plan §10.1a): correcting the
/// oldest step under 2, 12 and 72 later steps, the journal's added node
/// probes over the 60 further steps stay within 3x those over the first 10.
/// A walk of the selected line would grow by 60 against 10.
#[test]
fn a_correction_costs_logarithmic_journal_work_in_history_depth() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_roomy();
    let mut probes = Vec::new();
    for depth in [2u32, 12, 72] {
        let mut document = created(&host, &[E1]);
        for step in 0..depth {
            let value = if step % 2 == 0 { 2 } else { 1 };
            let edit = Action::Envelope { key: 2 + step, intent: Intent::Edit([Stage::Set(value), Stage::Skip]), stop: None };
            assert_eq!(document.act(&edit), Outcome::Recorded { node: 2 + step });
            // Release the past heads each step queues; the world's branch
            // capacity is small.
            if step % 4 == 3 {
                document.act(&Action::Maintain);
            }
        }
        document.act(&Action::Maintain);
        let (corrected, node_probes) = document.node_probes_during(|document| document.correct_recorded(1000, 1, E1, &SKIP));
        assert!(matches!(corrected, Corrected::Recorded(node) if node == depth + 2), "depth {depth}: {corrected:?}");
        probes.push(node_probes);
        document.release();
    }
    eprintln!("correction journal node probes at depths 2, 12, 72: {probes:?}");
    let (near, mid, far) = (probes[0], probes[1], probes[2]);
    assert!(mid >= near && far >= mid, "probes grow with depth: {probes:?}");
    assert!(far - mid <= 3 * (mid - near), "node probes at depths 2, 12, 72: {probes:?}");
}

/// S3: after an undo the document stands on the navigation's fork, and a
/// step still on its line remains a correction source: reversing `e1`'s
/// creation (step 1) records as one step there.
#[test]
fn a_step_still_on_the_line_is_corrected_after_an_undo() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = created(&host, &[E1, E2]);
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    let corrected = document.correct_recorded(3, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(3)), "{corrected:?}");
    let observed = document.observe(4);
    assert_eq!(observed.current, Some(3));
    assert_eq!(observed.head_diverged, Some(false));
    assert_eq!(observed.branch.as_ref().map(|branch| branch.live), Some([false, false]));
    document.release();
}

/// A step an `.edit` writer recorded is a correction source, as an
/// envelope's step is: its commit is indexed at the writer's adoption.
#[test]
fn a_step_an_edit_writer_recorded_is_corrected() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = world::Document::attach(&host);
    assert_eq!(document.act(&Action::Begin(super::reference::Writer::Edit)), Outcome::Begun);
    let commit = Action::Commit { key: 1, stages: [Stage::Set(1), Stage::Skip] };
    assert_eq!(document.act(&commit), Outcome::Recorded { node: 1 });
    let corrected = document.correct_recorded(2, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(2)), "{corrected:?}");
    assert_eq!(document.observe(4).branch.as_ref().map(|branch| branch.live), Some([false, false]));
    document.release();
}

/// Each member of a step a gesture recorded is a correction source: every
/// member's commit is indexed at the group's adoption. The gesture creates
/// `e1` then `e2`; reversing `e2`'s creation (the second member) records.
#[test]
fn a_member_of_a_step_a_gesture_recorded_is_corrected() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = world::Document::attach(&host);
    assert_eq!(document.act(&Action::Begin(super::reference::Writer::Gesture)), Outcome::Begun);
    let commit = Action::Commit { key: 1, stages: [Stage::Set(1), Stage::Set(1)] };
    assert_eq!(document.act(&commit), Outcome::Recorded { node: 1 });
    let corrected = document.correct_recorded(2, 1, E2, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(2)), "{corrected:?}");
    assert_eq!(document.observe(4).branch.as_ref().map(|branch| branch.live), Some([true, false]));
    document.release();
}

/// Public #368 released: the second stage of a multi-stage step is a
/// correction source through the document too.
#[test]
fn a_later_stage_of_a_step_is_corrected_through_the_document() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = world::Document::attach(&host);
    let both = Action::Envelope { key: 1, intent: Intent::Edit([Stage::Set(1), Stage::Set(1)]), stop: None };
    assert_eq!(document.act(&both), Outcome::Recorded { node: 1 });
    let corrected = document.correct_recorded(2, 1, E2, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(2)), "{corrected:?}");
    assert_eq!(document.observe(4).branch.as_ref().map(|branch| branch.live), Some([true, false]));
    document.release();
}

/// A correction's meaning names the stage it reverses (its commit member):
/// one key on the second stage of a step records, the same key on the first
/// stage drifts, and the same key on the second stage again replays.
#[test]
fn one_key_on_another_stage_of_the_same_step_drifts() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = world::Document::attach(&host);
    let both = Action::Envelope { key: 1, intent: Intent::Edit([Stage::Set(1), Stage::Set(1)]), stop: None };
    assert_eq!(document.act(&both), Outcome::Recorded { node: 1 });
    assert!(matches!(document.correct_recorded(2, 1, E2, &SKIP), Corrected::Recorded(2)));
    let other = document.correct_recorded(2, 1, E1, &SKIP);
    assert!(matches!(&other, Corrected::Refused(denial) if denial.contains("IntentDrift")), "{other:?}");
    let replay = document.correct_recorded(2, 1, E2, &SKIP);
    assert!(matches!(replay, Corrected::AlreadyRecorded(2)), "{replay:?}");
    document.release();
}

/// Phase 2 confines corrections to an owned-model-graph history. A
/// correction envelope on a scope-record history refuses
/// `CorrectionRequiresOwnedModelGraph` when it opens, before anything else,
/// and returns the document unchanged: it records no step and closes
/// cleanly. (Navigation across a recorded correction refuses again, defense
/// in depth.)
#[test]
fn a_correction_envelope_on_a_scope_record_history_refuses_at_open() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut owned = created(&host, &[E1]);
    let (source, _) = owned.recorded_source(1, E1);
    let scope = fixture::history_correction::attach_scope_history(&host);
    let (scope, opened) =
        fixture::history_correction::open_on_scope_history(&host, scope, owned.branch(), "scope-k1".into(), &source);
    assert_eq!(opened, Err("History(Document(CorrectionRequiresOwnedModelGraph))".to_owned()));
    let page = fixture::reads::with_request(&host, |request| {
        let current = scope.current_step(request).unwrap();
        (current.parent().is_none(), current.id())
    });
    assert!(page.0, "the refused open recorded no step: the document is still at its root");
    host.with_host_advancement(|phase| scope.close(&phase).unwrap()).unwrap();
    // A kept commit receipt retains its history: let it go before close.
    drop(source);
    owned.release();
}
