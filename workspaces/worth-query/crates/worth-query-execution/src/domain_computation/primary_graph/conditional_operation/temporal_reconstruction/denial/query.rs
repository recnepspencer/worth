//! Reconstruction query admission preserves stops, capacity, and ordinary refusal.
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use super::*;
use crate::domain_computation::primary_graph::{WorthQueryApplicationQueryAdmissionDenialKind as Query, WorthQueryOperationAuthorizationDenialKind as Authorization};
use crate::domain_computation::primary_graph::conditional_operation::application_operation_reentry::{WorthQueryTemporalReentryDenial as Reentry, WorthQueryTemporalControlStop as Control};
pub(super) fn admission(
    kind: Query,
    detail: String,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    match kind {
        Query::ExecutionRequest(kind) => read_stopped(Ok(kind)),
        Query::Cancelled => read_stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
        )),
        Query::DeadlineExceeded => read_stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
        )),
        Query::Authorization(kind) => authorization(kind, detail),
        Query::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => snapshot_capacity_reconstruction_denial(maximum_active_snapshots),
        Query::RetentionCapacityExhausted => retention_capacity_reconstruction_denial(),
        Query::RetentionIdentityExhausted => retention_identity_reconstruction_denial(),
        Query::SnapshotIdentityExhausted => snapshot_identity_reconstruction_denial(),
        Query::InstalledQuery(..)
        | Query::ForeignPrincipal
        | Query::ForeignScope
        | Query::StalePrincipal
        | Query::StaleScope
        | Query::ScopeTypeMismatch
        | Query::BasisUnsupported
        | Query::ForeignBasis
        | Query::StaleBasis
        | Query::WrongProviderBasis
        | Query::ExpiredBasis
        | Query::BasisUnavailable
        | Query::ForeignHistoricalReceipt
        | Query::RuntimeSupportUnavailable
        | Query::ForeignContinuation
        | Query::StaleContinuation
        | Query::ContinuationParameterMismatch
        | Query::ContinuationScopeMismatch
        | Query::ContinuationProviderMismatch
        | Query::ContinuationPageWidthUnsupported
        | Query::LaneUnsupported
        | Query::DisclosureGovernanceRequired
        | Query::DisclosureContractInvalid
        | Query::DisclosureAuthorizationMismatch
        | Query::InternalComputationDenied
        | Query::Parameter(..)
        | Query::WorkLimitExceeded
        | Query::ReadmissionPreparationMemoryExhausted
        | Query::CanonicalWorkDenied
        | Query::GraphReadPlan(..)
        | Query::GraphWorkAdmissionUnavailable
        | Query::ExecutionShapeUnsupported
        | Query::BranchMaterializationSuspended => reconstruction_denial(
            WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionQuery,
            detail,
        ),
    }
}
pub(super) fn authorization(
    kind: Authorization,
    detail: String,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    if kind == Authorization::Refused(AuthorizationRefusal::RetentionIdentityExhausted) {
        return retention_identity_reconstruction_denial();
    }
    if kind == Authorization::Refused(AuthorizationRefusal::SnapshotIdentityExhausted) {
        return snapshot_identity_reconstruction_denial();
    }
    match Reentry::authorization_kind(kind, crate::domain_computation::primary_graph::conditional_operation::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure::Authorization) {
  Reentry::ExecutionStopped(kind) => read_stopped(Ok(kind)),
  Reentry::ControlStopped(Control::Cancelled) => read_stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled)),
  Reentry::ControlStopped(Control::TimedOut) => read_stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut)),
  Reentry::ActiveSnapshotCapacityExhausted { maximum_active_snapshots } => snapshot_capacity_reconstruction_denial(maximum_active_snapshots), Reentry::RetentionCapacityExhausted => retention_capacity_reconstruction_denial(),
  Reentry::Terminal(_) | Reentry::Retryable(_) => reconstruction_denial(WorthQueryConditionalRuntimeInstallationDenialKind::ReconstructionQuery, detail),
 }
}
