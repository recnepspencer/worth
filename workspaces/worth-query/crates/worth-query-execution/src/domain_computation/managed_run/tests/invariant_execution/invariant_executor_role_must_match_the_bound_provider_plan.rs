//! Invariant executor role must match the bound provider plan.

use super::*;

#[test]
fn invariant_executor_role_must_match_the_bound_provider_plan() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let state = state();
        let requirement = WorthQueryInstalledInvariantExecutionRequirement::new(
            "closed-loop",
            "topology",
            NonZeroU32::new(1).unwrap(),
            WorthQueryInvariantEnforcement::Blocking,
            "different-graph-role",
            ["region"],
            4,
            8,
        )
        .unwrap();
        let failure = execute_invariant(
            execution,
            Arc::clone(&state),
            vec![requirement],
            "closed-loop",
            [locator("base")],
            resource_request,
        )
        .err()
        .expect("foreign executor role must deny before state load");
        assert_eq!(
            failure.kind(),
            WorthQueryInvariantExecutionDenialKind::InvariantNotInstalled
        );
        assert_eq!(state.lock().unwrap().invariant_load_calls, 0);
    });
}
