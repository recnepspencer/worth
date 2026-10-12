use super::*;

pub(super) fn foreign_runtime() -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
        "application-authorization",
    )
}

pub(super) fn inconsistent_authorization() -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
        "application-authorization",
    )
}

pub(super) fn stale_authorization() -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
        "application-authorization",
    )
}

pub(super) fn stale_principal() -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::StalePrincipal,
        "application-authorization",
    )
}

pub(super) fn validate_retained_currentness(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    authorization: &WorthQueryRetainedAuthorizationDecisionFacts,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    bridge: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    authorization
        .validate_currentness_in(execution, runtime, snapshot, bridge)
        .map_err(|kind| {
            WorthQueryOperationAuthorizationDenial::new(kind, "application-authorization")
        })
}

pub(super) fn validate_observed_currentness(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    authorization: &WorthQueryObservedCommitBasis,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    bridge: &worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    if !authorization.principal_remains_current_in(execution, runtime, snapshot) {
        return Err(stale_principal());
    }
    authorization
        .decisions_remain_current_in(execution, runtime, snapshot, bridge)
        .then_some(())
        .ok_or_else(stale_authorization)
}
