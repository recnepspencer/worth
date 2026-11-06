use worth_runtime_bridge::facade::{BridgeConditionalDenial, BridgeConditionalDenialKind};

use {
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial,
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind,
};

pub(in crate::domain_computation::primary_graph) fn bridge_denial(
    producer_identity: &str,
    failure: BridgeConditionalDenial,
) -> WorthQueryOutputDemandDenial {
    let kind = bridge_denial_kind(failure.kind());
    WorthQueryOutputDemandDenial::new(
        kind,
        format!(
            "{producer_identity}: conditional {:?}: {}",
            failure.kind(),
            failure.detail()
        ),
    )
}

pub fn bridge_denial_kind(kind: BridgeConditionalDenialKind) -> WorthQueryOutputDemandDenialKind {
    use BridgeConditionalDenialKind as Kind;
    match kind {
        Kind::ExecutionDenied(cause) => match super::bridge_execution::query_cause(cause) {
            Ok(cause) => WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
            Err(interruption) => {
                WorthQueryOutputDemandDenialKind::of_execution_interruption(interruption)
            }
        },
        native @ (Kind::ConditionalRetentionCapacity
        | Kind::ConditionalTargetReferenceExhausted
        | Kind::ConditionalRetentionClosed
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
        | Kind::SnapshotRead(_)
        | Kind::Delivery(_)
        | Kind::MissingSourceObservation
        | Kind::SourcePostureMismatch
        | Kind::AttemptMismatch
        | Kind::ManagedWakeMismatch
        | Kind::ManagedClockQuarantined
        | Kind::ManagedClockAdmission(_)
        | Kind::SignalExecution(_)
        | Kind::ConditionalTransitionMissing) => {
            WorthQueryOutputDemandDenialKind::BridgeConditional(Box::new(native))
        }
    }
}
