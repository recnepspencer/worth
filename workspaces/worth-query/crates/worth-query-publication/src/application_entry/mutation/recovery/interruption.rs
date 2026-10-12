//! Recovery request stops use the same outcome as opening interruption.
use crate::application_entry::WorthQueryApplicationRequestMutationDenial as Request;
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationInterruption as Interruption,
    },
    primary_graph::{
        WorthQueryEntityResolutionDenialKind as Scope,
        WorthQueryOperationAuthorizationDenialKind as Authorization,
        WorthQueryPrincipalResolutionDenialKind as Principal,
        WorthQueryProviderSessionControlStopKind as Control,
        WorthQueryProviderSessionDenialKind as Execution,
    },
    product::WorthQueryProductBranchAdmissionDenial as Product,
};
pub(super) fn request_stop(denial: &Request) -> Option<Denial> {
    match denial {
  Request::ExecutionRequest(cause) => Some(*cause),
  Request::Authorization(cause) => authorization_interruption(cause.kind()),
  Request::PrincipalResolution(cause) => match cause.kind() {
   Principal::Cancelled => Some(Denial::Interrupted(Interruption::Cancelled)),
   Principal::DeadlineExceeded => Some(Denial::Interrupted(Interruption::DeadlineExceeded)),
   Principal::ExecutionRequest(kind) => execution_stop(Ok(kind)),
   Principal::ProjectionWorkBudgetExceeded => Some(Denial::Resource(worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted)),
   Principal::ProjectionPreparationMemoryExhausted => Some(Denial::Resource(worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::PolicyMemoryLimit)),
   Principal::ForeignPhase | Principal::PrimaryGraphNotInstalled | Principal::BindingNotInstalled | Principal::ForeignRuntime | Principal::StaleInstalledSchema | Principal::ExpiredAuthentication | Principal::BranchMaterializationSuspended | Principal::IdentityIndexUnavailable | Principal::CorruptIdentityIndex | Principal::UnknownPrincipal | Principal::DisabledPrincipal | Principal::AmbiguousPrincipal | Principal::MissingPrincipalTarget | Principal::AmbiguousPrincipalTarget | Principal::WrongPrincipalTargetKind | Principal::StalePrincipalProof | Principal::ActiveSnapshotCapacityExhausted{ .. } | Principal::SnapshotIdentityExhausted | Principal::RetentionCapacityExhausted | Principal::RetentionIdentityExhausted => None,
  },
  Request::ScopeResolution(cause) => match cause.kind() {
   Scope::Stopped(cause) => execution_stop(worth_query_execution::facade::primary_graph::relational_execution_kind(cause, None)), Scope::Refused(_) => None,
  },
  Request::ProductSelection(Product::Stopped(cause)) => execution_stop(*cause), Request::ProductSelection(Product::Refused(_)) => None,
  Request::BindingInstallation(..) | Request::CapabilityInstallation(..) | Request::Idempotency(..) | Request::Identity(..) | Request::Handler(..) | Request::SourceExpectation(..) | Request::ProgramSelection(..) | Request::ApplicationProgramRequired | Request::ApplicationProgramMismatch | Request::RequiresWorkflowTransition | Request::WorkflowAuthoritySpent | Request::PreparationSpent | Request::WorkflowTransitionCurrentness(..) => None,
 }
}
pub(super) fn authorization_interruption(cause: Authorization) -> Option<Denial> {
    match cause {
        Authorization::Cancelled => Some(Denial::Interrupted(Interruption::Cancelled)),
        Authorization::DeadlineExceeded => {
            Some(Denial::Interrupted(Interruption::DeadlineExceeded))
        }
        Authorization::ExecutionRequest(kind) => execution_stop(Ok(kind)),
        Authorization::ProductSecurityBasis(Product::Stopped(cause)) => execution_stop(cause),
        Authorization::ProductSecurityBasis(Product::Refused(_)) => None,
        Authorization::ForeignPhase
        | Authorization::ExpiredAuthentication
        | Authorization::ForeignRuntime
        | Authorization::StaleInstalledSchema
        | Authorization::StaleInstalledOperation
        | Authorization::StalePrincipal
        | Authorization::StaleScope
        | Authorization::MutationPreconditionRejected
        | Authorization::CanonicalWorkDenied
        | Authorization::TrustedTimeUnavailable
        | Authorization::InvalidOperationInput
        | Authorization::CapabilityProjectionRejected
        | Authorization::CapabilityGrantMissing
        | Authorization::CapabilityAuthorizationMissing
        | Authorization::PurposeMismatch
        | Authorization::ExplicitDenyRuleMatched
        | Authorization::ConflictRuleMatched
        | Authorization::SeparationOfDutyRuleMatched
        | Authorization::DistinctActorRuleMatched
        | Authorization::CapabilityRequired
        | Authorization::CapabilityNotRequired
        | Authorization::CapabilityExpired
        | Authorization::ElevationRequired
        | Authorization::ElevationNotApplicable
        | Authorization::ElevationProjectionRejected
        | Authorization::ElevationExpired
        | Authorization::ElevationInactive
        | Authorization::ElevationSelfApproval
        | Authorization::ElevationApproverConflict
        | Authorization::ElevationTransitionRequired
        | Authorization::ElevationLifecycleRoleMismatch
        | Authorization::ElevationRequestRejected
        | Authorization::ElevationApprovalRejected
        | Authorization::ElevationCloseRejected
        | Authorization::MandatoryReviewRejected
        | Authorization::ElevationDurationExceeded
        | Authorization::DelegationRejected
        | Authorization::DelegationTransitionRequired
        | Authorization::DelegationDepthExceeded
        | Authorization::DelegationCycle
        | Authorization::DelegationLineageChanged
        | Authorization::StaleAuthorization
        | Authorization::AdmissionIdentityExhausted
        | Authorization::ActiveSnapshotCapacityExhausted { .. }
        | Authorization::SnapshotIdentityExhausted
        | Authorization::RetentionCapacityExhausted
        | Authorization::RetentionIdentityExhausted
        | Authorization::GraphWorkAdmissionUnavailable
        | Authorization::ScopeMismatch
        | Authorization::PolicyNotInstalled
        | Authorization::InvalidInstalledPolicy
        | Authorization::RelationalObservationRejected
        | Authorization::GrantSelectionLimitExceeded
        | Authorization::BridgeEvaluationRejected
        | Authorization::InconsistentDecision
        | Authorization::PermissionDenied => None,
    }
}
fn execution_stop(cause: Result<Execution, Control>) -> Option<Denial> {
    match cause {
        Err(Control::Cancelled) => Some(Denial::Interrupted(Interruption::Cancelled)),
        Err(Control::TimedOut) => Some(Denial::Interrupted(Interruption::DeadlineExceeded)),
        Ok(Execution::ExecutionResource { .. })
        | Ok(Execution::ExecutionNestedPatternStopped { .. })
        | Ok(Execution::ExecutionWorkerPanicked { .. })
        | Ok(Execution::ForeignOperationAttempt)
        | Ok(Execution::ForeignExecutionBasis)
        | Ok(Execution::ForeignGraphAuthority)
        | Ok(Execution::UndeclaredOperationScope)
        | Ok(Execution::ResourceEnvelopeMismatch)
        | Ok(Execution::ExecutionIdentitiesNotCanonical { .. })
        | Ok(Execution::ExecutionUncheckedCustomKernel { .. })
        | Ok(Execution::ActiveSnapshotCapacityExhausted { .. })
        | Ok(Execution::RetentionCapacityExhausted)
        | Ok(Execution::RetentionIdentityExhausted)
        | Ok(Execution::SnapshotIdentityExhausted)
        | Ok(Execution::CandidateIdentityExhausted)
        | Ok(Execution::IndexMaintenanceBudgetExceeded)
        | Ok(Execution::IndexGenerationIdentityExhausted)
        | Ok(Execution::ProviderIdentityMismatch)
        | Ok(Execution::ProviderGenerationMismatch)
        | Ok(Execution::SessionProtocolUnsupported)
        | Ok(Execution::AllocationDenied)
        | Ok(Execution::ProviderRejected)
        | Ok(Execution::ProviderPanicked)
        | Ok(Execution::TokenNotMintedForPlan)
        | Ok(Execution::EmptyPhysicalSessionIdentity)
        | Ok(Execution::SessionIdentityExhausted) => None,
    }
}
