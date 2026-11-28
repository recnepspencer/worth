//! Conditional posture projection for application commit deferrals.

use worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence;

use super::WorthQueryTemporalReentryCounts;
use crate::domain_computation::primary_graph::conditional_operation::signal_decision_reentry::{
    WorthQueryOperationBackpressureCause, WorthQueryRetainedConditionalDecision,
    WorthQueryRetainedConditionalWake,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDeferred, WorthQueryApplicationCommitDeferredKind as Kind,
};

pub(super) fn apply_provider_commit_deferred(
    wake: &mut WorthQueryRetainedConditionalWake,
    evidence: BridgeConditionalDecisionEvidence,
    deferred: WorthQueryApplicationCommitDeferred,
    counts: &mut WorthQueryTemporalReentryCounts,
) {
    let kind = deferred.kind();
    if matches!(
        kind,
        Kind::RelationalDeferred(
            worth_relational::facade::mvcc::RelationalPublicationDeferred::RetentionBackpressure
        )
    ) {
        counts.retention_capacity_backpressure = true;
    }
    backpressured(wake, evidence, kind);
}

fn backpressured(
    wake: &mut WorthQueryRetainedConditionalWake,
    evidence: BridgeConditionalDecisionEvidence,
    kind: Kind,
) {
    wake.decision = WorthQueryRetainedConditionalDecision::OperationBackpressured(
        evidence,
        WorthQueryOperationBackpressureCause::ProviderCommit(kind),
    );
}
