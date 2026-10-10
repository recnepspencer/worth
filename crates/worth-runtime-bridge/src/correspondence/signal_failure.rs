//! Native Signal causes retained by correspondence delivery.
/// The Signal seam and its exact typed refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeCorrespondenceSignalFailure {
    /// Admission of installed scoped changes failed before graph mutation.
    ScopedChangeAdmission(worth_signal::facade::SignalInstalledScopedChangeDenial),
    /// Raw graph scoped-change execution failed.
    ScopedChangeExecution(worth_signal::facade::SignalError),
    /// The owner-issued committed-patch service refused its operation.
    CommittedPatch(worth_signal::facade::branch::SignalCommittedPatchDeliveryDenial),
}
