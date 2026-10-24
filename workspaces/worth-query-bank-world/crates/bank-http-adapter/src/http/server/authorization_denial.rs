//! One instruction per authorization cause, shared by every route.
//!
//! A query, a mutation request, an estate operation and a recovery all refuse
//! with the same authorization kinds, so each kind names the one action that
//! can succeed wherever it arises. A lapse of the request itself is cleared by
//! repeating it or signing in again, never by a permission change.

use bank_server::BankAuthorizationDenialKind;

use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next,
};

pub(super) fn authorization_denial(kind: BankAuthorizationDenialKind) -> BankHttpDenial {
    use BankAuthorizationDenialKind as Authorization;
    let (denial, next) = match kind {
        Authorization::RelationalExecution(cause) => {
            return super::relational_execution_denial::denial(cause)
        }
        // The request lapsed before authorization decided.
        Authorization::ExecutionRequest(cause) => return execution_request_denial(cause),
        Authorization::Cancelled => (Kind::Cancelled, Next::Retry),
        Authorization::DeadlineExceeded => (Kind::DeadlineExceeded, Next::Retry),
        Authorization::ExpiredAuthentication => (Kind::Unauthenticated, Next::Authenticate),
        // Installed meaning, principal, scope or grants moved since the view
        // the request was built from.
        Authorization::StaleInstalledSchema
        | Authorization::StaleInstalledOperation
        | Authorization::StalePrincipal
        | Authorization::StaleScope
        | Authorization::StaleAuthorization
        | Authorization::DelegationLineageChanged => (Kind::Stale, Next::Refresh),
        // The decision itself is too large for its installed bound.
        Authorization::CanonicalWorkDenied | Authorization::GrantSelectionLimitExceeded => {
            (Kind::ResourceExhausted, Next::NarrowRequest)
        }
        // Capacity in use now frees as other work settles.
        Authorization::ActiveSnapshotCapacityExhausted { .. }
        | Authorization::RetentionCapacityExhausted => (Kind::Unavailable, Next::Retry),
        // Identity space never frees without reconfiguration.
        Authorization::SnapshotIdentityExhausted
        | Authorization::RetentionIdentityExhausted
        | Authorization::AdmissionIdentityExhausted => (Kind::Unavailable, Next::ContactOperator),
        Authorization::InvalidOperationInput => (Kind::MalformedRequest, Next::CorrectRequest),
        Authorization::ForeignRuntime
        | Authorization::MutationPreconditionRejected
        | Authorization::CapabilityGrantMissing
        | Authorization::CapabilityAuthorizationMissing
        | Authorization::PurposeMismatch
        | Authorization::ExplicitDenyRuleMatched
        | Authorization::ConflictRuleMatched
        | Authorization::SeparationOfDutyRuleMatched
        | Authorization::DistinctActorRuleMatched
        | Authorization::WorkflowActorContextRequired
        | Authorization::WorkflowActorAbsent
        | Authorization::WorkflowActorComparisonMismatch
        | Authorization::CapabilityRequired
        | Authorization::CapabilityNotRequired
        | Authorization::CapabilityExpired
        | Authorization::ElevationRequired
        | Authorization::ElevationNotApplicable
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
        | Authorization::ScopeMismatch
        | Authorization::ProductSecurityBasis(_)
        | Authorization::PermissionDenied => (Kind::PermissionDenied, Next::None),
        Authorization::WorkflowActorEvidenceUnavailable
        | Authorization::TrustedTimeUnavailable
        | Authorization::GraphWorkAdmissionUnavailable => (Kind::Unavailable, Next::Retry),
        // Repeating the request cannot install or repair its policy.
        Authorization::PolicyNotInstalled => (Kind::Unavailable, Next::ContactOperator),
        Authorization::ForeignPhase
        | Authorization::CapabilityProjectionRejected
        | Authorization::ElevationProjectionRejected
        | Authorization::InvalidInstalledPolicy
        | Authorization::RelationalObservationRejected
        | Authorization::BridgeEvaluationRejected
        | Authorization::WorkflowActorSourceInvalid
        | Authorization::InconsistentDecision => (Kind::InternalDenied, Next::ContactOperator),
    };
    BankHttpDenial::new(denial, next)
}

// Typed Query request causes retain the authorization read's HTTP rule.
fn execution_request_denial(
    kind: worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind,
) -> BankHttpDenial {
    use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind as Session;
    match kind {
        Session::ExecutionResource {
            denial:
                Resource::WorkExhausted
                | Resource::MemoryLimit { .. }
                | Resource::WorkerLimit
                | Resource::PolicyMemoryLimit
                | Resource::WorkLimit
                | Resource::ResultCapacityExceeded
                | Resource::ScratchCapacityExceeded,
            ..
        } => BankHttpDenial::new(Kind::ResourceExhausted, Next::NarrowRequest),
        kind @ (Session::ExecutionResource {
            denial:
                Resource::WorkCounterOverflow
                | Resource::AccountingUnavailable
                | Resource::RetainedBytesExhausted
                | Resource::CapacityOverflow
                | Resource::NativeAllocationRefused { .. }
                | Resource::ChargedBytesOverflow
                | Resource::NestedLeaseMisuse
                | Resource::NestedAdvancementOpening
                | Resource::ForeignAdvancementPhase
                | Resource::NoActiveExecutionScope
                | Resource::EquivalenceContractUnavailable,
            ..
        }
        | Session::ExecutionWorkerPanicked { .. }
        | Session::ExecutionUncheckedCustomKernel { .. }
        | Session::ExecutionIdentitiesNotCanonical { .. }
        | Session::ExecutionNestedPatternStopped { .. }
        | Session::RetentionIdentityExhausted
        | Session::SnapshotIdentityExhausted
        | Session::CandidateIdentityExhausted
        | Session::IndexGenerationIdentityExhausted
        | Session::ForeignOperationAttempt
        | Session::ForeignExecutionBasis
        | Session::ForeignGraphAuthority
        | Session::UndeclaredOperationScope
        | Session::ResourceEnvelopeMismatch
        | Session::ProviderIdentityMismatch
        | Session::ProviderGenerationMismatch
        | Session::SessionProtocolUnsupported
        | Session::ProviderPanicked
        | Session::TokenNotMintedForPlan
        | Session::EmptyPhysicalSessionIdentity
        | Session::SessionIdentityExhausted
        | Session::ActiveSnapshotCapacityExhausted { .. }
        | Session::RetentionCapacityExhausted
        | Session::IndexMaintenanceBudgetExceeded
        | Session::AllocationDenied
        | Session::ProviderRejected) => super::mutation_application::pending_execution_denial(kind),
    }
}

#[cfg(test)]
#[path = "authorization_denial/instruction_tests.rs"]
mod instruction_tests;
