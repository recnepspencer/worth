//! Workflow approval readmission keeps request stops distinct from authority refusal.
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryOperationAuthorizationDenial,
};
pub(super) fn denial(
    denial: WorthQueryOperationAuthorizationDenial,
) -> WorthQueryApplicationAttemptDenial {
    use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind;
    let kind = match denial.kind() {
        WorthQueryOperationAuthorizationDenialKind::StalePrincipal => {
            WorthQueryApplicationAttemptDenialKind::WorkflowApprovalPrincipalStale
        }
        WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing => {
            WorthQueryApplicationAttemptDenialKind::WorkflowApprovalGrantUnavailable
        }
        WorthQueryOperationAuthorizationDenialKind::CapabilityExpired => {
            WorthQueryApplicationAttemptDenialKind::WorkflowApprovalExpired
        }
        WorthQueryOperationAuthorizationDenialKind::DelegationLineageChanged
        | WorthQueryOperationAuthorizationDenialKind::DelegationRejected => {
            WorthQueryApplicationAttemptDenialKind::WorkflowApprovalDelegationChanged
        }
        WorthQueryOperationAuthorizationDenialKind::TrustedTimeUnavailable => {
            WorthQueryApplicationAttemptDenialKind::WorkflowTrustedTimeUnavailable
        }
        WorthQueryOperationAuthorizationDenialKind::ExecutionRequest(kind) => {
            WorthQueryApplicationAttemptDenialKind::ExecutionRequest(kind)
        }
        WorthQueryOperationAuthorizationDenialKind::Cancelled => {
            WorthQueryApplicationAttemptDenialKind::Cancelled
        }
        WorthQueryOperationAuthorizationDenialKind::DeadlineExceeded => {
            WorthQueryApplicationAttemptDenialKind::DeadlineExceeded
        }
        WorthQueryOperationAuthorizationDenialKind::ProductSecurityBasis(
            crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(stop),
        ) => match stop {
            Ok(kind) => WorthQueryApplicationAttemptDenialKind::ExecutionRequest(kind),
            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled) => {
                WorthQueryApplicationAttemptDenialKind::Cancelled
            }
            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut) => {
                WorthQueryApplicationAttemptDenialKind::DeadlineExceeded
            }
        },
        WorthQueryOperationAuthorizationDenialKind::ForeignPhase
        | WorthQueryOperationAuthorizationDenialKind::ExpiredAuthentication
        | WorthQueryOperationAuthorizationDenialKind::ForeignRuntime
        | WorthQueryOperationAuthorizationDenialKind::StaleInstalledSchema
        | WorthQueryOperationAuthorizationDenialKind::StaleInstalledOperation
        | WorthQueryOperationAuthorizationDenialKind::StaleScope
        | WorthQueryOperationAuthorizationDenialKind::MutationPreconditionRejected
        | WorthQueryOperationAuthorizationDenialKind::CanonicalWorkDenied
        | WorthQueryOperationAuthorizationDenialKind::InvalidOperationInput
        | WorthQueryOperationAuthorizationDenialKind::CapabilityProjectionRejected
        | WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
        | WorthQueryOperationAuthorizationDenialKind::PurposeMismatch
        | WorthQueryOperationAuthorizationDenialKind::ExplicitDenyRuleMatched
        | WorthQueryOperationAuthorizationDenialKind::ConflictRuleMatched
        | WorthQueryOperationAuthorizationDenialKind::SeparationOfDutyRuleMatched
        | WorthQueryOperationAuthorizationDenialKind::DistinctActorRuleMatched
        | WorthQueryOperationAuthorizationDenialKind::CapabilityRequired
        | WorthQueryOperationAuthorizationDenialKind::CapabilityNotRequired
        | WorthQueryOperationAuthorizationDenialKind::ElevationRequired
        | WorthQueryOperationAuthorizationDenialKind::ElevationNotApplicable
        | WorthQueryOperationAuthorizationDenialKind::ElevationProjectionRejected
        | WorthQueryOperationAuthorizationDenialKind::ElevationExpired
        | WorthQueryOperationAuthorizationDenialKind::ElevationInactive
        | WorthQueryOperationAuthorizationDenialKind::ElevationSelfApproval
        | WorthQueryOperationAuthorizationDenialKind::ElevationApproverConflict
        | WorthQueryOperationAuthorizationDenialKind::ElevationTransitionRequired
        | WorthQueryOperationAuthorizationDenialKind::ElevationLifecycleRoleMismatch
        | WorthQueryOperationAuthorizationDenialKind::ElevationRequestRejected
        | WorthQueryOperationAuthorizationDenialKind::ElevationApprovalRejected
        | WorthQueryOperationAuthorizationDenialKind::ElevationCloseRejected
        | WorthQueryOperationAuthorizationDenialKind::MandatoryReviewRejected
        | WorthQueryOperationAuthorizationDenialKind::ElevationDurationExceeded
        | WorthQueryOperationAuthorizationDenialKind::DelegationTransitionRequired
        | WorthQueryOperationAuthorizationDenialKind::DelegationDepthExceeded
        | WorthQueryOperationAuthorizationDenialKind::DelegationCycle
        | WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
        | WorthQueryOperationAuthorizationDenialKind::ProductSecurityBasis(
            crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_),
        )
        | WorthQueryOperationAuthorizationDenialKind::AdmissionIdentityExhausted
        | WorthQueryOperationAuthorizationDenialKind::ActiveSnapshotCapacityExhausted { .. }
        | WorthQueryOperationAuthorizationDenialKind::SnapshotIdentityExhausted
        | WorthQueryOperationAuthorizationDenialKind::RetentionCapacityExhausted
        | WorthQueryOperationAuthorizationDenialKind::RetentionIdentityExhausted
        | WorthQueryOperationAuthorizationDenialKind::GraphWorkAdmissionUnavailable
        | WorthQueryOperationAuthorizationDenialKind::ScopeMismatch
        | WorthQueryOperationAuthorizationDenialKind::PolicyNotInstalled
        | WorthQueryOperationAuthorizationDenialKind::InvalidInstalledPolicy
        | WorthQueryOperationAuthorizationDenialKind::RelationalObservationRejected
        | WorthQueryOperationAuthorizationDenialKind::GrantSelectionLimitExceeded
        | WorthQueryOperationAuthorizationDenialKind::BridgeEvaluationRejected
        | WorthQueryOperationAuthorizationDenialKind::InconsistentDecision
        | WorthQueryOperationAuthorizationDenialKind::PermissionDenied => {
            WorthQueryApplicationAttemptDenialKind::WorkflowApprovalAuthorityDenied
        }
    };
    WorthQueryApplicationAttemptDenial::new(kind, denial.to_string())
}
