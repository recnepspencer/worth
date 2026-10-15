//! Workflow approval readmission keeps request stops distinct from authority refusal.
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryOperationAuthorizationDenial,
};
pub(super) fn denial(
    denial: WorthQueryOperationAuthorizationDenial,
) -> WorthQueryApplicationAttemptDenial {
    use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind;
    let kind = match denial.kind() {
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StalePrincipal,
        ) => WorthQueryApplicationAttemptDenialKind::WorkflowApprovalPrincipalStale,
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityGrantMissing,
        ) => WorthQueryApplicationAttemptDenialKind::WorkflowApprovalGrantUnavailable,
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityExpired,
        ) => WorthQueryApplicationAttemptDenialKind::WorkflowApprovalExpired,
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::DelegationLineageChanged,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::DelegationRejected,
        ) => WorthQueryApplicationAttemptDenialKind::WorkflowApprovalDelegationChanged,
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::TrustedTimeUnavailable,
        ) => WorthQueryApplicationAttemptDenialKind::WorkflowTrustedTimeUnavailable,
        WorthQueryOperationAuthorizationDenialKind::Stopped(Ok(kind)) => {
            WorthQueryApplicationAttemptDenialKind::ExecutionRequest(kind)
        }
        WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
        )) => WorthQueryApplicationAttemptDenialKind::Cancelled,
        WorthQueryOperationAuthorizationDenialKind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
        )) => WorthQueryApplicationAttemptDenialKind::DeadlineExceeded,
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ForeignPhase)
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ExpiredAuthentication,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ForeignRuntime,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleInstalledSchema,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleInstalledOperation,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::StaleScope)
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::MutationPreconditionRejected,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CanonicalWorkDenied,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InvalidOperationInput,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityProjectionRejected,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityAuthorizationMissing,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::WorkflowActorContextRequired,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::WorkflowActorAbsent,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::WorkflowActorComparisonMismatch,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::WorkflowActorSourceInvalid,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::WorkflowActorEvidenceUnavailable,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::PurposeMismatch,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ExplicitDenyRuleMatched,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ConflictRuleMatched,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::SeparationOfDutyRuleMatched,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::DistinctActorRuleMatched,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityRequired,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityNotRequired,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationRequired,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationNotApplicable,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationProjectionRejected,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationExpired,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationInactive,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationSelfApproval,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationApproverConflict,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationTransitionRequired,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationLifecycleRoleMismatch,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationRequestRejected,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationApprovalRejected,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationCloseRejected,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::MandatoryReviewRejected,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ElevationDurationExceeded,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::DelegationTransitionRequired,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::DelegationDepthExceeded,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::DelegationCycle,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::StaleAuthorization,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ProductSecurityBasis(_),
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::AdmissionIdentityExhausted,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ActiveSnapshotCapacityExhausted { .. },
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::SnapshotIdentityExhausted,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::RetentionCapacityExhausted,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::RetentionIdentityExhausted,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::GraphWorkAdmissionUnavailable,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::ScopeMismatch,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::PolicyNotInstalled,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InvalidInstalledPolicy,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::RelationalObservationRejected,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::GrantSelectionLimitExceeded,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::BridgeEvaluationRejected,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InconsistentDecision,
        )
        | WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::PermissionDenied,
        ) => WorthQueryApplicationAttemptDenialKind::WorkflowApprovalAuthorityDenied,
    };
    WorthQueryApplicationAttemptDenial::authorization_denied(kind, denial)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn approval_readmission_preserves_native_actor_source_cause() {
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
        for cause in [
            Native::Cancelled,
            Native::DeclaredMemoryExhausted {
                requested: 37,
                admitted: 0,
            },
        ] {
            let original = WorthQueryOperationAuthorizationDenial::workflow_actor_native(
                WorthQueryApplicationAttemptDenial::relational_read_stopped(cause, "actor source"),
            );
            let expected = original.clone();
            let mapped = denial(original);
            assert_eq!(mapped.request_authority(), Some(&expected));
            let source = mapped
                .request_authority()
                .unwrap()
                .workflow_actor_source_denial()
                .unwrap();
            assert_eq!(source.relational_execution_cause(), Some(cause));
            assert!(std::error::Error::source(&mapped).is_some());
        }
    }
}
