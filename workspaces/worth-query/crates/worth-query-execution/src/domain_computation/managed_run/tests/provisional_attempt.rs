use std::sync::{Arc, Mutex};

use super::provisional_attempt_fixture::*;
use crate::domain_computation::{
    WorthQueryProposedFactOrigin, WorthQueryProviderSessionRecoveryPosture,
    WorthQueryProvisionalDenialKind, WorthQueryProvisionalEffectAction,
    WorthQueryProvisionalProposalBasisParts,
};

#[test]
fn proposed_state_exposes_typed_overlay_origins_without_mutating_authoritative_truth() {
    let state = state();
    let (mut running, graph) = provisional_run(Arc::clone(&state));
    let (staged, fresh) = staged_with_fresh_read_set(&mut running, &graph);
    let program = staged
        .effect_authority()
        .lower_provisional_program(
            &fresh,
            [
                effect_step(WorthQueryProvisionalEffectAction::Create {
                    symbolic_identity: "draft".into(),
                }),
                effect_step(WorthQueryProvisionalEffectAction::Replace {
                    target_identity: "base".into(),
                }),
                effect_step(WorthQueryProvisionalEffectAction::Retire {
                    target_identity: "old".into(),
                }),
                effect_step(WorthQueryProvisionalEffectAction::DeriveView {
                    view_identity: "summary".into(),
                })
                .with_symbolic_dependencies(["draft"])
                .unwrap(),
            ],
        )
        .unwrap();
    let proposed = staged
        .begin_provisional_attempt(fresh, program)
        .unwrap()
        .materialize_proposed_state();
    let origins = proposed
        .facts()
        .iter()
        .map(|fact| fact.origin())
        .collect::<Vec<_>>();
    for expected in [
        WorthQueryProposedFactOrigin::AuthoritativeBase,
        WorthQueryProposedFactOrigin::StagedReplacement,
        WorthQueryProposedFactOrigin::StagedCreation,
        WorthQueryProposedFactOrigin::StagedRetirement,
        WorthQueryProposedFactOrigin::DerivedProvisionalView,
    ] {
        assert!(origins.contains(&expected));
    }
    assert_eq!(
        state.lock().unwrap().authoritative.get("base").unwrap(),
        "base-value"
    );
    assert_eq!(state.lock().unwrap().overlays.len(), 1);
    let discarded = proposed.discard();
    assert_eq!(
        discarded.recovery_posture(),
        WorthQueryProviderSessionRecoveryPosture::Closed
    );
    assert!(state.lock().unwrap().overlays.is_empty());
    cleanup(running);
}

#[test]
fn every_provisional_stage_has_a_consuming_discard_that_clears_overlay_and_session() {
    discard_at_stage(Stage::Attempt);
    discard_at_stage(Stage::Proposed);
    discard_at_stage(Stage::Inspection);
}

#[test]
fn abandoning_each_provisional_state_discards_overlay_before_aborting_session() {
    abandon_at_stage(Stage::Attempt);
    abandon_at_stage(Stage::Proposed);
    abandon_at_stage(Stage::Inspection);
}

#[test]
fn equivalent_direct_and_revised_programs_have_the_same_semantic_post_state() {
    let direct_state = state();
    let (mut direct_run, direct_graph) = provisional_run(Arc::clone(&direct_state));
    let (direct_staged, direct_fresh) = staged_with_fresh_read_set(&mut direct_run, &direct_graph);
    let direct_program = final_program(&direct_staged, &direct_fresh);
    let direct = direct_staged
        .begin_provisional_attempt(direct_fresh, direct_program)
        .unwrap()
        .materialize_proposed_state();
    let direct_facts = direct.facts().to_vec();
    direct.discard();
    cleanup(direct_run);

    let revised_state = state();
    let (mut revised_run, revised_graph) = provisional_run(Arc::clone(&revised_state));
    let (revised_staged, revised_fresh) =
        staged_with_fresh_read_set(&mut revised_run, &revised_graph);
    let initial = revised_staged
        .effect_authority()
        .lower_provisional_program(
            &revised_fresh,
            [effect_step(WorthQueryProvisionalEffectAction::Create {
                symbolic_identity: "temporary".into(),
            })],
        )
        .unwrap();
    let final_program = final_program(&revised_staged, &revised_fresh);
    let revised = revised_staged
        .begin_provisional_attempt(revised_fresh, initial)
        .unwrap()
        .materialize_proposed_state()
        .inspect()
        .revise(final_program)
        .unwrap()
        .materialize_proposed_state();
    assert_eq!(revised.generation(), 2);
    assert_eq!(revised.facts(), direct_facts);
    assert_eq!(revised_state.lock().unwrap().discard_calls, 1);
    revised.discard();
    cleanup(revised_run);
}

#[test]
fn proposal_dimensions_and_symbol_order_are_checked_before_provider_staging() {
    let state = state();
    let (mut running, graph) = provisional_run(Arc::clone(&state));
    let (staged, fresh) = staged_with_fresh_read_set(&mut running, &graph);
    let basis = proposal_parts(1);
    let effect_authority = staged.effect_authority();
    let first = effect_authority
        .admit_proposal_basis(&fresh, basis.clone())
        .unwrap();
    for changed in changed_proposal_dimensions(&basis) {
        match effect_authority.admit_proposal_basis(&fresh, changed) {
            Ok(changed) => assert_ne!(first, changed),
            Err(failure) => assert_eq!(
                failure.kind(),
                WorthQueryProvisionalDenialKind::ProposalBasisMismatch
            ),
        }
    }

    let invalid_symbol = effect_step(WorthQueryProvisionalEffectAction::DeriveView {
        view_identity: "view".into(),
    })
    .with_symbolic_dependencies(["not-created"])
    .unwrap();
    let failure = staged
        .effect_authority()
        .lower_provisional_program(&fresh, [invalid_symbol])
        .err()
        .expect("unknown symbolic reference must deny");
    assert_eq!(
        failure.kind(),
        WorthQueryProvisionalDenialKind::UnknownSymbolicReference
    );
    let undeclared_artifact = effect_step(WorthQueryProvisionalEffectAction::Replace {
        target_identity: "base".into(),
    })
    .with_artifact_dependencies(["caller-authored-artifact"])
    .unwrap();
    let failure = staged
        .effect_authority()
        .lower_provisional_program(&fresh, [undeclared_artifact])
        .err()
        .expect("undeclared artifact dependency must deny");
    assert_eq!(
        failure.kind(),
        WorthQueryProvisionalDenialKind::UndeclaredArtifactDependency
    );

    let wrong_generation = staged
        .effect_authority()
        .admit_proposal_basis(&fresh, proposal_parts(2))
        .unwrap();
    let program = staged
        .effect_authority()
        .lower_provisional_program(
            &fresh,
            [effect_step(WorthQueryProvisionalEffectAction::Replace {
                target_identity: "base".into(),
            })
            .with_proposal_basis(wrong_generation)],
        )
        .unwrap();
    let failure = match staged.begin_provisional_attempt(fresh, program) {
        Ok(_) => panic!("wrong proposal generation must deny before overlay staging"),
        Err(failure) => failure,
    };
    assert_eq!(
        failure.kind(),
        WorthQueryProvisionalDenialKind::ProposalBasisMismatch
    );
    assert_eq!(state.lock().unwrap().stage_calls, 0);
    cleanup(running);
}

#[test]
fn proposal_from_a_peer_session_cannot_be_repaired_by_equal_rendered_basis() {
    let first_state = state();
    let (mut first_run, first_graph) = provisional_run(Arc::clone(&first_state));
    let (first_staged, first_fresh) = staged_with_fresh_read_set(&mut first_run, &first_graph);
    let proposal = first_staged
        .effect_authority()
        .admit_proposal_basis(&first_fresh, proposal_parts(1))
        .expect("the originating session admits its proposal");

    let second_state = state();
    let (mut second_run, second_graph) = provisional_run(Arc::clone(&second_state));
    let (second_staged, second_fresh) = staged_with_fresh_read_set(&mut second_run, &second_graph);
    assert_eq!(
        first_staged.plan().basis_identity(),
        second_staged.plan().basis_identity()
    );
    let failure = second_staged
        .effect_authority()
        .lower_provisional_program(
            &second_fresh,
            [effect_step(WorthQueryProvisionalEffectAction::Replace {
                target_identity: "base".into(),
            })
            .with_proposal_basis(proposal)],
        )
        .err()
        .expect("a peer terminal binding cannot adopt the proposal");
    assert_eq!(
        failure.kind(),
        WorthQueryProvisionalDenialKind::ProposalBasisMismatch
    );
    let _ = first_staged.abort();
    let _ = second_staged.abort();
    cleanup(first_run);
    cleanup(second_run);
}

#[derive(Clone, Copy)]
enum Stage {
    Attempt,
    Proposed,
    Inspection,
}

fn discard_at_stage(stage: Stage) {
    let state = state();
    let (mut running, graph) = provisional_run(Arc::clone(&state));
    let (staged, fresh) = staged_with_fresh_read_set(&mut running, &graph);
    let program = final_program(&staged, &fresh);
    let attempt = staged.begin_provisional_attempt(fresh, program).unwrap();
    let outcome = match stage {
        Stage::Attempt => attempt.discard(),
        Stage::Proposed => attempt.materialize_proposed_state().discard(),
        Stage::Inspection => attempt.materialize_proposed_state().inspect().discard(),
    };
    assert_eq!(
        outcome.recovery_posture(),
        WorthQueryProviderSessionRecoveryPosture::Closed
    );
    assert!(state.lock().unwrap().overlays.is_empty());
    assert_eq!(
        state.lock().unwrap().authoritative.get("base").unwrap(),
        "base-value"
    );
    cleanup(running);
}

fn abandon_at_stage(stage: Stage) {
    let state = state();
    let (mut running, graph) = provisional_run(Arc::clone(&state));
    let (staged, fresh) = staged_with_fresh_read_set(&mut running, &graph);
    let program = final_program(&staged, &fresh);
    let attempt = staged.begin_provisional_attempt(fresh, program).unwrap();
    match stage {
        Stage::Attempt => drop(attempt),
        Stage::Proposed => drop(attempt.materialize_proposed_state()),
        Stage::Inspection => drop(attempt.materialize_proposed_state().inspect()),
    }
    let state = state.lock().unwrap();
    assert!(state.overlays.is_empty());
    assert_eq!(state.discard_calls, 1);
    assert_eq!(state.abort_calls, 1);
    drop(state);
    cleanup(running);
}

pub(super) fn final_program(
    staged: &crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'_>,
    fresh: &crate::domain_computation::WorthQueryFreshDecisionReadSet,
) -> crate::domain_computation::WorthQueryLoweredProvisionalEffectProgram {
    staged
        .effect_authority()
        .lower_provisional_program(
            fresh,
            [
                effect_step(WorthQueryProvisionalEffectAction::Replace {
                    target_identity: "base".into(),
                }),
                effect_step(WorthQueryProvisionalEffectAction::Create {
                    symbolic_identity: "final".into(),
                }),
            ],
        )
        .unwrap()
}

pub(super) fn state() -> Arc<Mutex<ProvisionalProviderState>> {
    Arc::new(Mutex::new(ProvisionalProviderState {
        authoritative: [
            ("base".to_owned(), "base-value".to_owned()),
            ("old".to_owned(), "old-value".to_owned()),
            ("untouched".to_owned(), "untouched-value".to_owned()),
        ]
        .into_iter()
        .collect(),
        ..ProvisionalProviderState::default()
    }))
}

fn proposal_parts(target_generation: u64) -> WorthQueryProvisionalProposalBasisParts {
    WorthQueryProvisionalProposalBasisParts {
        source_occurrence: "source-1".to_owned(),
        search_occurrence: "search-1".to_owned(),
        candidate_identity: "candidate-a".to_owned(),
        transformation_evidence: "transform-1".to_owned(),
        target_generation,
        installed_policy_identity: "policy-1".to_owned(),
        correspondence_identity: "correspondence-1".to_owned(),
        identity_consequence_identity: "identity-map-1".to_owned(),
    }
}

fn changed_proposal_dimensions(
    basis: &WorthQueryProvisionalProposalBasisParts,
) -> Vec<WorthQueryProvisionalProposalBasisParts> {
    let mut changes = Vec::new();
    macro_rules! changed {
        ($field:ident, $value:expr) => {{
            let mut value = basis.clone();
            value.$field = $value;
            changes.push(value);
        }};
    }
    changed!(source_occurrence, "source-2".to_owned());
    changed!(search_occurrence, "search-2".to_owned());
    changed!(candidate_identity, "candidate-b".to_owned());
    changed!(transformation_evidence, "transform-2".to_owned());
    changed!(target_generation, 2);
    changed!(installed_policy_identity, "policy-2".to_owned());
    changed!(correspondence_identity, "correspondence-2".to_owned());
    changed!(identity_consequence_identity, "identity-map-2".to_owned());
    changes
}
