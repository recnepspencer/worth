//! Spec-owner ruling (u): reversal facts follow navigation (plan §10.3 and
//! §11.2.2). A recorded correction posts an aftermath-causality fact, "step
//! c reversed source s on entity e". A move retires the facts of the steps it
//! leaves and restores, at their original IDs, the facts of the steps it
//! enters, so a fact is live exactly where its step is on the selected line
//! (`FactsFollowLine`). Without it an undone deletion still refuses
//! `SourceAlreadyReversed` on every other branch (plan review r1).
use super::driver::System as _;
use super::reference::{Action, Fact, Intent, Outcome, Stage, Stages, E1};
use super::world::Corrected;
use super::*;

const SKIP: Stages = [Stage::Skip; 2];

fn facts(document: &mut world::Document<'_>) -> Vec<Fact> {
    document.observe(4).reversals.into_iter().collect()
}

fn live(document: &mut world::Document<'_>) -> Option<[bool; 2]> {
    document.observe(4).branch.map(|branch| branch.live)
}

/// create e1 (1) -> delete it by correction (2) -> undo -> edit e1 on a
/// sibling branch (3) -> delete it again by correction (4) records. Each
/// checkout then sees exactly its own line's facts, and the original
/// deletion's branch still refuses a third deletion `SourceAlreadyReversed`.
#[test]
fn a_deletion_undone_on_one_branch_can_be_deleted_again_on_another() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = world::Document::attach(&host);
    let create = Action::Envelope { key: 1, intent: Intent::Edit([Stage::Set(1), Stage::Skip]), stop: None };
    assert_eq!(document.act(&create), Outcome::Recorded { node: 1 });
    let corrected = document.correct_recorded(2, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(2)), "first deletion: {corrected:?}");
    let first = Fact { step: 2, source: 1, entity: E1 };
    assert_eq!(facts(&mut document), vec![first]);
    assert_eq!(live(&mut document), Some([false, false]));

    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    assert_eq!(facts(&mut document), vec![], "the undo retires the deletion's fact");
    assert_eq!(live(&mut document), Some([true, false]));

    let sibling = Action::Envelope { key: 3, intent: Intent::Edit([Stage::Set(2), Stage::Skip]), stop: None };
    assert_eq!(document.act(&sibling), Outcome::Recorded { node: 3 });
    let corrected = document.correct_recorded(4, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(4)), "deletion on the sibling branch: {corrected:?}");
    let second = Fact { step: 4, source: 1, entity: E1 };
    assert_eq!(facts(&mut document), vec![second]);

    assert_eq!(document.act(&Action::Checkout(2)), Outcome::Moved);
    assert_eq!(facts(&mut document), vec![first], "the first deletion's fact is restored at its ID");
    assert_eq!(live(&mut document), Some([false, false]));
    let corrected = document.correct_recorded(5, 1, E1, &SKIP);
    assert!(
        matches!(&corrected, Corrected::Refused(text) if text.contains("SourceAlreadyReversed")),
        "the original deletion's branch: {corrected:?}"
    );

    assert_eq!(document.act(&Action::Checkout(4)), Outcome::Moved);
    assert_eq!(facts(&mut document), vec![second]);
    assert_eq!(document.act(&Action::Checkout(3)), Outcome::Moved);
    assert_eq!(facts(&mut document), vec![]);
    assert_eq!(live(&mut document), Some([true, false]));
    document.release();
}

/// Redo crosses the deletion forward: its fact comes back, and so does the
/// refusal of a second deletion of the same source.
#[test]
fn a_redo_restores_the_fact_and_its_refusal() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = world::Document::attach(&host);
    let create = Action::Envelope { key: 1, intent: Intent::Edit([Stage::Set(1), Stage::Skip]), stop: None };
    assert_eq!(document.act(&create), Outcome::Recorded { node: 1 });
    let corrected = document.correct_recorded(2, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(2)), "{corrected:?}");
    assert_eq!(document.act(&Action::Undo), Outcome::Moved);
    assert_eq!(facts(&mut document), vec![]);
    assert_eq!(document.act(&Action::Redo(2)), Outcome::Moved);
    assert_eq!(facts(&mut document), vec![Fact { step: 2, source: 1, entity: E1 }]);
    let corrected = document.correct_recorded(3, 1, E1, &SKIP);
    assert!(
        matches!(&corrected, Corrected::Refused(text) if text.contains("SourceAlreadyReversed")),
        "{corrected:?}"
    );
    document.release();
}

/// A correction step is not itself a correction source (ruling 2026-11-25,
/// option 1; reapply is out of Phase 2's scope, spec-owner ruling Q2): a
/// corrective binding declares no correction, so correcting the deletion
/// refuses `SourceNotCorrectable` before any effect, as the model's
/// `MeaningDerivable` (`CorrectionSteps`) and the hand trace
/// `hand-correction-step-not-a-source` say.
#[test]
fn a_correction_step_is_not_a_correction_source() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = world::Document::attach(&host);
    let create = Action::Envelope { key: 1, intent: Intent::Edit([Stage::Set(1), Stage::Skip]), stop: None };
    assert_eq!(document.act(&create), Outcome::Recorded { node: 1 });
    let corrected = document.correct_recorded(2, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(2)), "{corrected:?}");
    let corrected = document.correct_recorded(3, 2, E1, &SKIP);
    assert!(
        matches!(&corrected, Corrected::Refused(text) if text == "Source(SourceNotCorrectable)"),
        "correcting the correction: {corrected:?}"
    );
    assert_eq!(live(&mut document), Some([false, false]));
    assert_eq!(facts(&mut document), vec![Fact { step: 2, source: 1, entity: E1 }]);
    document.release();
}

/// The canonical read accepts a later revision of a fact's canonical fields
/// only when that revision's commit is a navigation that rematerialized the
/// fact (its canonical envelope names it among the restored companions). An
/// ordinary commit that rewrites a fact still refuses
/// `InconsistentCanonicalRelationships`.
#[test]
fn a_fact_rewritten_outside_navigation_still_refuses_its_canonical_read() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = world::Document::attach(&host);
    let create = Action::Envelope { key: 1, intent: Intent::Edit([Stage::Set(1), Stage::Skip]), stop: None };
    assert_eq!(document.act(&create), Outcome::Recorded { node: 1 });
    let corrected = document.correct_recorded(2, 1, E1, &SKIP);
    assert!(matches!(corrected, Corrected::Recorded(2)), "{corrected:?}");
    let read = document.rewrite_posted_fact(2);
    assert_eq!(read, Err("InconsistentCanonicalRelationships".to_owned()));
    document.release();
}

/// Refinement (p), narrowed to the declared contract (ruling 2026-11-25,
/// confirmed by the spec owner): a cascade co-retires only what the same
/// source binding (stage) created (`co_retire_created`). Reversing `e1`'s
/// creation while `e2`, born by another stage of the same step, still lives
/// is a refused partial retirement: the corrective `despawn-e1` stage refuses
/// `DomainDenied` (the model's `StageRefused`) before any effect. A
/// cross-stage cascade (a coupled retire stage) waits on public #369; the
/// hand traces `hand-co-retire-across-stages-refused` and
/// `hand-rejected-co-retire-across-stages` pin the model.
#[test]
fn reversing_a_creation_while_its_sibling_stage_lives_refuses() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = world::Document::attach(&host);
    let both = Action::Envelope { key: 1, intent: Intent::Edit([Stage::Set(1), Stage::Set(1)]), stop: None };
    assert_eq!(document.act(&both), Outcome::Recorded { node: 1 });
    let corrected = document.correct_recorded(2, 1, E1, &SKIP);
    assert!(
        matches!(&corrected, Corrected::Refused(text)
            if text.contains("DomainDenied") && text.contains("owned-outputs.dst.despawn-e1.v1")),
        "{corrected:?}"
    );
    assert_eq!(live(&mut document), Some([true, true]));
    assert_eq!(facts(&mut document), vec![]);
    document.release();
}
