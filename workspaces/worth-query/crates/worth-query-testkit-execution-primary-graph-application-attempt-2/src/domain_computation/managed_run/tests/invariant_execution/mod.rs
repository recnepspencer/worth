use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::InvariantFixtureOutcome;
use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::ProvisionalProviderState;

use crate::domain_computation::managed_run::tests::invariant_execution_fixture as invariant_fixture;
use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::*;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::Mutex;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionFailure;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionFailurePosture;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantReceipt;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantStateLocator;
use worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement;
use worth_query_installation::facade::WorthQueryInvariantEnforcement;
pub fn execute_invariant(
    execution: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    state: Arc<Mutex<ProvisionalProviderState>>,
    requirements: Vec<WorthQueryInstalledInvariantExecutionRequirement>,
    slot: &str,
    locators: impl IntoIterator<Item = WorthQueryInvariantStateLocator>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WorthQueryInvariantReceipt, WorthQueryInvariantExecutionFailure> {
    crate::domain_computation::managed_run::tests::invariant_execution_fixture::execute_invariant(
        execution,
        state,
        requirements,
        slot,
        locators,
        resource_request,
    )
    .result
}
pub fn requirement(
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
pub fn locator(identity: &str) -> WorthQueryInvariantStateLocator {
    WorthQueryInvariantStateLocator::new("region", identity).unwrap()
}
pub fn state_with_outcome(
    invariant_outcome: InvariantFixtureOutcome,
) -> Arc<Mutex<ProvisionalProviderState>> {
    let state = state();
    state.lock().unwrap().invariant_outcome = invariant_outcome;
    state
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
