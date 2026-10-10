use std::sync::{Arc, Mutex};

mod proposal_validation;

use super::provisional_attempt_fixture::*;
use crate::domain_computation::{
    WorthQueryProposedFactOrigin, WorthQueryProviderSessionRecoveryPosture,
    WorthQueryProvisionalEffectAction,
};

#[test]
fn proposed_state_exposes_typed_overlay_origins_without_mutating_authoritative_truth() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let state = state();
        let (mut running, graph) = provisional_run(Arc::clone(&state), resource_request);
        let (staged, fresh) = staged_with_fresh_read_set(execution, &mut running, &graph);
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
    });
}

#[test]
fn every_provisional_stage_has_a_consuming_discard_that_clears_overlay_and_session() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        discard_at_stage(execution, Stage::Attempt, resource_request);
        discard_at_stage(execution, Stage::Proposed, resource_request);
        discard_at_stage(execution, Stage::Inspection, resource_request);
    });
}

#[test]
fn abandoning_each_provisional_state_discards_overlay_before_aborting_session() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        abandon_at_stage(execution, Stage::Attempt, resource_request);
        abandon_at_stage(execution, Stage::Proposed, resource_request);
        abandon_at_stage(execution, Stage::Inspection, resource_request);
    });
}

#[test]
fn equivalent_direct_and_revised_programs_have_the_same_semantic_post_state() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let direct_state = state();
        let (mut direct_run, direct_graph) =
            provisional_run(Arc::clone(&direct_state), resource_request);
        let (direct_staged, direct_fresh) =
            staged_with_fresh_read_set(execution, &mut direct_run, &direct_graph);
        let direct_program = final_program(&direct_staged, &direct_fresh);
        let direct = direct_staged
            .begin_provisional_attempt(direct_fresh, direct_program)
            .unwrap()
            .materialize_proposed_state();
        let direct_facts = direct.facts().to_vec();
        direct.discard();
        cleanup(direct_run);

        let revised_state = state();
        let (mut revised_run, revised_graph) =
            provisional_run(Arc::clone(&revised_state), resource_request);
        let (revised_staged, revised_fresh) =
            staged_with_fresh_read_set(execution, &mut revised_run, &revised_graph);
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
    });
}

#[derive(Clone, Copy)]
enum Stage {
    Attempt,
    Proposed,
    Inspection,
}

fn discard_at_stage(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    stage: Stage,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) {
    let state = state();
    let (mut running, graph) = provisional_run(Arc::clone(&state), resource_request);
    let (staged, fresh) = staged_with_fresh_read_set(execution, &mut running, &graph);
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

fn abandon_at_stage(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    stage: Stage,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) {
    let state = state();
    let (mut running, graph) = provisional_run(Arc::clone(&state), resource_request);
    let (staged, fresh) = staged_with_fresh_read_set(execution, &mut running, &graph);
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
