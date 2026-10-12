//! The retained request authority exposes execution causes without changing ordinary authorization.

pub(super) fn authority_execution(
    kind: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
) -> Option<
    Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    >,
> {
    use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as Authority;
    match kind {
        Authority::ExecutionRequest(kind) => Some(Ok(kind)),
        Authority::ProductSecurityBasis(
            crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(stop),
        ) => Some(stop),
        Authority::ProductSecurityBasis(
            crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_),
        )
        | Authority::ForeignPhase
        | Authority::Cancelled
        | Authority::DeadlineExceeded
        | Authority::ExpiredAuthentication
        | Authority::ForeignRuntime
        | Authority::StaleInstalledSchema
        | Authority::StaleInstalledOperation
        | Authority::StalePrincipal
        | Authority::StaleScope
        | Authority::MutationPreconditionRejected
        | Authority::CanonicalWorkDenied
        | Authority::TrustedTimeUnavailable
        | Authority::InvalidOperationInput
        | Authority::CapabilityProjectionRejected
        | Authority::CapabilityGrantMissing
        | Authority::CapabilityAuthorizationMissing
        | Authority::PurposeMismatch
        | Authority::ExplicitDenyRuleMatched
        | Authority::ConflictRuleMatched
        | Authority::SeparationOfDutyRuleMatched
        | Authority::DistinctActorRuleMatched
        | Authority::CapabilityRequired
        | Authority::CapabilityNotRequired
        | Authority::CapabilityExpired
        | Authority::ElevationRequired
        | Authority::ElevationNotApplicable
        | Authority::ElevationProjectionRejected
        | Authority::ElevationExpired
        | Authority::ElevationInactive
        | Authority::ElevationSelfApproval
        | Authority::ElevationApproverConflict
        | Authority::ElevationTransitionRequired
        | Authority::ElevationLifecycleRoleMismatch
        | Authority::ElevationRequestRejected
        | Authority::ElevationApprovalRejected
        | Authority::ElevationCloseRejected
        | Authority::MandatoryReviewRejected
        | Authority::ElevationDurationExceeded
        | Authority::DelegationRejected
        | Authority::DelegationTransitionRequired
        | Authority::DelegationDepthExceeded
        | Authority::DelegationCycle
        | Authority::DelegationLineageChanged
        | Authority::StaleAuthorization
        | Authority::AdmissionIdentityExhausted
        | Authority::ActiveSnapshotCapacityExhausted { .. }
        | Authority::SnapshotIdentityExhausted
        | Authority::RetentionCapacityExhausted
        | Authority::RetentionIdentityExhausted
        | Authority::GraphWorkAdmissionUnavailable
        | Authority::ScopeMismatch
        | Authority::PolicyNotInstalled
        | Authority::InvalidInstalledPolicy
        | Authority::RelationalObservationRejected
        | Authority::GrantSelectionLimitExceeded
        | Authority::BridgeEvaluationRejected
        | Authority::InconsistentDecision
        | Authority::PermissionDenied => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::domain_computation::primary_graph::{
        WorthQueryApplicationCommitDenial as Denial,
        WorthQueryApplicationCommitDenialStage as Stage,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    };
    use crate::domain_computation::{
        authorization::{
            WorthQueryOperationAuthorizationDenial as Authorization,
            WorthQueryOperationAuthorizationDenialKind as Authority,
        },
        WorthQueryProviderSessionDenialKind as Kind,
    };
    #[test]
    fn authority_evidence_exposes_request_resources_by_value_and_keeps_ordinary_refusal() {
        for denial in [
            Resource::WorkExhausted,
            Resource::MemoryLimit {
                requested: 43,
                admitted: 17,
                level: Level::Policy,
            },
        ] {
            let policy_ancestor = if matches!(
                denial,
                Resource::MemoryLimit {
                    level: Level::Policy,
                    ..
                }
            ) {
                Some(2)
            } else {
                None
            };
            let kind = Kind::ExecutionResource {
                denial,
                partition_identity: None,
                policy_ancestor,
            };
            let authorization =
                Authorization::new(Authority::ExecutionRequest(kind), "authorization read");
            let outcome = Denial::request_authority_denied(Stage::DecisionReadSet, &authorization);
            assert_eq!(outcome.execution_denial_cause(), Some(Ok(kind)));
            assert_eq!(
                outcome.request_authority(),
                Some(Authority::ExecutionRequest(kind))
            );
            assert_eq!(outcome.stage(), Stage::DecisionReadSet);
        }
        let ordinary = Authorization::new(Authority::ExplicitDenyRuleMatched, "authorization rule");
        let outcome = Denial::request_authority_denied(Stage::DecisionReadSet, &ordinary);
        assert_eq!(outcome.execution_denial_cause(), None);
        assert_eq!(
            outcome.request_authority(),
            Some(Authority::ExplicitDenyRuleMatched)
        );
    }
}
