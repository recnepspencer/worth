use crate::domain_computation::primary_graph::{
    WorthQueryApplicationSettlementDeferred, WorthQueryApplicationSettlementRecoveryError,
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub(super) enum WorthQuerySettlementReentry {
    AlreadyCommitted,
    ExecutionDenied(
        WorthQueryApplicationSettlementDeferred,
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
    ),
    Indeterminate(super::super::WorthQueryConditionalReentryFailure),
    Deferred(WorthQueryApplicationSettlementDeferred),
    SnapshotBackpressured(WorthQueryApplicationSettlementDeferred, usize),
    RetentionBackpressured(WorthQueryApplicationSettlementDeferred),
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
}

pub(super) fn repair<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    deferred: WorthQueryApplicationSettlementDeferred,
) -> WorthQuerySettlementReentry
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    match runtime.recover_deferred_application_settlement(&deferred) {
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
