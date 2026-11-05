use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt::Stage;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt::abandon_at_stage;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt_fixture::cleanup;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt::discard_at_stage;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt_fixture::effect_step;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt::final_program;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt_fixture::provisional_run;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt_fixture::staged_with_fresh_read_set;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt::state;

use std::sync::Arc;
use std::sync::Mutex;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt_fixture::*;
use worth_query_execution::facade::domain_computation::WorthQueryProposedFactOrigin;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionRecoveryPosture;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalEffectAction;
#[test]
fn proposed_state_exposes_typed_overlay_origins_without_mutating_authoritative_truth() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
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
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
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
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
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
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
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

#[path = "proposal_validation/mod.rs"]
pub mod proposal_validation;
