//! Owned Signal causes retained by Bridge without diagnostic-string conversion.
use worth_signal::facade::branch::{
    SignalBranchBasisObservationDenial, SignalConditionalEvaluationReadmissionDenial,
    SignalConditionalServiceExecutionDenial, SignalConditionalServiceIssuanceDenial,
    SignalOwnerServiceIssuanceDenial,
};

/// A Signal failure or the exact missing Bridge binding that prevented contact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeSignalDenial {
    /// Signal rejected the installed conditional contract.
    ContractInstallation(worth_signal::facade::SignalConditionalContractDenial),
    /// Signal refused an installation extension or successor activation.
    InstallationExtension(
        worth_signal::facade::branch::SignalConditionalInstallationExtensionDenial,
    ),
    /// Signal refused a conditional installation lifecycle change.
    InstallationChange(worth_signal::facade::branch::SignalConditionalInstallationChangeDenial),
    /// Bridge owner services have not been sealed.
    UnsealedOwnerServices,
    /// An installed lowering has no exact conditional service port.
    MissingConditionalPort,
    /// Definition-publication ownership was already transferred to World.
    DefinitionPublicationAlreadyOwned,
    /// Signal's native evaluation, preparation or runtime-policy failure.
    Error(worth_signal::facade::SignalError),
    /// Admission of the exact Signal branch observation failed.
    BranchObservation(SignalBranchBasisObservationDenial),
    /// Exact branch-basis readmission failed at the native owner.
    BranchReadmission(worth_signal::facade::branch::SignalBranchBasisReadmissionDenial),
    /// Issuance of Signal's owner services failed.
    OwnerServiceIssuance(SignalOwnerServiceIssuanceDenial),
    /// Issuance of the conditional service failed.
    ConditionalServiceIssuance(SignalConditionalServiceIssuanceDenial),
    /// The conditional service refused execution before a completion.
    ConditionalExecution(SignalConditionalServiceExecutionDenial),
    /// The retained conditional evaluation could not enter its successor basis.
    EvaluationReadmission(SignalConditionalEvaluationReadmissionDenial),
}
