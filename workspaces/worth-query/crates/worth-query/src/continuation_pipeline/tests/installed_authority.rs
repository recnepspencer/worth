use crate::continuation_pipeline::{
    WorthQueryContinuationExecutionOutcome, WorthQueryPreparedContinuationOutcome,
};
use crate::domain_installation::{
    WorthQueryInstalledDomainExecutionDrift, WorthQueryInstalledDomainExecutionDriftKind,
    WorthQueryInstalledDomainExecutionNextAction,
};
use crate::recovery_boundary::{
    worth_query_recovery_brief_from_continuation_execution_checked, WorthQueryRecoveryAction,
    WorthQueryRecoveryStopKind,
};

use super::support::{
    admitted_handle, admitted_workspace, continuation_handle_in, runtime_route_request,
    target_request, RuntimeFamily,
};

#[test]
fn prepared_continuation_rejects_foreign_runtime_before_later_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let owner = admitted_handle(request_execution, "main");
    let foreign = admitted_handle(request_execution, "main");
    let prepared = prepared_runtime_continuation(&owner);

    let WorthQueryContinuationExecutionOutcome::InstalledAuthorityDrift(drift) =
        foreign.execute_prepared_continuation(prepared)
    else {
        panic!("foreign runtime must stop at installed authority")
    };

    assert_drift(
        &drift,
        WorthQueryInstalledDomainExecutionDriftKind::ForeignRuntime,
        WorthQueryInstalledDomainExecutionNextAction::UseOwningRuntime,
    );
    assert_ne!(
        drift.retained_authority().authority().authority_identity(),
        drift
            .current_authority()
            .expect("foreign execution has a current authority")
            .authority()
            .authority_identity()
    );
}

#[test]
fn prepared_continuation_rejects_stale_generation_on_current_handle() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, retained_handle) = admitted_workspace(request_execution, "main");
    let prepared = prepared_runtime_continuation(&retained_handle);
    let retained_generation = retained_handle
        .installed_authority()
        .authority()
        .installation_generation();
    workspace
        .replace_domain_installation_with_successor_generation()
        .unwrap();
    let current_handle = continuation_handle_in(&workspace, "main");
    assert!(
        current_handle
            .installed_authority()
            .authority()
            .installation_generation()
            > retained_generation
    );

    let WorthQueryContinuationExecutionOutcome::InstalledAuthorityDrift(drift) =
        current_handle.execute_prepared_continuation(prepared)
    else {
        panic!("stale prepared authority must not execute on the successor generation")
    };

    assert_drift(
        &drift,
        WorthQueryInstalledDomainExecutionDriftKind::StaleInstallation,
        WorthQueryInstalledDomainExecutionNextAction::RebindCurrentInstallation,
    );
    assert_ne!(
        drift
            .retained_authority()
            .authority()
            .installation_generation(),
        drift
            .current_authority()
            .expect("successor generation is observable")
            .authority()
            .installation_generation()
    );
}

#[test]
fn stale_handle_cannot_prepare_after_generation_turnover() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, stale_handle) = admitted_workspace(request_execution, "main");
    let request = target_request::<RuntimeFamily>(&stale_handle, "face-a", runtime_route_request());
    workspace
        .replace_domain_installation_with_successor_generation()
        .unwrap();

    let WorthQueryPreparedContinuationOutcome::InstalledAuthorityDrift(drift) =
        stale_handle.prepare_continuation_from_target(request)
    else {
        panic!("stale installed authority must stop before continuation preparation")
    };

    assert_drift(
        &drift,
        WorthQueryInstalledDomainExecutionDriftKind::StaleInstallation,
        WorthQueryInstalledDomainExecutionNextAction::RebindCurrentInstallation,
    );
    assert!(drift.current_authority().is_none());
}

#[test]
fn recovery_retains_foreign_runtime_authority_drift_without_promoting_it() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let owner = admitted_handle(request_execution, "main");
    let foreign = admitted_handle(request_execution, "main");
    let prepared = prepared_runtime_continuation(&owner);

    let brief = worth_query_recovery_brief_from_continuation_execution_checked(
        foreign.execute_prepared_continuation_checked(prepared),
    )
    .expect("foreign installed authority must produce a recovery brief");
    let drift = brief
        .explanation()
        .installed_domain_execution_drift()
        .expect("recovery must retain installed authority drift evidence");

    assert_eq!(
        brief.stop_kind(),
        WorthQueryRecoveryStopKind::RebindRequired
    );
    assert_eq!(
        brief.recommended_action(),
        WorthQueryRecoveryAction::RebindContext
    );
    assert_drift(
        drift,
        WorthQueryInstalledDomainExecutionDriftKind::ForeignRuntime,
        WorthQueryInstalledDomainExecutionNextAction::UseOwningRuntime,
    );
    assert_ne!(
        drift.retained_authority().authority().authority_identity(),
        drift
            .current_authority()
            .expect("foreign recovery has a current authority")
            .authority()
            .authority_identity()
    );
}

fn prepared_runtime_continuation(
    handle: &crate::application::WorthQueryInstalledDomainDeclarationContext<
        super::support::ContinuationDomain,
        super::support::ContinuationWorld,
    >,
) -> crate::continuation_pipeline::WorthQueryPreparedContinuation<
    super::support::ContinuationDomain,
    super::support::Input<RuntimeFamily>,
> {
    match handle.prepare_continuation_from_target(target_request::<RuntimeFamily>(
        handle,
        "face-a",
        runtime_route_request(),
    )) {
        WorthQueryPreparedContinuationOutcome::Prepared(prepared) => prepared,
        _ => panic!("fixture must prepare a runtime continuation"),
    }
}

fn assert_drift(
    drift: &WorthQueryInstalledDomainExecutionDrift,
    expected_kind: WorthQueryInstalledDomainExecutionDriftKind,
    expected_action: WorthQueryInstalledDomainExecutionNextAction,
) {
    assert_eq!(drift.kind(), expected_kind);
    assert_eq!(drift.next_action(), expected_action);
    assert_eq!(drift.counters().planning_attempts(), 0);
    assert_eq!(drift.counters().lower_runtime_attempts(), 0);
    assert_eq!(drift.counters().execution_attempts(), 0);
}
