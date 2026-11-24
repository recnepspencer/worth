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

/// The two shapes a plain envelope refuses on #348, with the retirement now
/// an authored correction: `MissingCoupledOwner` beside a set of `e1`, and
/// `OwnedPublicationRequiresCascade` beside a retirement of `e1`.
#[test]
#[ignore = "held on public #348: a corrective retirement beside another live member's stage refuses MissingCoupledOwner / OwnedPublicationRequiresCascade (correction identity)"]
fn a_corrective_retirement_beside_a_coupled_stage_commits_outside_the_document() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut refused = Vec::new();
    for (name, coupled) in [
        ("[undo-create e2, set e1]", [Stage::Set(2), Stage::Skip]),
        ("[undo-create e2, retire e1]", [Stage::Retire, Stage::Skip]),
    ] {
        let mut document = created(&host, &[E1, E2]);
        match document.correct_coupled(9, 2, E2, &coupled) {
            Coupled::Committed => {}
            other => refused.push(format!("{name}: {other:?}")),
        }
        document.release();
    }
    assert!(refused.is_empty(), "#348 under a correction identity refuses:\n{}", refused.join("\n"));
}
