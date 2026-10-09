use std::sync::Arc;

use super::invariant_execution::{execute_invariant, locator, requirement, state_with_outcome};
use super::provisional_attempt_fixture::InvariantFixtureOutcome;
use crate::domain_computation::WorthQueryInvariantExecutionDenialKind;
use worth_query_installation::facade::WorthQueryInvariantEnforcement;

#[test]
fn provider_must_load_exactly_the_admitted_locator_closure() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        for (outcome, locators) in [
            (
                InvariantFixtureOutcome::OmitLoad,
                vec![locator("base"), locator("old")],
            ),
            (InvariantFixtureOutcome::OverreadLoad, vec![locator("base")]),
        ] {
            let state = state_with_outcome(outcome);
            let failure = execute_invariant(
                execution,
                Arc::clone(&state),
                vec![requirement(
                    "closed-loop",
                    WorthQueryInvariantEnforcement::Blocking,
                    4,
                )],
                "closed-loop",
                locators,
            )
            .err()
            .expect("omitted or out-of-closure state must deny");
            assert_eq!(
                failure.kind(),
                WorthQueryInvariantExecutionDenialKind::StateLoadClosureMismatch
            );
            assert_eq!(state.lock().unwrap().invariant_load_calls, 1);
            assert_eq!(state.lock().unwrap().invariant_execution_calls, 0);
        }
    });
}
