use worth_runtime_bridge::facade::{BridgeConditionalDenial, BridgeConditionalDenialKind};

use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};

pub(super) fn bridge_denial(
    producer_identity: &str,
    failure: BridgeConditionalDenial,
) -> WorthQueryOutputDemandDenial {
    let kind = bridge_denial_kind(&failure);
    WorthQueryOutputDemandDenial::new(
        kind,
        format!(
            "{producer_identity}: conditional {:?}: {}",
            failure.kind(),
            failure.detail()
        ),
    )
}

pub(super) fn bridge_denial_kind(
    failure: &BridgeConditionalDenial,
) -> WorthQueryOutputDemandDenialKind {
    use BridgeConditionalDenialKind as Kind;
    match failure.kind() {
        Kind::ExecutionDenied(cause) => WorthQueryOutputDemandDenialKind::ExecutionRequest(
            super::bridge_execution_denial::query_cause(cause),
        ),
        Kind::ConditionalRetentionCapacity
        | Kind::ConditionalEvaluationBusy
        | Kind::ConditionalEvaluationUnwindPending
        | Kind::ConditionalEvaluationAdmissionCapacity => {
            WorthQueryOutputDemandDenialKind::SchedulingDeferred
        }
        Kind::ConditionalRetentionClosed
        | Kind::ConditionalRetentionQuarantined
        | Kind::ForeignSignalGraph
        | Kind::CorrespondenceAdmission
        | Kind::EmptyCorrespondenceSet
        | Kind::MixedSignalNodes
        | Kind::DeclarationLocationMismatch
        | Kind::DeclarationCorrespondenceMismatch
        | Kind::SignalNodeAlreadyBound
        | Kind::MissingConditionProvider
        | Kind::ExtraConditionProvider
        | Kind::MissingDependencyComparator
        | Kind::ExtraDependencyComparator
        | Kind::MissingOutputComparator
        | Kind::ExtraOutputComparator
        | Kind::MissingReuseComparator
        | Kind::ExtraReuseComparator
        | Kind::MissingTriggerProvider
        | Kind::ExtraTriggerProvider
        | Kind::MissingWakeProvider
        | Kind::ExtraWakeProvider
        | Kind::MissingComputeProvider
        | Kind::ExtraComputeProvider
        | Kind::UnsupportedMaintenancePosture
        | Kind::UnsupportedArtifactPosture
        | Kind::SignalContractInstallation
        | Kind::StaleLowering
        | Kind::OperationAuthorityMismatch
        | Kind::GraphAuthorityMismatch
        | Kind::SignalContractMismatch
        | Kind::DependencyOrdinalMismatch
        | Kind::SnapshotMismatch
        | Kind::SnapshotAdmission
        | Kind::MissingSourceObservation
        | Kind::SourcePostureMismatch
        | Kind::AttemptMismatch
        | Kind::ManagedWakeMismatch
        | Kind::ManagedClockQuarantined
        | Kind::SignalExecution
        | Kind::ConditionalTransitionChainIncomplete
        | Kind::ConditionalTransitionChainMismatch
        | Kind::ConditionalPredecessorNotExecuted
        | Kind::ConditionalEvaluationPoisoned => {
            WorthQueryOutputDemandDenialKind::SchedulingRejected
        }
    }
}
