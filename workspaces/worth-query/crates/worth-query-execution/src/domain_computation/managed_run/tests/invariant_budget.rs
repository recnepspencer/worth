use std::num::NonZeroU32;

use super::invariant_execution::{execute_invariant, locator, state};
use crate::domain_computation::{
    WorthQueryInvariantExecutionDenialKind, WorthQueryInvariantExecutionFailurePosture,
};
use worth_query_installation::facade::{
    WorthQueryInstalledInvariantExecutionRequirement, WorthQueryInvariantEnforcement,
};

#[test]
fn state_load_and_validator_share_one_installed_work_budget() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let state = state();
        let requirement = WorthQueryInstalledInvariantExecutionRequirement::new(
            "closed-loop",
            "topology",
            NonZeroU32::new(1).unwrap(),
            WorthQueryInvariantEnforcement::Blocking,
            "managed-graph",
            ["region"],
            4,
            1,
        )
        .unwrap();
        let failure = execute_invariant(
            execution,
            std::sync::Arc::clone(&state),
            vec![requirement],
            "closed-loop",
            [locator("base")],
        )
        .err()
        .expect("one load unit plus one execution unit must exceed a one-unit total budget");
        assert_eq!(
            failure.kind(),
            WorthQueryInvariantExecutionDenialKind::ExecutionBudgetExceeded
        );
        assert_eq!(
            failure.posture(),
            WorthQueryInvariantExecutionFailurePosture::Exhausted
        );
        let state = state.lock().unwrap();
        assert_eq!(state.invariant_load_calls, 1);
        assert_eq!(state.invariant_execution_calls, 1);
    });
}
