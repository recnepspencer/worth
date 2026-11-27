//! Public #445: reversing a retirement after the document's own navigation
//! revived the retired generation and retired it again. The navigation
//! restores the recorded state, so the same generation is still retired at
//! the target and the reversal records; only the retirement's commit version
//! moved. Each refusal court still refuses, for its own reason.
use super::reference::{Action, Intent, Outcome, Stage, Stages, E2};
use super::world::Corrected;
use super::*;
use worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test as fact_verification;

const SKIP: Stages = [Stage::Skip; 2];

/// Step 1 creates `e1` and `e2`; step 2 retires `e2`.
const RETIRED: [[Stage; 2]; 2] = [[Stage::Set(1), Stage::Set(1)], [Stage::Skip, Stage::Retire]];

fn edit(key: u32, e1: Stage, e2: Stage) -> Action {
    Action::Envelope {
        key,
        intent: Intent::Edit([e1, e2]),
        stop: None,
    }
}

/// Record `steps` as edits from key 1, each expected to be the next node.
fn recorded<'h>(host: &'h fixture::reads::Host, steps: &[[Stage; 2]]) -> world::Document<'h> {
    let mut document = world::Document::attach(host);
    for (index, [e1, e2]) in steps.iter().enumerate() {
        let node = index as u32 + 1;
        assert_eq!(
            document.act(&edit(node, *e1, *e2)),
            Outcome::Recorded { node },
            "step {node}"
        );
    }
    document
}

/// Record [`RETIRED`], run `moves` (each a navigation or the next recorded
/// edit), then reverse step 2's retirement of `e2` under `key`. It records,
/// restores `e2` and its outputs, and leaves the head undiverged.
fn reversed_after(moves: &[(Action, Outcome)], key: u32) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &RETIRED);
    for (index, (action, expected)) in moves.iter().enumerate() {
        assert_eq!(document.act(action), *expected, "move {index}");
    }
    fact_verification();
    let corrected = document.correct_recorded(key, 2, E2, &SKIP);
    let rejection = fact_verification().correction_restored_support_rejection;
    assert!(
        matches!(corrected, Corrected::Recorded(node) if node == key),
        "{corrected:?}; {rejection:?}"
    );
    assert_eq!(rejection, None);
    let observed = document.observe(8);
    assert_eq!(
        observed.branch.as_ref().map(|branch| branch.live),
        Some([true, true]),
        "e2 is live again"
    );
    assert!(
        observed
            .outputs
            .is_some_and(|(a, b)| a[E2] != 0 && b[E2] != 0),
        "{:?}",
        observed.outputs
    );
    assert_eq!(observed.head_diverged, Some(false));
    document.release();
}

const MOVED: Outcome = Outcome::Moved;

/// P1: undo the retirement, redo it, then reverse it.
#[test]
fn a_retirement_is_reversed_after_its_undo_and_redo() {
    reversed_after(&[(Action::Undo, MOVED), (Action::Redo(2), MOVED)], 3);
}

/// P2: a checkout to the step before the retirement and back.
#[test]
fn a_retirement_is_reversed_after_a_checkout_away_and_back() {
    reversed_after(
        &[(Action::Checkout(1), MOVED), (Action::Checkout(2), MOVED)],
        3,
    );
}

/// P3: through the root, then the step before, then back.
#[test]
fn a_retirement_is_reversed_after_a_checkout_through_the_root() {
    let moves = [
        (Action::Checkout(0), MOVED),
        (Action::Checkout(1), MOVED),
        (Action::Checkout(2), MOVED),
    ];
    reversed_after(&moves, 3);
}

/// P4: two undos and two redos.
#[test]
fn a_retirement_is_reversed_after_two_undos_and_redos() {
    let moves = [
        (Action::Undo, MOVED),
        (Action::Undo, MOVED),
        (Action::Redo(1), MOVED),
        (Action::Redo(2), MOVED),
    ];
    reversed_after(&moves, 3);
}

/// P5: a sibling edit of `e1` on the way (seed 0x7b02125f's shape). The
/// edit forks a branch the final checkout leaves.
#[test]
fn a_retirement_is_reversed_with_a_sibling_edit_on_the_way() {
    let moves = [
        (Action::Checkout(1), MOVED),
        (
            edit(3, Stage::Set(2), Stage::Skip),
            Outcome::Recorded { node: 3 },
        ),
        (Action::Checkout(2), MOVED),
    ];
    reversed_after(&moves, 4);
}

/// P6: a checkout to the root, where `e2` does not exist, and straight back:
/// the retired generation is never revived, so its version never moves.
#[test]
fn a_retirement_is_reversed_after_a_checkout_that_never_revived_it() {
    reversed_after(
        &[(Action::Checkout(0), MOVED), (Action::Checkout(2), MOVED)],
        3,
    );
}

/// P8: a checkout round trip, then an unrelated step on the line, its undo
/// and its redo (review r3's fallback shape). Both occurrences after the
/// unrelated step are navigation commits; the moved retirement version is
/// the only difference the comparison sees.
#[test]
fn a_retirement_is_reversed_after_a_checkout_round_trip_and_an_unrelated_line_step() {
    let moves = [
        (Action::Checkout(1), MOVED),
        (Action::Checkout(2), MOVED),
        (
            edit(3, Stage::Set(2), Stage::Skip),
            Outcome::Recorded { node: 3 },
        ),
        (Action::Undo, MOVED),
        (Action::Redo(3), MOVED),
    ];
    reversed_after(&moves, 4);
}

fn refused(corrected: &Corrected, denial: &str) -> bool {
    matches!(corrected, Corrected::Refused(text) if text.contains(denial))
}

/// N2-live: `e2` is created again after its retirement and is live at the
/// target. The model refuses (the new occurrence is in `Since(2)`). The
/// re-creation is a line write the redo cannot supersede (public #453), so
/// the platform refuses at the canonical cancellation; before #453 it
/// refused later, at the comparison, on the member's indexed selection.
#[test]
fn a_retirement_followed_by_a_live_new_occurrence_refuses() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(
        &host,
        &[RETIRED[0], RETIRED[1], [Stage::Skip, Stage::Set(1)]],
    );
    assert_eq!(document.act(&Action::Undo), MOVED);
    assert_eq!(document.act(&Action::Redo(3)), MOVED);
    fact_verification();
    let corrected = document.correct_recorded(4, 2, E2, &SKIP);
    let stage = fact_verification().correction_restored_support_rejection;
    assert!(
        refused(&corrected, "SourceDiverged"),
        "{corrected:?}; {stage:?}"
    );
    assert!(
        stage
            .as_deref()
            .is_some_and(|stage| stage.starts_with("stage=canonical-cancellation")),
        "{stage:?}"
    );
    document.release();
}

/// N3: the undo revives `e2`, a write outside the document then sets it, and
/// the redo refuses on the diverged head; so does the reversal.
#[test]
fn a_write_outside_after_reviving_the_retired_entity_refuses() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(&host, &RETIRED);
    assert_eq!(document.act(&Action::Undo), MOVED);
    assert_eq!(
        document.act(&Action::OutOfBand([Stage::Skip, Stage::Set(2)])),
        Outcome::OutOfBand
    );
    assert_eq!(document.act(&Action::Redo(2)), Outcome::HeadDiverged);
    let corrected = document.correct_recorded(3, 2, E2, &SKIP);
    assert!(refused(&corrected, "HeadDiverged"), "{corrected:?}");
    document.release();
}

/// Seed 0x7b02205d's shape, with no navigation: `e2` retired, created
/// again, the re-creation corrected, then the retirement corrected. The
/// model refuses (`e2` is live at step 3, in `Since(2)`); the platform's
/// canonical cancellation covers the re-creation with its performed
/// correction pair and records.
#[test]
#[ignore = "held on public #456: a performed correction pair cancels a re-creation the model counts as live on the line"]
fn a_retirement_reversed_after_its_recreation_was_corrected_follows_the_model() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = recorded(
        &host,
        &[RETIRED[0], RETIRED[1], [Stage::Skip, Stage::Set(1)]],
    );
    assert!(matches!(
        document.correct_recorded(4, 3, E2, &SKIP),
        Corrected::Recorded(4)
    ));
    let corrected = document.correct_recorded(5, 2, E2, &SKIP);
    assert!(
        matches!(corrected, Corrected::Refused(_)),
        "the model refuses SourceRefused: {corrected:?}"
    );
    document.release();
}
