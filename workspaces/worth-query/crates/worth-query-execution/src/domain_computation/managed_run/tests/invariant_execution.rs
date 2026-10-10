mod installed_budgets;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use super::invariant_execution_fixture as invariant_fixture;
use super::provisional_attempt_fixture::*;
use crate::domain_computation::{
    WorthQueryInvariantExecutionDenialKind, WorthQueryInvariantExecutionFailure,
    WorthQueryInvariantExecutionFailurePosture, WorthQueryInvariantReceipt,
    WorthQueryInvariantStateLocator,
};
use worth_query_installation::facade::{
    WorthQueryInstalledInvariantExecutionRequirement, WorthQueryInvariantEnforcement,
};

#[test]
fn blocking_and_advisory_receipts_retain_their_installed_posture() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let blocking = requirement("closed-loop", WorthQueryInvariantEnforcement::Blocking, 4);
        let advisory = requirement(
            "recommended-density",
            WorthQueryInvariantEnforcement::Advisory,
            4,
        );

        let passed = execute_invariant(
            execution,
            state(),
            vec![blocking],
            "closed-loop",
            [locator("base")],
            resource_request,
        )
        .unwrap();
        let WorthQueryInvariantReceipt::Passed(passed) = passed else {
            panic!("valid blocking invariant must pass")
        };
        assert_eq!(passed.invariant_family(), "topology");
        assert_eq!(passed.invariant_version(), 1);
        assert_eq!(passed.counters().loaded_facts(), 1);
        assert_eq!(passed.counters().execution_work_units(), 1);
        assert!(!passed.physical_execution_evidence().is_empty());

        let advisory = execute_invariant(
            execution,
            state(),
            vec![advisory],
            "recommended-density",
            [locator("base")],
            resource_request,
        )
        .unwrap();
        let WorthQueryInvariantReceipt::Advisory(advisory) = advisory else {
            panic!("valid advisory invariant must retain advisory posture")
        };
        assert_eq!(advisory.invariant_slot(), "recommended-density");
    });
}

#[test]
fn progression_requires_the_exact_complete_installed_invariant_set() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let requirements = vec![
            requirement("closed-loop", WorthQueryInvariantEnforcement::Blocking, 4),
            requirement("manifold", WorthQueryInvariantEnforcement::Blocking, 4),
        ];
        let incomplete = invariant_fixture::admit_progression(
            execution,
            state(),
            requirements.clone(),
            ["closed-loop"],
            resource_request,
        )
        .expect_err("one passed receipt cannot progress a two-slot contract");
        assert_eq!(
            incomplete.kind(),
            WorthQueryInvariantExecutionDenialKind::VerdictPostureMismatch
        );
        let complete = invariant_fixture::admit_progression(
            execution,
            state(),
            requirements,
            ["closed-loop", "manifold"],
            resource_request,
        )
        .expect("the exact passed slot set should progress");
        assert_eq!(complete.receipt_identities().len(), 2);
    });
}

#[test]
fn each_selected_blocking_invariant_observes_corruption_and_cannot_progress() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        for slot in ["closed-loop", "manifold"] {
            let corrupt = state();
            corrupt
                .lock()
                .unwrap()
                .authoritative
                .insert("untouched".to_owned(), "corrupt".to_owned());
            let receipt = execute_invariant(
                execution,
                corrupt,
                vec![
                    requirement("closed-loop", WorthQueryInvariantEnforcement::Blocking, 4),
                    requirement("manifold", WorthQueryInvariantEnforcement::Blocking, 4),
                ],
                slot,
                [locator("untouched")],
                resource_request,
            )
            .unwrap();
            let WorthQueryInvariantReceipt::Violated(receipt) = receipt else {
                panic!("corrupted blocking invariant {slot} must be violated")
            };
            assert_eq!(receipt.invariant_slot(), slot);
        }
    });
}

#[test]
fn regional_and_full_validation_agree_or_return_indeterminate() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let valid = invariant_fixture::execute_invariant(
            execution,
            state(),
            vec![requirement(
                "closed-loop",
                WorthQueryInvariantEnforcement::Blocking,
                4,
            )],
            "closed-loop",
            [locator("base")],
            resource_request,
        );
        assert!(!valid.full_invalid);
        let valid = valid.result.unwrap();
        assert!(matches!(valid, WorthQueryInvariantReceipt::Passed(_)));

        let corrupt = state();
        corrupt
            .lock()
            .unwrap()
            .authoritative
            .insert("untouched".to_owned(), "corrupt".to_owned());
        let matching = invariant_fixture::execute_invariant(
            execution,
            corrupt,
            vec![requirement(
                "closed-loop",
                WorthQueryInvariantEnforcement::Blocking,
                4,
            )],
            "closed-loop",
            [locator("untouched")],
            resource_request,
        );
        assert!(matching.full_invalid);
        let matching = matching.result.unwrap();
        assert!(matches!(matching, WorthQueryInvariantReceipt::Violated(_)));

        let replaced_corruption = state();
        replaced_corruption
            .lock()
            .unwrap()
            .authoritative
            .insert("base".to_owned(), "corrupt".to_owned());
        let proposed_state_wins = invariant_fixture::execute_invariant(
            execution,
            replaced_corruption,
            vec![requirement(
                "closed-loop",
                WorthQueryInvariantEnforcement::Blocking,
                4,
            )],
            "closed-loop",
            [locator("base")],
            resource_request,
        );
        assert!(!proposed_state_wins.full_invalid);
        let proposed_state_wins = proposed_state_wins.result.unwrap();
        assert!(matches!(
            proposed_state_wins,
            WorthQueryInvariantReceipt::Passed(_)
        ));

        let incomplete = state_with_outcome(InvariantFixtureOutcome::Indeterminate);
        incomplete
            .lock()
            .unwrap()
            .authoritative
            .insert("untouched".to_owned(), "corrupt".to_owned());
        let mismatch = invariant_fixture::execute_invariant(
            execution,
            incomplete,
            vec![requirement(
                "closed-loop",
                WorthQueryInvariantEnforcement::Blocking,
                4,
            )],
            "closed-loop",
            [locator("base")],
            resource_request,
        );
        assert!(mismatch.full_invalid);
        let mismatch = mismatch.result.unwrap();
        assert!(matches!(
            mismatch,
            WorthQueryInvariantReceipt::Indeterminate(_)
        ));
    });
}

#[test]
fn empty_or_undeclared_state_loads_never_reach_validator_execution() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let empty = state_with_outcome(InvariantFixtureOutcome::EmptyLoad);
        let failure = execute_invariant(
            execution,
            Arc::clone(&empty),
            vec![requirement(
                "closed-loop",
                WorthQueryInvariantEnforcement::Blocking,
                4,
            )],
            "closed-loop",
            [locator("base")],
            resource_request,
        )
        .err()
        .expect("empty provider load must deny");
        assert_eq!(
            failure.kind(),
            WorthQueryInvariantExecutionDenialKind::EmptyStateLoad
        );
        assert_eq!(empty.lock().unwrap().invariant_execution_calls, 0);

        let undeclared = state();
        let failure = execute_invariant(
            execution,
            Arc::clone(&undeclared),
            vec![requirement(
                "closed-loop",
                WorthQueryInvariantEnforcement::Blocking,
                4,
            )],
            "closed-loop",
            [WorthQueryInvariantStateLocator::new("foreign", "base").unwrap()],
            resource_request,
        )
        .err()
        .expect("undeclared load family must deny");
        assert_eq!(
            failure.kind(),
            WorthQueryInvariantExecutionDenialKind::UndeclaredStateLoadFamily
        );
        assert_eq!(undeclared.lock().unwrap().invariant_load_calls, 0);
    });
}

pub(super) fn execute_invariant(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    state: Arc<Mutex<ProvisionalProviderState>>,
    requirements: Vec<WorthQueryInstalledInvariantExecutionRequirement>,
    slot: &str,
    locators: impl IntoIterator<Item = WorthQueryInvariantStateLocator>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WorthQueryInvariantReceipt, WorthQueryInvariantExecutionFailure> {
    invariant_fixture::execute_invariant(
        execution,
        state,
        requirements,
        slot,
        locators,
        resource_request,
    )
    .result
}

pub(super) fn requirement(
    slot: &str,
    enforcement: WorthQueryInvariantEnforcement,
    max_state_facts: usize,
) -> WorthQueryInstalledInvariantExecutionRequirement {
    WorthQueryInstalledInvariantExecutionRequirement::new(
        slot,
        "topology",
        NonZeroU32::new(1).unwrap(),
        enforcement,
        "managed-graph",
        ["region"],
        max_state_facts,
        8,
    )
    .unwrap()
}

pub(super) fn locator(identity: &str) -> WorthQueryInvariantStateLocator {
    WorthQueryInvariantStateLocator::new("region", identity).unwrap()
}

pub(super) fn state_with_outcome(
    invariant_outcome: InvariantFixtureOutcome,
) -> Arc<Mutex<ProvisionalProviderState>> {
    let state = state();
    state.lock().unwrap().invariant_outcome = invariant_outcome;
    state
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

#[path = "invariant_execution/invariant_executor_role_must_match_the_bound_provider_plan.rs"]
mod invariant_executor_role_must_match_the_bound_provider_plan;
