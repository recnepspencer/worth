use super::*;

#[test]
fn installed_budgets_never_degrade_into_success() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let state_load = state();
        let failure = execute_invariant(
            execution,
            state_load,
            vec![requirement(
                "closed-loop",
                WorthQueryInvariantEnforcement::Blocking,
                1,
            )],
            "closed-loop",
            [locator("base"), locator("old")],
            resource_request,
        )
        .err()
        .expect("oversized state load must deny");
        assert_eq!(
            failure.kind(),
            WorthQueryInvariantExecutionDenialKind::StateLoadBudgetExceeded
        );
        assert_eq!(
            failure.posture(),
            WorthQueryInvariantExecutionFailurePosture::Exhausted
        );

        for (outcome, expected) in [
            (
                InvariantFixtureOutcome::Exhausted,
                "execution budget exhaustion",
            ),
            (
                InvariantFixtureOutcome::Indeterminate,
                "incomplete execution evidence",
            ),
        ] {
            let receipt = execute_invariant(
                execution,
                state_with_outcome(outcome),
                vec![requirement(
                    "closed-loop",
                    WorthQueryInvariantEnforcement::Blocking,
                    4,
                )],
                "closed-loop",
                [locator("base")],
                resource_request,
            )
            .unwrap();
            match outcome {
                InvariantFixtureOutcome::Exhausted => {
                    assert!(
                        matches!(receipt, WorthQueryInvariantReceipt::Exhausted(_)),
                        "{expected}"
                    )
                }
                InvariantFixtureOutcome::Indeterminate => assert!(
                    matches!(receipt, WorthQueryInvariantReceipt::Indeterminate(_)),
                    "{expected}"
                ),
                _ => unreachable!(),
            }
        }
    });
}
