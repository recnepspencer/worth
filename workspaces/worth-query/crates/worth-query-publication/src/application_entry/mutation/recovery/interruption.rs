//! Recovery request stops use the same outcome as opening interruption.
use crate::application_entry::WorthQueryApplicationRequestMutationDenial as Request;
use worth_query_execution::facade::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
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
        Request::SignedCreation(worth_query_execution::facade::primary_graph::WorthQuerySignedCreationPreparationDenial::Execution(cause)) => Some(*cause),
        Request::SignedCreation(worth_query_execution::facade::primary_graph::WorthQuerySignedCreationPreparationDenial::LifecycleObservation(outcome)) => {
            use worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitOutcome as Outcome;
            match outcome.as_ref() {
                Outcome::Cancelled => Some(Denial::Interrupted(Interruption::Cancelled)),
                Outcome::TimedOut => Some(Denial::Interrupted(Interruption::DeadlineExceeded)),
                Outcome::Denied(cause) => cause.execution_denial_cause().and_then(execution_stop),
                _ => None,
            }
        }
        Request::ExecutionRequest(cause) => Some(*cause),
        Request::InputCapture(cause) => cause.execution_stop(),
        Request::Authorization(cause) => authorization_interruption(cause.kind()),
        Request::PrincipalResolution(cause) => match cause.kind() {
            Principal::Stopped(stop) => execution_stop(stop),
            Principal::Refused(PrincipalRefusal::ForeignPhase)
            | Principal::Refused(PrincipalRefusal::PrimaryGraphNotInstalled)
            | Principal::Refused(PrincipalRefusal::BindingNotInstalled)
            | Principal::Refused(PrincipalRefusal::ForeignRuntime)
            | Principal::Refused(PrincipalRefusal::StaleInstalledSchema)
            | Principal::Refused(PrincipalRefusal::ExpiredAuthentication)
            | Principal::Refused(PrincipalRefusal::BranchMaterializationSuspended)
            | Principal::Refused(PrincipalRefusal::IdentityIndexUnavailable)
            | Principal::Refused(PrincipalRefusal::CorruptIdentityIndex)
            | Principal::Refused(PrincipalRefusal::UnknownPrincipal)
            | Principal::Refused(PrincipalRefusal::DisabledPrincipal)
            | Principal::Refused(PrincipalRefusal::AmbiguousPrincipal)
            | Principal::Refused(PrincipalRefusal::MissingPrincipalTarget)
            | Principal::Refused(PrincipalRefusal::AmbiguousPrincipalTarget)
            | Principal::Refused(PrincipalRefusal::WrongPrincipalTargetKind)
            | Principal::Refused(PrincipalRefusal::StalePrincipalProof)
            | Principal::Refused(PrincipalRefusal::ActiveSnapshotCapacityExhausted { .. })
            | Principal::Refused(PrincipalRefusal::SnapshotIdentityExhausted)
            | Principal::Refused(PrincipalRefusal::RetentionCapacityExhausted)
            | Principal::Refused(PrincipalRefusal::RetentionIdentityExhausted) => None,
        },
        Request::ScopeResolution(cause) => match cause.kind() {
            Scope::Stopped(cause) => execution_stop(
                worth_query_execution::facade::primary_graph::relational_execution_kind(
                    cause, None,
                ),
            ),
            Scope::Refused(_) => None,
        },
        Request::ProductSelection(Product::Stopped(cause)) => execution_stop(*cause),
        Request::ProductSelection(_) => None,
        Request::BindingInstallation(..)
        | Request::CapabilityInstallation(..)
        | Request::Idempotency(..)
        | Request::Identity(..)
        | Request::Handler(..)
        | Request::SourceExpectation(..)
        | Request::ProgramSelection(..)
        | Request::ApplicationProgramRequired
        | Request::ApplicationProgramMismatch
        | Request::RequiresWorkflowTransition
        | Request::WorkflowAuthoritySpent
        | Request::WorkflowCapturedInput(..)
        | Request::PreparationSpent
        | Request::WorkflowTransitionCurrentness(..)
        | Request::SignedCreation(..) => None,
    }
}

pub(super) fn authorization_interruption(cause: Authorization) -> Option<Denial> {
    match cause {
        Authorization::Stopped(stop) => execution_stop(stop),
        Authorization::Refused(_) => None,
    }
}

fn execution_stop(cause: Result<Execution, Control>) -> Option<Denial> {
    match cause {
        Err(Control::Cancelled) => Some(Denial::Interrupted(Interruption::Cancelled)),
        Err(Control::TimedOut) => Some(Denial::Interrupted(Interruption::DeadlineExceeded)),
        Ok(Execution::ExecutionResource { denial, .. }) => Some(Denial::Resource(denial)),
        Ok(Execution::ExecutionNestedPatternStopped { .. }) => Some(Denial::NestedStopped),
        Ok(Execution::ExecutionWorkerPanicked { .. }) => Some(Denial::Panicked),
        Ok(Execution::StorageUnavailable(_))
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

#[cfg(test)]
mod tests {
    use super::{request_stop, Denial, Interruption, Request};
    use worth_query_execution::facade::primary_graph::{
        WorthQueryApplicationCommitOutcome as Outcome,
        WorthQuerySignedCreationPreparationDenial as Signing,
    };
    #[test]
    fn signing_lifecycle_observation_does_not_hide_native_request_stops() {
        for (outcome, expected) in [
            (Outcome::Cancelled, Interruption::Cancelled),
            (Outcome::TimedOut, Interruption::DeadlineExceeded),
        ] {
            let denial = Request::SignedCreation(Signing::LifecycleObservation(Box::new(outcome)));
            assert!(
                matches!(request_stop(&denial), Some(Denial::Interrupted(actual)) if actual == expected)
            );
        }
    }
}
