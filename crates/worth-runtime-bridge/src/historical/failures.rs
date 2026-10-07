use std::sync::Arc;

use crate::facade::{
    BridgeDeliveryError, BridgeDeliveryErrorKind, BridgeHistoricalEvaluationCounters,
    BridgeHistoricalEvaluationFailureClass, BridgeHistoricalEvaluationFailureRecord,
    BridgeHistoricalMaterializationPath, HistoricalEvaluationDeclaration, RuntimeBridge,
};
use crate::snapshot::{
    BridgeTruthViewKind, PlannedTruthViewPacket,
    TruthViewPolicyRejectionKind as PolicyRejectionKind,
};

pub(crate) fn historical_materialization_path_for(
    planned: &PlannedTruthViewPacket,
) -> BridgeHistoricalMaterializationPath {
    match planned.declaration().selector().view_kind() {
        BridgeTruthViewKind::CommittedSnapshot | BridgeTruthViewKind::BranchSnapshot => {
            BridgeHistoricalMaterializationPath::DirectSnapshotRead
        }
        BridgeTruthViewKind::HistoricalCommit | BridgeTruthViewKind::BranchCommit => {
            BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot
        }
        BridgeTruthViewKind::BranchHead => {
            BridgeHistoricalMaterializationPath::BranchHeadEnvelopeSnapshot
        }
    }
}

pub(crate) fn historical_materialization_path_for_declaration(
    declaration: &HistoricalEvaluationDeclaration,
) -> BridgeHistoricalMaterializationPath {
    match declaration.selector().view_kind() {
        BridgeTruthViewKind::BranchHead => {
            BridgeHistoricalMaterializationPath::BranchHeadEnvelopeSnapshot
        }
        BridgeTruthViewKind::HistoricalCommit | BridgeTruthViewKind::BranchCommit => {
            BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot
        }
        BridgeTruthViewKind::CommittedSnapshot | BridgeTruthViewKind::BranchSnapshot => {
            BridgeHistoricalMaterializationPath::DirectSnapshotRead
        }
    }
}

pub(crate) fn historical_failure_class_for_policy_rejection(
    kind: PolicyRejectionKind,
) -> BridgeHistoricalEvaluationFailureClass {
    match kind {
        PolicyRejectionKind::UnsupportedTruthViewSelector => {
            BridgeHistoricalEvaluationFailureClass::UnsupportedTruthViewSelector
        }
        PolicyRejectionKind::UnavailableTruthView => {
            BridgeHistoricalEvaluationFailureClass::TruthViewUnavailable
        }
        PolicyRejectionKind::SourceCapabilityMismatch
        | PolicyRejectionKind::UnresolvedPolicyConflict
        | PolicyRejectionKind::ReplayNotPermitted => {
            BridgeHistoricalEvaluationFailureClass::UnresolvedTruthViewPolicyConflict
        }
        PolicyRejectionKind::BranchMismatch => {
            BridgeHistoricalEvaluationFailureClass::RejectedBranchMismatch
        }
    }
}

pub(crate) fn historical_failure_class_for_delivery_error(
    error: &BridgeDeliveryError,
) -> BridgeHistoricalEvaluationFailureClass {
    match error.kind() {
        BridgeDeliveryErrorKind::ExecutionDenied(denial) => {
            BridgeHistoricalEvaluationFailureClass::ExecutionDenied(denial)
        }
        BridgeDeliveryErrorKind::SourceContractMismatch => {
            BridgeHistoricalEvaluationFailureClass::UnresolvedTruthViewPolicyConflict
        }
        BridgeDeliveryErrorKind::StructuralContractMismatch
        | BridgeDeliveryErrorKind::StructuralPlanRejected => {
            BridgeHistoricalEvaluationFailureClass::RejectedHistoricalResolutionFailure
        }
        BridgeDeliveryErrorKind::HistoricalTruthViewUnavailable
        | BridgeDeliveryErrorKind::SnapshotAcquisitionFailure => {
            BridgeHistoricalEvaluationFailureClass::TruthViewUnavailable
        }
        BridgeDeliveryErrorKind::HistoricalBranchMismatch => {
            BridgeHistoricalEvaluationFailureClass::RejectedBranchMismatch
        }
        BridgeDeliveryErrorKind::SnapshotIdentityMismatch => {
            BridgeHistoricalEvaluationFailureClass::RejectedSnapshotMismatch
        }
        BridgeDeliveryErrorKind::BulkDeliveryRejected
        | BridgeDeliveryErrorKind::HistoricalPolicyRejected
        | BridgeDeliveryErrorKind::HistoricalCommitMismatch
        | BridgeDeliveryErrorKind::HistoricalSelectorMissingCommit
        | BridgeDeliveryErrorKind::InvalidWideningAdmission
        | BridgeDeliveryErrorKind::SnapshotReadFailure
        | BridgeDeliveryErrorKind::SnapshotReadContractViolation
        | BridgeDeliveryErrorKind::SignalSinkRejection => {
            BridgeHistoricalEvaluationFailureClass::RejectedHistoricalResolutionFailure
        }
    }
}

pub(crate) fn historical_failure_counters_for_policy_rejection(
    declaration: &HistoricalEvaluationDeclaration,
    kind: PolicyRejectionKind,
) -> BridgeHistoricalEvaluationCounters {
    let counters = BridgeHistoricalEvaluationCounters::from_failed_materialization(
        declaration,
        historical_materialization_path_for_declaration(declaration),
    );
    match kind {
        PolicyRejectionKind::UnavailableTruthView => counters.with_unavailable_truth_view(),
        PolicyRejectionKind::BranchMismatch => counters.with_branch_mismatch(),
        _ => counters,
    }
}

pub(crate) fn historical_failure_counters_for_delivery_error(
    declaration: &HistoricalEvaluationDeclaration,
    error: &BridgeDeliveryError,
) -> BridgeHistoricalEvaluationCounters {
    let counters = BridgeHistoricalEvaluationCounters::from_failed_materialization(
        declaration,
        historical_materialization_path_for_declaration(declaration),
    );
    match error.kind() {
        BridgeDeliveryErrorKind::SourceContractMismatch => counters,
        BridgeDeliveryErrorKind::StructuralContractMismatch
        | BridgeDeliveryErrorKind::StructuralPlanRejected => counters,
        BridgeDeliveryErrorKind::HistoricalTruthViewUnavailable
        | BridgeDeliveryErrorKind::SnapshotAcquisitionFailure => {
            counters.with_unavailable_truth_view()
        }
        BridgeDeliveryErrorKind::HistoricalBranchMismatch => counters.with_branch_mismatch(),
        BridgeDeliveryErrorKind::SnapshotIdentityMismatch => counters.with_snapshot_mismatch(),
        _ => counters,
    }
}

impl RuntimeBridge {
    pub(crate) fn record_historical_evaluation_failure(
        &self,
        declaration: &HistoricalEvaluationDeclaration,
        failure_class: BridgeHistoricalEvaluationFailureClass,
        detail: impl Into<Arc<str>>,
        counters: BridgeHistoricalEvaluationCounters,
    ) {
        self.diagnostic_sink.record_historical_evaluation_failure(
            BridgeHistoricalEvaluationFailureRecord::new(
                declaration.declaration_identity().clone(),
                declaration.selector().selector_identity().clone(),
                declaration.selector().branch_identity().clone(),
                declaration.selector().commit_identity().cloned(),
                declaration.selector().snapshot_identity().cloned(),
                failure_class,
                detail,
                counters,
            ),
        );
    }
}

#[cfg(test)]
mod execution_cause_tests {
    use super::*;
    #[test]
    fn historical_failure_preserves_cancellation_and_deadline() {
        use crate::facade::BridgeExecutionDenial as Own;
        for (stop, expected) in [
            (worth_execution::MapKernelStop::Cancelled, Own::Cancelled),
            (
                worth_execution::MapKernelStop::DeadlineElapsed,
                Own::DeadlineElapsed,
            ),
        ] {
            let error = BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::ExecutionDenied(stop.into()),
                "resource refusal",
            );
            assert_eq!(
                historical_failure_class_for_delivery_error(&error),
                BridgeHistoricalEvaluationFailureClass::ExecutionDenied(expected)
            );
        }
    }
}
