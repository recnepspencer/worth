//! authorization refusal at conditional reentry.
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
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
            Kind::Refused(AuthorizationRefusal::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            }) => Self::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            },
            Kind::Refused(AuthorizationRefusal::RetentionCapacityExhausted) => {
                Self::RetentionCapacityExhausted
            }
            Kind::Stopped(Ok(cause)) => Self::from_read_stop(Ok(cause)),
            Kind::Stopped(Err(
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
            )) => Self::from_native_stop(Native::Cancelled),
            Kind::Stopped(Err(
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
            )) => Self::from_native_stop(Native::DeadlineElapsed),
            kind @ Kind::Refused(AuthorizationRefusal::ProductSecurityBasis(_)) => {
                Self::Terminal(refused(kind))
            }
            kind @ (Kind::Refused(AuthorizationRefusal::ForeignPhase)
            | Kind::Refused(AuthorizationRefusal::ExpiredAuthentication)
            | Kind::Refused(AuthorizationRefusal::ForeignRuntime)
            | Kind::Refused(AuthorizationRefusal::StaleInstalledSchema)
            | Kind::Refused(AuthorizationRefusal::StaleInstalledOperation)
            | Kind::Refused(AuthorizationRefusal::StalePrincipal)
            | Kind::Refused(AuthorizationRefusal::StaleScope)
            | Kind::Refused(AuthorizationRefusal::MutationPreconditionRejected)
            | Kind::Refused(AuthorizationRefusal::CanonicalWorkDenied)
            | Kind::Refused(AuthorizationRefusal::TrustedTimeUnavailable)
            | Kind::Refused(AuthorizationRefusal::InvalidOperationInput)
            | Kind::Refused(AuthorizationRefusal::CapabilityProjectionRejected)
            | Kind::Refused(AuthorizationRefusal::CapabilityGrantMissing)
            | Kind::Refused(AuthorizationRefusal::CapabilityAuthorizationMissing)
            | Kind::Refused(AuthorizationRefusal::PurposeMismatch)
            | Kind::Refused(AuthorizationRefusal::ExplicitDenyRuleMatched)
            | Kind::Refused(AuthorizationRefusal::ConflictRuleMatched)
            | Kind::Refused(AuthorizationRefusal::SeparationOfDutyRuleMatched)
            | Kind::Refused(AuthorizationRefusal::DistinctActorRuleMatched)
            | Kind::Refused(AuthorizationRefusal::CapabilityRequired)
            | Kind::Refused(AuthorizationRefusal::CapabilityNotRequired)
            | Kind::Refused(AuthorizationRefusal::CapabilityExpired)
            | Kind::Refused(AuthorizationRefusal::ElevationRequired)
            | Kind::Refused(AuthorizationRefusal::ElevationNotApplicable)
            | Kind::Refused(AuthorizationRefusal::ElevationProjectionRejected)
            | Kind::Refused(AuthorizationRefusal::ElevationExpired)
            | Kind::Refused(AuthorizationRefusal::ElevationInactive)
            | Kind::Refused(AuthorizationRefusal::ElevationSelfApproval)
            | Kind::Refused(AuthorizationRefusal::ElevationApproverConflict)
            | Kind::Refused(AuthorizationRefusal::ElevationTransitionRequired)
            | Kind::Refused(AuthorizationRefusal::ElevationLifecycleRoleMismatch)
            | Kind::Refused(AuthorizationRefusal::ElevationRequestRejected)
            | Kind::Refused(AuthorizationRefusal::ElevationApprovalRejected)
            | Kind::Refused(AuthorizationRefusal::ElevationCloseRejected)
            | Kind::Refused(AuthorizationRefusal::MandatoryReviewRejected)
            | Kind::Refused(AuthorizationRefusal::ElevationDurationExceeded)
            | Kind::Refused(AuthorizationRefusal::DelegationRejected)
            | Kind::Refused(AuthorizationRefusal::DelegationTransitionRequired)
            | Kind::Refused(AuthorizationRefusal::DelegationDepthExceeded)
            | Kind::Refused(AuthorizationRefusal::DelegationCycle)
            | Kind::Refused(AuthorizationRefusal::DelegationLineageChanged)
            | Kind::Refused(AuthorizationRefusal::StaleAuthorization)
            | Kind::Refused(AuthorizationRefusal::AdmissionIdentityExhausted)
            | Kind::Refused(AuthorizationRefusal::SnapshotIdentityExhausted)
            | Kind::Refused(AuthorizationRefusal::RetentionIdentityExhausted)
            | Kind::Refused(AuthorizationRefusal::GraphWorkAdmissionUnavailable)
            | Kind::Refused(AuthorizationRefusal::ScopeMismatch)
            | Kind::Refused(AuthorizationRefusal::PolicyNotInstalled)
            | Kind::Refused(AuthorizationRefusal::InvalidInstalledPolicy)
            | Kind::Refused(AuthorizationRefusal::RelationalObservationRejected)
            | Kind::Refused(AuthorizationRefusal::GrantSelectionLimitExceeded)
            | Kind::Refused(AuthorizationRefusal::BridgeEvaluationRejected)
            | Kind::Refused(AuthorizationRefusal::InconsistentDecision)
            | Kind::Refused(AuthorizationRefusal::PermissionDenied)) => {
                Self::Terminal(refused(kind))
            }
        }
    }
}
