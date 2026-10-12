//! authorization refusal at conditional reentry.
use super::WorthQueryTemporalReentryDenial;
use crate::domain_computation::primary_graph::{WorthQueryOperationAuthorizationDenialKind as Kind, WorthQueryOperationAuthorizationDenial as Failure};
use crate::domain_computation::primary_graph::conditional_operation::application_operation_reentry::WorthQueryTemporalAdmissionTerminalFailure as Terminal;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
impl WorthQueryTemporalReentryDenial {
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_authorization(
        denial: Failure,
    ) -> Self {
        Self::authorization_kind(denial.kind(), Terminal::Authorization)
    }
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn authorization_kind(
        kind: Kind,
        refused: impl FnOnce(Kind) -> Terminal,
    ) -> Self {
        match kind {
            Kind::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            Kind::RetentionCapacityExhausted => Self::RetentionCapacityExhausted,
            Kind::ExecutionRequest(cause) => Self::from_read_stop(Ok(cause)),
            Kind::Cancelled => Self::from_native_stop(Native::Cancelled),
            Kind::DeadlineExceeded => Self::from_native_stop(Native::DeadlineElapsed),
            Kind::ProductSecurityBasis(
                crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(stop),
            ) => Self::from_read_stop(stop),
            kind @ Kind::ProductSecurityBasis(
                crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_),
            ) => Self::Terminal(refused(kind)),
            kind @ (Kind::ForeignPhase
            | Kind::ExpiredAuthentication
            | Kind::ForeignRuntime
            | Kind::StaleInstalledSchema
            | Kind::StaleInstalledOperation
            | Kind::StalePrincipal
            | Kind::StaleScope
            | Kind::MutationPreconditionRejected
            | Kind::CanonicalWorkDenied
            | Kind::TrustedTimeUnavailable
            | Kind::InvalidOperationInput
            | Kind::CapabilityProjectionRejected
            | Kind::CapabilityGrantMissing
            | Kind::CapabilityAuthorizationMissing
            | Kind::PurposeMismatch
            | Kind::ExplicitDenyRuleMatched
            | Kind::ConflictRuleMatched
            | Kind::SeparationOfDutyRuleMatched
            | Kind::DistinctActorRuleMatched
            | Kind::CapabilityRequired
            | Kind::CapabilityNotRequired
            | Kind::CapabilityExpired
            | Kind::ElevationRequired
            | Kind::ElevationNotApplicable
            | Kind::ElevationProjectionRejected
            | Kind::ElevationExpired
            | Kind::ElevationInactive
            | Kind::ElevationSelfApproval
            | Kind::ElevationApproverConflict
            | Kind::ElevationTransitionRequired
            | Kind::ElevationLifecycleRoleMismatch
            | Kind::ElevationRequestRejected
            | Kind::ElevationApprovalRejected
            | Kind::ElevationCloseRejected
            | Kind::MandatoryReviewRejected
            | Kind::ElevationDurationExceeded
            | Kind::DelegationRejected
            | Kind::DelegationTransitionRequired
            | Kind::DelegationDepthExceeded
            | Kind::DelegationCycle
            | Kind::DelegationLineageChanged
            | Kind::StaleAuthorization
            | Kind::AdmissionIdentityExhausted
            | Kind::SnapshotIdentityExhausted
            | Kind::RetentionIdentityExhausted
            | Kind::GraphWorkAdmissionUnavailable
            | Kind::ScopeMismatch
            | Kind::PolicyNotInstalled
            | Kind::InvalidInstalledPolicy
            | Kind::RelationalObservationRejected
            | Kind::GrantSelectionLimitExceeded
            | Kind::BridgeEvaluationRejected
            | Kind::InconsistentDecision
            | Kind::PermissionDenied) => Self::Terminal(refused(kind)),
        }
    }
}
