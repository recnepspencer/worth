//! Public #348 under a real correction identity (plan §8a.7, S2's first
//! court). An authored correction that retires an owner, with another stage
//! on a live member in the same envelope, must commit: CAD Phase 3's
//! `delete_feature` puts its corrective retirement and a coupled edit stage in
//! one envelope. Today's correction route runs it outside the document, on
//! the commit pipeline the history route (`document.correction_envelope`)
//! shares; the history route repeats these courts once it exists.
use super::fixture::coupled::Coupled;
use super::reference::{Action, Intent, Outcome, Stage, Stages, E1, E2};
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
        let outcome = document.act(&action);
        assert_eq!(outcome, Outcome::Recorded { node: step as u32 + 1 }, "creating {}", fixture::NAMES[entity]);
    }
    document
}

/// The controls: an authored creation reversal alone commits, for the leaf
/// `e2` and for `e1` once nothing depends on it.
#[test]
fn a_corrective_retirement_alone_commits_outside_the_document() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    for (name, creations, source, entity) in [("undo-create e2", &[E1, E2][..], 2, E2), ("undo-create e1", &[E1][..], 1, E1)] {
        let mut document = created(&host, creations);
        let outcome = document.correct_coupled(9, source, entity, &SKIP);
        assert!(matches!(outcome, Coupled::Committed), "{name}: {outcome:?}");
        document.release();
    }
}

/// Released by public #348 (PR #374): a corrective retirement as the prefix,
/// with a set of the other live member after it, commits in one envelope.
#[test]
fn a_corrective_retirement_before_a_coupled_set_commits_outside_the_document() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = created(&host, &[E1, E2]);
    let outcome = document.correct_coupled(9, 2, E2, &[Stage::Set(2), Stage::Skip]);
    document.release();
    assert!(matches!(outcome, Coupled::Committed), "[undo-create e2, set e1]: {outcome:?}");
}

/// A retirement in a later stage than the corrective prefix is postponed on
/// public #369 (per-stage retirement events): the envelope refuses typed,
/// before any effect.
#[test]
fn a_retirement_after_the_corrective_prefix_refuses_typed_on_369() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = created(&host, &[E1, E2]);
    let outcome = document.correct_coupled(9, 2, E2, &[Stage::Retire, Stage::Skip]);
    document.release();
    assert!(
        matches!(&outcome, Coupled::Refused(denial) if denial.contains("LaterStageRetirementPostponed { issue: 369")),
        "[undo-create e2, retire e1]: {outcome:?}"
    );
}

/// Released by public #368 (PR #380): correction evidence is stage-granular,
/// so reversing `e2`'s creation, the second stage of `[create e1, create
/// e2]`, commits outside the document.
#[test]
fn a_later_stage_of_a_multi_stage_step_can_be_corrected() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = world::Document::attach(&host);
    let both = Action::Envelope { key: 1, intent: Intent::Edit([Stage::Set(1), Stage::Set(1)]), stop: None };
    assert_eq!(document.act(&both), Outcome::Recorded { node: 1 });
    let outcome = document.correct_coupled(9, 1, E2, &SKIP);
    document.release();
    assert!(matches!(outcome, Coupled::Committed), "#368: {outcome:?}");
}
