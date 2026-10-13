use crate::domain_computation::primary_graph::{
    WorthQueryApplicationSettlementDeferred, WorthQueryApplicationSettlementRecoveryError,
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub(super) enum WorthQuerySettlementReentry {
    AlreadyCommitted,
    ExecutionDenied(
        WorthQueryApplicationSettlementDeferred,
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    Indeterminate(super::super::WorthQueryConditionalReentryFailure),
    Deferred(WorthQueryApplicationSettlementDeferred),
    SnapshotBackpressured(WorthQueryApplicationSettlementDeferred, usize),
    RetentionBackpressured(WorthQueryApplicationSettlementDeferred),
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
}

pub(super) fn repair<Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    deferred: WorthQueryApplicationSettlementDeferred,
) -> WorthQuerySettlementReentry
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    match runtime.recover_deferred_application_settlement(execution, &deferred) {
        Err(WorthQueryApplicationSettlementRecoveryError::ExecutionDenied(kind)) => {
            WorthQuerySettlementReentry::ExecutionDenied(deferred, kind)
        }
        Ok(_) => WorthQuerySettlementReentry::AlreadyCommitted,
        Err(WorthQueryApplicationSettlementRecoveryError::IdempotencyAbsent) => {
            WorthQuerySettlementReentry::Indeterminate(
                super::super::WorthQueryConditionalReentryFailure::SettlementIdempotencyAbsent,
            )
        }
        Err(WorthQueryApplicationSettlementRecoveryError::IdempotencyDrift) => {
            WorthQuerySettlementReentry::Indeterminate(
                super::super::WorthQueryConditionalReentryFailure::SettlementIdempotencyDrift,
            )
        }
        Err(
            WorthQueryApplicationSettlementRecoveryError::Durability(_)
            | WorthQueryApplicationSettlementRecoveryError::Publication(_),
        ) => WorthQuerySettlementReentry::Deferred(deferred),
        Err(WorthQueryApplicationSettlementRecoveryError::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        }) => {
            WorthQuerySettlementReentry::SnapshotBackpressured(deferred, maximum_active_snapshots)
        }
        Err(WorthQueryApplicationSettlementRecoveryError::RetentionCapacityExhausted) => {
            WorthQuerySettlementReentry::RetentionBackpressured(deferred)
        }
        Err(WorthQueryApplicationSettlementRecoveryError::RetentionIdentityExhausted) => {
            WorthQuerySettlementReentry::RetentionIdentityExhausted
        }
        Err(WorthQueryApplicationSettlementRecoveryError::SnapshotIdentityExhausted) => {
            WorthQuerySettlementReentry::SnapshotIdentityExhausted
        }
    }
}
