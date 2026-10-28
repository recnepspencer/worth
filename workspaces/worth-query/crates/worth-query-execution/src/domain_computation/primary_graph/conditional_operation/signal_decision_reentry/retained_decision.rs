use worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence;

pub(in crate::domain_computation::primary_graph::conditional_operation) enum WorthQueryRetainedConditionalDecision
{
    OperationCleanupRequired(
        BridgeConditionalDecisionEvidence,
        Box<super::super::retained_managed_cleanup::RetainedManagedCleanup>,
    ),
    OperationProductStale(
        BridgeConditionalDecisionEvidence,
        crate::domain_computation::WorthQueryProductStaleApplication,
    ),
    OperationNoEffect(
        BridgeConditionalDecisionEvidence,
        crate::domain_computation::primary_graph::WorthQueryApplicationNoEffectCause,
    ),
    Eligible(BridgeConditionalDecisionEvidence),
    Suppressed(BridgeConditionalDecisionEvidence),
    Deferred(BridgeConditionalDecisionEvidence),
    OperationRetryable(
        BridgeConditionalDecisionEvidence,
        super::super::WorthQueryConditionalReentryFailure,
    ),
    OperationCommitRetryable(
        BridgeConditionalDecisionEvidence,
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind,
    ),
    OperationExecutionControlRetryable(
        BridgeConditionalDecisionEvidence,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    ),
    OperationBackpressured(
        BridgeConditionalDecisionEvidence,
        WorthQueryOperationBackpressureCause,
    ),
    OperationControlStopped(
        BridgeConditionalDecisionEvidence,
        super::super::application_operation_reentry::WorthQueryTemporalControlStop,
    ),
    OperationTerminalFailure(
        BridgeConditionalDecisionEvidence,
        super::super::application_operation_reentry::WorthQueryTemporalTerminalFailure,
    ),
    OperationProductUnpublished(
        BridgeConditionalDecisionEvidence,
        crate::domain_computation::WorthQueryProductUnpublishedRecovery,
    ),
    OperationIndeterminate(
        BridgeConditionalDecisionEvidence,
        super::super::WorthQueryConditionalReentryFailure,
    ),
    OperationCommitted(BridgeConditionalDecisionEvidence),
    OperationAlreadyCommitted(BridgeConditionalDecisionEvidence),
    Failed(worth_runtime_bridge::facade::BridgeConditionalDenial),
    InterruptedDuringReentry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::conditional_operation) enum WorthQueryOperationBackpressureCause
{
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    RetentionCapacityExhausted,
    ProviderCommit(
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDeferredKind,
    ),
}
