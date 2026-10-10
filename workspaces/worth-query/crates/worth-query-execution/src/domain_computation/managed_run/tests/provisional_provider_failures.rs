use std::sync::Arc;

use super::provisional_attempt::{final_program, state};
use super::provisional_attempt_fixture::*;
use crate::domain_computation::{
    WorthQueryProviderSessionRecoveryPosture, WorthQueryProvisionalDenialKind,
    WorthQueryProvisionalEffectAction,
};

#[test]
fn provider_overlay_must_match_every_and_only_lowered_staged_change() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        for outcome in [
            ProvisionalStageFixtureOutcome::OmitStagedFact,
            ProvisionalStageFixtureOutcome::AddUndeclaredStagedFact,
        ] {
            let state = state();
            state.lock().unwrap().provisional_stage_outcome = outcome;
            let (mut running, graph) = provisional_run(Arc::clone(&state), resource_request);
            let (staged, fresh) = staged_with_fresh_read_set(execution, &mut running, &graph);
            let program = final_program(&staged, &fresh);
            let failure = staged
                .begin_provisional_attempt(fresh, program)
                .err()
                .expect("dishonest proposed facts must not mint an attempt");
            assert_eq!(
                failure.kind(),
                WorthQueryProvisionalDenialKind::ProviderProgramMismatch
            );
            assert_eq!(
                failure.recovery_posture(),
                WorthQueryProviderSessionRecoveryPosture::Closed
            );
            let state = state.lock().unwrap();
            assert!(state.overlays.is_empty());
            assert_eq!(state.discard_calls, 1);
            assert_eq!(state.abort_calls, 1);
            drop(state);
            cleanup(running);
        }
    });
}

#[test]
fn duplicate_proposed_fact_identities_deny_before_provider_staging() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        for actions in [
            vec![
                WorthQueryProvisionalEffectAction::Replace {
                    target_identity: "base".into(),
                },
                WorthQueryProvisionalEffectAction::Replace {
                    target_identity: "base".into(),
                },
            ],
            vec![
                WorthQueryProvisionalEffectAction::Replace {
                    target_identity: "base".into(),
                },
                WorthQueryProvisionalEffectAction::Retire {
                    target_identity: "base".into(),
                },
            ],
        ] {
            let state = state();
            let (mut running, graph) = provisional_run(Arc::clone(&state), resource_request);
            let (staged, fresh) = staged_with_fresh_read_set(execution, &mut running, &graph);
            let failure = staged
                .effect_authority()
                .lower_provisional_program(&fresh, actions.into_iter().map(effect_step))
                .err()
                .expect("one proposed fact identity cannot be staged twice");
            assert_eq!(
                failure.kind(),
                WorthQueryProvisionalDenialKind::ProposedFactIdentityAlreadyDefined
            );
            assert_eq!(state.lock().unwrap().stage_calls, 0);
            staged.abort();
            cleanup(running);
        }
    });
}

#[test]
fn provider_staging_panic_is_typed_and_aborts_the_live_session() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let state = state();
        state.lock().unwrap().provisional_stage_outcome = ProvisionalStageFixtureOutcome::Panic;
        let (mut running, graph) = provisional_run(Arc::clone(&state), resource_request);
        let (staged, fresh) = staged_with_fresh_read_set(execution, &mut running, &graph);
        let program = final_program(&staged, &fresh);
        let failure = staged
            .begin_provisional_attempt(fresh, program)
            .err()
            .expect("provider panic must become a typed provisional failure");
        assert_eq!(
            failure.kind(),
            WorthQueryProvisionalDenialKind::ProviderPanicked
        );
        assert_eq!(
            failure.recovery_posture(),
            WorthQueryProviderSessionRecoveryPosture::Closed
        );
        assert_eq!(state.lock().unwrap().abort_calls, 1);
        cleanup(running);
    });
}

#[test]
fn provider_discard_panic_is_typed_and_the_guard_retries_cleanup() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let state = state();
        state.lock().unwrap().discard_panics_remaining = 1;
        let (mut running, graph) = provisional_run(Arc::clone(&state), resource_request);
        let (staged, fresh) = staged_with_fresh_read_set(execution, &mut running, &graph);
        let program = final_program(&staged, &fresh);
        let outcome = staged
            .begin_provisional_attempt(fresh, program)
            .unwrap()
            .discard();
        assert_eq!(
            outcome.overlay_failure().unwrap().kind(),
            WorthQueryProvisionalDenialKind::ProviderPanicked
        );
        assert_eq!(
            outcome.recovery_posture(),
            WorthQueryProviderSessionRecoveryPosture::RecoveryRequired
        );
        let state_guard = state.lock().unwrap();
        assert_eq!(state_guard.discard_calls, 2);
        assert!(state_guard.overlays.is_empty());
        assert_eq!(state_guard.abort_calls, 1);
        drop(state_guard);
        cleanup(running);
    });
}

#[test]
fn rejected_overlay_discard_panic_is_typed_and_retried_by_the_guard() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let state = state();
        {
            let mut state_guard = state.lock().unwrap();
            state_guard.provisional_stage_outcome = ProvisionalStageFixtureOutcome::OmitStagedFact;
            state_guard.discard_panics_remaining = 1;
        }
        let (mut running, graph) = provisional_run(Arc::clone(&state), resource_request);
        let (staged, fresh) = staged_with_fresh_read_set(execution, &mut running, &graph);
        let program = final_program(&staged, &fresh);
        let failure = staged
            .begin_provisional_attempt(fresh, program)
            .err()
            .expect("rejected overlay cleanup panic must remain typed");
        assert_eq!(
            failure.kind(),
            WorthQueryProvisionalDenialKind::DiscardFailed
        );
        assert_eq!(
            failure.recovery_posture(),
            WorthQueryProviderSessionRecoveryPosture::RecoveryRequired
        );
        let state_guard = state.lock().unwrap();
        assert_eq!(state_guard.discard_calls, 2);
        assert!(state_guard.overlays.is_empty());
        assert_eq!(state_guard.abort_calls, 1);
        drop(state_guard);
        cleanup(running);
    });
}
