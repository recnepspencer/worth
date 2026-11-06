use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::cleanup;
use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::effect_step;
use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::provisional_run;
use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::staged_with_fresh_read_set;
use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::ProvisionalProviderState;

use std::sync::Arc;
use std::sync::Mutex;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionRecoveryPosture;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalEffectAction;
#[derive(Clone, Copy)]
pub enum Stage {
    Attempt,
    Proposed,
    Inspection,
}
pub fn discard_at_stage(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
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
pub fn abandon_at_stage(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
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
pub fn final_program(
    staged: &worth_query_execution::facade::domain_computation::WorthQuerySessionBoundReadsAndEffects<'_>,
    fresh: &worth_query_execution::facade::domain_computation::WorthQueryFreshDecisionReadSet,
) -> worth_query_execution::facade::domain_computation::WorthQueryLoweredProvisionalEffectProgram {
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
pub fn state() -> Arc<Mutex<ProvisionalProviderState>> {
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
