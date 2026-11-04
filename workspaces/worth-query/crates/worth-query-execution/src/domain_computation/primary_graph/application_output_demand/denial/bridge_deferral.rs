//! Whether a bridge conditional refusal is a temporary deferral: the retry meaning of
//! an output demand stop, read from the native cause alone.
use worth_runtime_bridge::facade::BridgeConditionalDenialKind;

// Recovery grouping never replaces the cause stored in the demand kind.
pub(in crate::domain_computation::primary_graph) fn is_temporarily_deferred(kind: &BridgeConditionalDenialKind) -> bool {
    use BridgeConditionalDenialKind as Kind;
    match kind {
        Kind::ConditionalRetentionCapacity => true,
        Kind::SignalExecution(cause) => signal_service_is_deferred(cause),
        Kind::ConditionalTargetReferenceExhausted
        | Kind::ExecutionDenied(_)
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
        | Kind::ConditionalTransitionMissing => false,
    }
}

fn signal_service_is_deferred(cause: &worth_runtime_bridge::facade::BridgeSignalDenial) -> bool {
    use worth_runtime_bridge::facade::BridgeSignalDenial as Cause;
    use worth_signal::facade::branch::{
        SignalConditionalEvaluationReadmissionDenial as Readmit,
        SignalConditionalServiceExecutionDenial as Execute,
    };
    match cause {
        Cause::ConditionalExecution(native) => match native {
            Execute::AdmissionCapacityExhausted | Execute::SlotBusy | Execute::UnconsumedUnwind => {
                true
            }
            Execute::OwnerUnavailable(_)
            | Execute::OwnerAdmission(_)
            | Execute::StaleBasisAdmission
            | Execute::DefinitionReadmissionRequired
            | Execute::DefinitionMismatch
            | Execute::NestedOperationScopeMismatch
            | Execute::MissingSourceEvidence
            | Execute::UnexpectedSourceEvidence
            | Execute::SourceAuthorityMismatch
            | Execute::EvaluationIdentityExhausted
            | Execute::AdmissionUnavailable
            | Execute::SlotPoisoned
            | Execute::SlotAdmission(_)
            | Execute::ObservationAdmission(_) => false,
        },
        Cause::EvaluationReadmission(native) => match native {
            Readmit::AdmissionCapacityExhausted | Readmit::SlotBusy | Readmit::UnconsumedUnwind => {
                true
            }
            Readmit::OwnerUnavailable(_)
            | Readmit::OwnerAdmission(_)
            | Readmit::StaleBasisAdmission
            | Readmit::DefinitionReadmissionRequired
            | Readmit::DefinitionMismatch
            | Readmit::TransitionChainIncomplete
            | Readmit::TransitionChainMismatch
            | Readmit::PredecessorNotExecuted
            | Readmit::EvaluationIdentityExhausted
            | Readmit::AdmissionUnavailable
            | Readmit::SlotPoisoned
            | Readmit::SlotAdmission(_) => false,
        },
        Cause::ContractInstallation(_)
        | Cause::InstallationExtension(_)
        | Cause::InstallationChange(_)
        | Cause::UnsealedOwnerServices
        | Cause::MissingConditionalPort
        | Cause::DefinitionPublicationAlreadyOwned
        | Cause::Error(_)
        | Cause::BranchObservation(_)
        | Cause::BranchReadmission(_)
        | Cause::OwnerServiceIssuance(_)
        | Cause::ConditionalServiceIssuance(_) => false,
    }
}
