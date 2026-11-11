//! Real redispatch distinguishes a determinate rejection from a lost response.
use super::*;
use worth_query_execution::facade::domain_computation::{ExternalEffectNoEffectCause, WorthQueryExternalDispatchPostureKind};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::commit_on_world;

struct ScriptedTransport(
    std::sync::Mutex<std::collections::VecDeque<WorthQueryExternalTransportOutcome>>,
);
impl WorthQueryExternalEffectTransport for ScriptedTransport {
    fn dispatch(
        &self,
        _: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0
            .lock()
            .unwrap()
            .pop_front()
            .expect("the exact contact script")
    }
}

#[test]
fn rejected_redispatch_retains_its_no_effect_cause_while_lost_response_stays_unresolved() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let transport = Arc::new(ScriptedTransport(std::sync::Mutex::new(
        [
            WorthQueryExternalTransportOutcome::Rejected,
            WorthQueryExternalTransportOutcome::Rejected,
            WorthQueryExternalTransportOutcome::LostResponse,
            WorthQueryExternalTransportOutcome::Completed,
        ]
        .into_iter()
        .collect(),
    )));
    world
        .application
        .install_external_effect_transport(transport.clone())
        .unwrap();
    let receipt = commit_on_world(&world, 217, "open", "rejected-original");
    assert_eq!(
        receipt.external_dispatch().unwrap().posture().kind(),
        WorthQueryExternalDispatchPostureKind::ProvenNoEffect
    );
    let (world, _, handle, admission) =
        real_handle_on(world, receipt, "rejected-original", live_scope());
    let authority = authority(&world, &handle, &admission);
    let rejected = performed_redispatch(&world, &handle, &authority, &admission);
    let denial = safe_retry_recovery_handle(handle, &authority, rejected).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryRecoveryHandleDenialKind::ProvenNoEffect(
            ExternalEffectNoEffectCause::PayloadRejected
        )
    );
    let (_, handle) = denial.into_parts();
    let handle = handle.expect("no-effect denial retains the live handle");
    let lost = performed_redispatch(&world, &handle, &authority, &admission);
    let denial = safe_retry_recovery_handle(handle, &authority, lost).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryRecoveryHandleDenialKind::UnresolvedExternalPosture
    );
    let (_, handle) = denial.into_parts();
    let handle = handle.expect("uncertainty also retains its original handle");
    let completed = performed_redispatch(&world, &handle, &authority, &admission);
    safe_retry_recovery_handle(handle, &authority, completed)
        .expect("only actual completion consumes as completed");
    assert!(transport.0.lock().unwrap().is_empty());
}
