use worth_query_execution::facade::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_query_execution::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_execution::facade::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationOneShotDenialKind as ExecutionKind,
    WorthQueryApplicationQueryAdmissionDenialKind as AdmissionKind,
    WorthQueryEntityResolutionDenialKind as ScopeKind,
    WorthQueryOperationAuthorizationDenialKind as AuthorizationKind,
    WorthQueryPrincipalResolutionDenialKind as PrincipalKind,
    WorthQueryProductBranchAdmissionDenial as ProductKind,
};

use crate::application_entry::WorthQueryApplicationRequestQueryDenial;

use super::{output_posture, WorthQueryRequiredOutputRecoveryPosture};

pub(super) fn query_posture(
    denial: &WorthQueryApplicationRequestQueryDenial,
) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryApplicationRequestQueryDenial as Denial;
    match denial {
        Denial::ExecutionRequest(cause) => {
            if cause.is_transient() {
                WorthQueryRequiredOutputRecoveryPosture::Retryable
            } else {
                WorthQueryRequiredOutputRecoveryPosture::Terminal
            }
        }
        Denial::BindingInstallation(_) | Denial::Limit(_) => {
            WorthQueryRequiredOutputRecoveryPosture::Terminal
        }
        Denial::ProductSelection(denial) => product_posture(*denial),
        Denial::OutputSettlement(output) => output_posture(output),
        Denial::PrincipalResolution(denial) => principal_posture(denial.kind()),
        Denial::ScopeResolution(denial) => scope_posture(denial.kind()),
        Denial::Admission(denial) => admission_posture(denial.kind()),
        Denial::Execution(denial) => execution_posture(denial.kind()),
        Denial::ContinuationExecution(_)
        | Denial::RequestMode
        | Denial::CapabilityInstallation(_)
        | Denial::CapabilityAdmission(_) => WorthQueryRequiredOutputRecoveryPosture::Terminal,
    }
}

fn product_posture(kind: ProductKind) -> WorthQueryRequiredOutputRecoveryPosture {
    match kind {
        ProductKind::Stopped(_) => WorthQueryRequiredOutputRecoveryPosture::Terminal,
        ProductKind::Refused(refusal) => {
            if refusal.is_transient() {
                WorthQueryRequiredOutputRecoveryPosture::Retryable
            } else {
                WorthQueryRequiredOutputRecoveryPosture::Terminal
            }
        }
    }
}

fn authorization_posture(kind: AuthorizationKind) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryRequiredOutputRecoveryPosture::{Retryable, Terminal};
    match kind {
        AuthorizationKind::Refused(AuthorizationRefusal::ProductSecurityBasis(refusal)) => product_posture(ProductKind::Refused(refusal)),
        AuthorizationKind::Stopped(Err(worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind::Cancelled))
        | AuthorizationKind::Stopped(Err(worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind::TimedOut))
        | AuthorizationKind::Refused(AuthorizationRefusal::TrustedTimeUnavailable)
        | AuthorizationKind::Refused(AuthorizationRefusal::ActiveSnapshotCapacityExhausted{ .. })
        | AuthorizationKind::Refused(AuthorizationRefusal::RetentionCapacityExhausted)
        | AuthorizationKind::Refused(AuthorizationRefusal::GraphWorkAdmissionUnavailable) => Retryable,
        AuthorizationKind::Stopped(Ok(..))
        | AuthorizationKind::Refused(AuthorizationRefusal::ForeignPhase)
        | AuthorizationKind::Refused(AuthorizationRefusal::ExpiredAuthentication)
        | AuthorizationKind::Refused(AuthorizationRefusal::ForeignRuntime)
        | AuthorizationKind::Refused(AuthorizationRefusal::StaleInstalledSchema)
        | AuthorizationKind::Refused(AuthorizationRefusal::StaleInstalledOperation)
        | AuthorizationKind::Refused(AuthorizationRefusal::StalePrincipal)
        | AuthorizationKind::Refused(AuthorizationRefusal::StaleScope)
        | AuthorizationKind::Refused(AuthorizationRefusal::MutationPreconditionRejected)
        | AuthorizationKind::Refused(AuthorizationRefusal::CanonicalWorkDenied)
        | AuthorizationKind::Refused(AuthorizationRefusal::InvalidOperationInput)
        | AuthorizationKind::Refused(AuthorizationRefusal::CapabilityProjectionRejected)
        | AuthorizationKind::Refused(AuthorizationRefusal::CapabilityGrantMissing)
        | AuthorizationKind::Refused(AuthorizationRefusal::CapabilityAuthorizationMissing)
        | AuthorizationKind::Refused(AuthorizationRefusal::PurposeMismatch)
        | AuthorizationKind::Refused(AuthorizationRefusal::ExplicitDenyRuleMatched)
        | AuthorizationKind::Refused(AuthorizationRefusal::ConflictRuleMatched)
        | AuthorizationKind::Refused(AuthorizationRefusal::SeparationOfDutyRuleMatched)
        | AuthorizationKind::Refused(AuthorizationRefusal::DistinctActorRuleMatched)
        | AuthorizationKind::Refused(AuthorizationRefusal::CapabilityRequired)
        | AuthorizationKind::Refused(AuthorizationRefusal::CapabilityNotRequired)
        | AuthorizationKind::Refused(AuthorizationRefusal::CapabilityExpired)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationRequired)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationNotApplicable)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationProjectionRejected)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationExpired)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationInactive)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationSelfApproval)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationApproverConflict)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationTransitionRequired)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationLifecycleRoleMismatch)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationRequestRejected)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationApprovalRejected)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationCloseRejected)
        | AuthorizationKind::Refused(AuthorizationRefusal::MandatoryReviewRejected)
        | AuthorizationKind::Refused(AuthorizationRefusal::ElevationDurationExceeded)
        | AuthorizationKind::Refused(AuthorizationRefusal::DelegationRejected)
        | AuthorizationKind::Refused(AuthorizationRefusal::DelegationTransitionRequired)
        | AuthorizationKind::Refused(AuthorizationRefusal::DelegationDepthExceeded)
        | AuthorizationKind::Refused(AuthorizationRefusal::DelegationCycle)
        | AuthorizationKind::Refused(AuthorizationRefusal::DelegationLineageChanged)
        | AuthorizationKind::Refused(AuthorizationRefusal::StaleAuthorization)
        | AuthorizationKind::Refused(AuthorizationRefusal::AdmissionIdentityExhausted)
        | AuthorizationKind::Refused(AuthorizationRefusal::SnapshotIdentityExhausted)
        | AuthorizationKind::Refused(AuthorizationRefusal::RetentionIdentityExhausted)
        | AuthorizationKind::Refused(AuthorizationRefusal::ScopeMismatch)
        | AuthorizationKind::Refused(AuthorizationRefusal::PolicyNotInstalled)
        | AuthorizationKind::Refused(AuthorizationRefusal::InvalidInstalledPolicy)
        | AuthorizationKind::Refused(AuthorizationRefusal::RelationalObservationRejected)
        | AuthorizationKind::Refused(AuthorizationRefusal::GrantSelectionLimitExceeded)
        | AuthorizationKind::Refused(AuthorizationRefusal::BridgeEvaluationRejected)
        | AuthorizationKind::Refused(AuthorizationRefusal::InconsistentDecision)
        | AuthorizationKind::Refused(AuthorizationRefusal::PermissionDenied) => Terminal,
    }
}

fn principal_posture(kind: PrincipalKind) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryRequiredOutputRecoveryPosture::{Retryable, Terminal};
    match kind {
        PrincipalKind::Stopped(Ok(_)) => Terminal,
        PrincipalKind::Refused(PrincipalRefusal::ExpiredAuthentication)
        | PrincipalKind::Stopped(Err(worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind::Cancelled))
        | PrincipalKind::Stopped(Err(worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind::TimedOut))
        | PrincipalKind::Refused(PrincipalRefusal::BranchMaterializationSuspended)
        | PrincipalKind::Refused(PrincipalRefusal::IdentityIndexUnavailable)
        | PrincipalKind::Refused(PrincipalRefusal::ActiveSnapshotCapacityExhausted{ .. })
        | PrincipalKind::Refused(PrincipalRefusal::RetentionCapacityExhausted) => Retryable,
        PrincipalKind::Refused(PrincipalRefusal::PrimaryGraphNotInstalled)
        | PrincipalKind::Refused(PrincipalRefusal::BindingNotInstalled)
        | PrincipalKind::Refused(PrincipalRefusal::ForeignRuntime)
        | PrincipalKind::Refused(PrincipalRefusal::StaleInstalledSchema)
        | PrincipalKind::Refused(PrincipalRefusal::CorruptIdentityIndex)
        | PrincipalKind::Refused(PrincipalRefusal::UnknownPrincipal)
        | PrincipalKind::Refused(PrincipalRefusal::DisabledPrincipal)
        | PrincipalKind::Refused(PrincipalRefusal::AmbiguousPrincipal)
        | PrincipalKind::Refused(PrincipalRefusal::MissingPrincipalTarget)
        | PrincipalKind::Refused(PrincipalRefusal::AmbiguousPrincipalTarget)
        | PrincipalKind::Refused(PrincipalRefusal::WrongPrincipalTargetKind)
        | PrincipalKind::Refused(PrincipalRefusal::StalePrincipalProof)
        | PrincipalKind::Refused(PrincipalRefusal::SnapshotIdentityExhausted)
        | PrincipalKind::Refused(PrincipalRefusal::RetentionIdentityExhausted) => Terminal,
        PrincipalKind::Refused(PrincipalRefusal::ForeignPhase) => Terminal,
    }
}

fn scope_posture(kind: ScopeKind) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryRequiredOutputRecoveryPosture::{Retryable, Terminal};
    match kind {
        ScopeKind::Stopped(cause) => {
            match worth_query_execution::facade::primary_graph::relational_execution_kind(
                cause, None,
            ) {
                Err(_) => Retryable,
                Ok(_) => Terminal,
            }
        }
        ScopeKind::Refused(ResolutionRefusal::EqualityIndexUnavailable)
        | ScopeKind::Refused(ResolutionRefusal::ActiveSnapshotCapacityExhausted { .. })
        | ScopeKind::Refused(ResolutionRefusal::RetentionCapacityExhausted) => Retryable,
        ScopeKind::Refused(ResolutionRefusal::ForeignPhase)
        | ScopeKind::Refused(ResolutionRefusal::PrimaryGraphNotInstalled)
        | ScopeKind::Refused(ResolutionRefusal::FieldNotInstalled)
        | ScopeKind::Refused(ResolutionRefusal::ValueEncodingRejected)
        | ScopeKind::Refused(ResolutionRefusal::UnknownEntity)
        | ScopeKind::Refused(ResolutionRefusal::AmbiguousEntity)
        | ScopeKind::Refused(ResolutionRefusal::CorruptIdentityIndex)
        | ScopeKind::Refused(ResolutionRefusal::InvalidCandidateLimit)
        | ScopeKind::Refused(ResolutionRefusal::CandidateLimitExceeded { .. })
        | ScopeKind::Refused(ResolutionRefusal::SnapshotIdentityExhausted)
        | ScopeKind::Refused(ResolutionRefusal::RetentionIdentityExhausted)
        | ScopeKind::Refused(ResolutionRefusal::ForeignResolutionTruth) => Terminal,
    }
}

fn admission_posture(kind: AdmissionKind) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryRequiredOutputRecoveryPosture::{Retryable, Terminal};
    match kind {
        AdmissionKind::ExecutionRequest(_) => Retryable,
        AdmissionKind::Cancelled
        | AdmissionKind::DeadlineExceeded
        | AdmissionKind::BasisUnavailable
        | AdmissionKind::BranchMaterializationSuspended
        | AdmissionKind::ActiveSnapshotCapacityExhausted { .. }
        | AdmissionKind::RetentionCapacityExhausted
        | AdmissionKind::GraphWorkAdmissionUnavailable => Retryable,
        AdmissionKind::Authorization(kind) => authorization_posture(kind),
        AdmissionKind::InstalledQuery(_)
        | AdmissionKind::ForeignPrincipal
        | AdmissionKind::ForeignScope
        | AdmissionKind::StalePrincipal
        | AdmissionKind::StaleScope
        | AdmissionKind::ScopeTypeMismatch
        | AdmissionKind::BasisUnsupported
        | AdmissionKind::ForeignBasis
        | AdmissionKind::StaleBasis
        | AdmissionKind::WrongProviderBasis
        | AdmissionKind::ExpiredBasis
        | AdmissionKind::ForeignHistoricalReceipt
        | AdmissionKind::RuntimeSupportUnavailable
        | AdmissionKind::SnapshotIdentityExhausted
        | AdmissionKind::RetentionIdentityExhausted
        | AdmissionKind::ForeignContinuation
        | AdmissionKind::StaleContinuation
        | AdmissionKind::ContinuationParameterMismatch
        | AdmissionKind::ContinuationScopeMismatch
        | AdmissionKind::ContinuationProviderMismatch
        | AdmissionKind::ContinuationPageWidthUnsupported
        | AdmissionKind::LaneUnsupported
        | AdmissionKind::DisclosureGovernanceRequired
        | AdmissionKind::DisclosureContractInvalid
        | AdmissionKind::DisclosureAuthorizationMismatch
        | AdmissionKind::InternalComputationDenied
        | AdmissionKind::Parameter(_)
        | AdmissionKind::WorkLimitExceeded
        | AdmissionKind::CanonicalWorkDenied
        | AdmissionKind::GraphReadPlan(_)
        | AdmissionKind::ExecutionShapeUnsupported => Terminal,
        AdmissionKind::ReadmissionPreparationMemoryExhausted => Terminal,
    }
}

fn execution_posture(kind: ExecutionKind) -> WorthQueryRequiredOutputRecoveryPosture {
    use WorthQueryRequiredOutputRecoveryPosture::{Retryable, Terminal};
    match kind {
        ExecutionKind::Cancelled
        | ExecutionKind::DeadlineExceeded
        | ExecutionKind::BasisUnavailable
        | ExecutionKind::ActiveSnapshotCapacityExhausted { .. }
        | ExecutionKind::RetentionCapacityExhausted
        | ExecutionKind::PredicateIndexUnavailable
        | ExecutionKind::TraversalUnavailable => Retryable,
        ExecutionKind::ExecutionRequest(_) => Retryable,
        ExecutionKind::Authorization(kind) => authorization_posture(kind),
        ExecutionKind::ForeignPlan
        | ExecutionKind::StaleInstalledQuery
        | ExecutionKind::StalePrincipal
        | ExecutionKind::StaleScope
        | ExecutionKind::RetentionIdentityExhausted
        | ExecutionKind::SnapshotIdentityExhausted
        | ExecutionKind::ExpiredBasis
        | ExecutionKind::BasisReleaseFailed
        | ExecutionKind::PredicateLookupOverflow
        | ExecutionKind::ResultLimitExceeded
        | ExecutionKind::CardinalityMismatch
        | ExecutionKind::ProjectionUnavailable
        | ExecutionKind::Projection(_)
        | ExecutionKind::ResultBufferLimitExceeded
        | ExecutionKind::WorkLimitExceeded
        | ExecutionKind::SourceIdentityExhausted => Terminal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_query_execution::facade::product::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;

    #[test]
    fn nested_world_capacity_is_retryable_but_foreign_authority_is_terminal() {
        assert_eq!(
            admission_posture(AdmissionKind::Authorization(AuthorizationKind::Refused(
                AuthorizationRefusal::ProductSecurityBasis(
                    AdmissionRefusal::ObservationCapacityExhausted
                )
            ),)),
            WorthQueryRequiredOutputRecoveryPosture::Retryable,
        );
        assert_eq!(
            execution_posture(ExecutionKind::Authorization(AuthorizationKind::Refused(
                AuthorizationRefusal::ForeignRuntime
            ),)),
            WorthQueryRequiredOutputRecoveryPosture::Terminal,
        );
    }
}

#[cfg(test)]
mod currency_request_tests {
    use super::*;
    #[test]
    fn currency_request_ceilings_keep_master_terminal_posture() {
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
        for cause in [
            Native::WorkExhausted,
            Native::DeclaredMemoryExhausted {
                requested: 43,
                admitted: 17,
            },
        ] {
            let stop = worth_query_execution::facade::primary_graph::relational_execution_kind(
                cause, None,
            );
            assert_eq!(
                (
                    principal_posture(PrincipalKind::Stopped(stop)),
                    scope_posture(ScopeKind::Stopped(cause))
                ),
                (
                    WorthQueryRequiredOutputRecoveryPosture::Terminal,
                    WorthQueryRequiredOutputRecoveryPosture::Terminal
                )
            );
        }
    }
}
