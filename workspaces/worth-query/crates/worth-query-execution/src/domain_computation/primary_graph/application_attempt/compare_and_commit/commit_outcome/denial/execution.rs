//! Typed execution refusals before application publication.
use super::{
    WorthQueryApplicationCommitDenial as Denial, WorthQueryApplicationCommitDenialKind as Kind,
    WorthQueryApplicationCommitDenialStage as Stage,
};
use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;

impl Denial {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_resource(
        denial: Resource,
        partition_identity: Option<u64>,
        policy_ancestor: Option<u32>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::provider_execution_denied(
            Stage::ProviderCommit,
            Ok(
                crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial,
                    partition_identity,
                    policy_ancestor,
                },
            ),
            detail,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_nested_stopped(
        partition_identity: Option<u64>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::provider_execution_denied(
            Stage::ProviderCommit,
            Ok(crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionNestedPatternStopped {
                partition_identity,
            }),
            detail,
        )
    }
}

impl Denial {
    /// The original execution cause at the stage that refused publication.
    pub fn execution_denial_cause(
        &self,
    ) -> Option<
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    > {
        use super::denial_cause::DenialCause as Cause;
        use crate::domain_computation::WorthQueryInvariantExecutionDenialKind as Invariant;
        match self.cause.as_deref() {
            Some(Cause::Execution(cause)) => Some(*cause),
            Some(Cause::InvariantExecution(failure)) => match failure.kind() {
                Invariant::ExecutionDenied(kind) => Some(Ok(kind)),
                Invariant::ExecutionControlStopped(kind) => Some(Err(kind)),
                Invariant::RequestInterrupted(stop) => Some(Err(control_kind(stop))),
                Invariant::AllocationDenied
                | Invariant::InvariantNotInstalled
                | Invariant::ExecutorRoleMismatch
                | Invariant::UndeclaredStateLoadFamily
                | Invariant::StateLoadBudgetExceeded
                | Invariant::ExecutionBudgetExceeded
                | Invariant::ProviderUnsupported
                | Invariant::ProviderRejected
                | Invariant::RelationalDeferred(..)
                | Invariant::CustomInvariantDenied
                | Invariant::ProductBasisStale
                | Invariant::RetentionCapacityExhausted
                | Invariant::RetentionIdentityExhausted
                | Invariant::SnapshotIdentityExhausted
                | Invariant::SavepointCapacityExhausted { .. }
                | Invariant::SavepointIdentityExhausted
                | Invariant::CandidateCapacityExhausted { .. }
                | Invariant::PublishedSnapshotCapacityExhausted { .. }
                | Invariant::CandidateIdentityExhausted
                | Invariant::PatchPositionReservationContended
                | Invariant::ProposalIdentityExhausted
                | Invariant::ProviderPanicked
                | Invariant::EvidenceSubstitution
                | Invariant::EmptyStateLoad
                | Invariant::StateLoadClosureMismatch
                | Invariant::VerdictPostureMismatch => {
                    allocation_control(failure.allocation_denial())
                }
            },
            Some(Cause::ProviderSession(failure)) => {
                use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;
                let kind = match crate::domain_computation::primary_graph::provider::relational_execution_denial::native_preparation_kind(failure) {
                    Ok(kind) => kind.unwrap_or(failure.kind()),
                    Err(stop) => return Some(Err(stop)),
                };
                match kind {
                    kind @ (Session::ExecutionResource { .. }
                    | Session::ExecutionNestedPatternStopped { .. }
                    | Session::ExecutionWorkerPanicked { .. }
                    | Session::ExecutionUncheckedCustomKernel { .. }
                    | Session::ExecutionIdentitiesNotCanonical { .. }) => Some(Ok(kind)),
                    Session::ForeignOperationAttempt
                    | Session::ForeignExecutionBasis
                    | Session::ForeignGraphAuthority
                    | Session::UndeclaredOperationScope
                    | Session::ResourceEnvelopeMismatch
                    | Session::ActiveSnapshotCapacityExhausted { .. }
                    | Session::RetentionCapacityExhausted
                    | Session::RetentionIdentityExhausted
                    | Session::SnapshotIdentityExhausted
                    | Session::CandidateIdentityExhausted
                    | Session::IndexMaintenanceBudgetExceeded
                    | Session::IndexGenerationIdentityExhausted
                    | Session::ProviderIdentityMismatch
                    | Session::ProviderGenerationMismatch
                    | Session::SessionProtocolUnsupported
                    | Session::AllocationDenied
                    | Session::ProviderRejected
                    | Session::ProviderPanicked
                    | Session::TokenNotMintedForPlan
                    | Session::EmptyPhysicalSessionIdentity
                    | Session::SessionIdentityExhausted => {
                        allocation_control(failure.allocation_denial())
                    }
                }
            }
            Some(Cause::DecisionReadSet(failure)) => {
                use crate::domain_computation::WorthQueryDecisionReadSetDenialKind as ReadSet;
                use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
                use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Interruption;
                match failure.kind() {
                    ReadSet::ExecutionRequest(kind) => Some(Ok(kind)),
                    ReadSet::RequestInterrupted(Interruption::Cancelled) => {
                        Some(Err(Control::Cancelled))
                    }
                    ReadSet::RequestInterrupted(Interruption::DeadlineExceeded) => {
                        Some(Err(Control::TimedOut))
                    }
                    ReadSet::InvalidRequest
                    | ReadSet::UndeclaredFamily
                    | ReadSet::FamilyKindMismatch
                    | ReadSet::DecisionFactsUnsupported
                    | ReadSet::InvalidProviderEvidence
                    | ReadSet::EvidenceSubstitution
                    | ReadSet::IncompleteRequiredFamilies
                    | ReadSet::IncompleteRequiredFacts
                    | ReadSet::DecisionFactBudgetExceeded
                    | ReadSet::FactCountOverflow
                    | ReadSet::AllocationDenied
                    | ReadSet::ActiveSnapshotCapacityExhausted { .. }
                    | ReadSet::RetentionCapacityExhausted
                    | ReadSet::RetentionIdentityExhausted
                    | ReadSet::SnapshotIdentityExhausted
                    | ReadSet::ProviderRejected
                    | ReadSet::ProviderPanicked => allocation_control(failure.allocation_denial()),
                }
            }
            Some(Cause::RequestAuthority(kind)) => authority_execution(*kind),
            Some(Cause::SigningLifecycleAttempt(failure)) => {
                if let crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ExecutionRequest(kind) = failure.kind() {
                    Some(Ok(kind))
                } else { allocation_control(failure.allocation_denial()) }
            }
            Some(Cause::SourceRebase(_)) | None => None,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn provider_execution_denied(
        stage: Stage,
        cause: Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self {
            kind: execution_kind(cause),
            stage,
            detail: Some(detail.into()),
            cause: Some(Box::new(super::denial_cause::DenialCause::Execution(cause))),
        }
    }
}

/// Both projections of a preparation refusal carry the same execution category.
pub(super) fn execution_kind(
    cause: Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    >,
) -> Kind {
    use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;
    match cause {
        Ok(Session::ExecutionResource {
            denial,
            partition_identity,
            policy_ancestor,
        }) => Kind::ExecutionResource {
            denial,
            partition_identity,
            policy_ancestor,
        },
        Ok(Session::ExecutionNestedPatternStopped { partition_identity }) => {
            Kind::ExecutionNestedPatternStopped { partition_identity }
        }
        Ok(Session::ExecutionWorkerPanicked { partition_identity }) => {
            Kind::ExecutionWorkerPanicked { partition_identity }
        }
        Ok(Session::ExecutionUncheckedCustomKernel { partition_identity }) => {
            Kind::ExecutionUncheckedCustomKernel { partition_identity }
        }
        Ok(Session::ExecutionIdentitiesNotCanonical { partition_identity }) => {
            Kind::ExecutionIdentitiesNotCanonical { partition_identity }
        }
        Ok(Session::ForeignOperationAttempt)
        | Ok(Session::ForeignExecutionBasis)
        | Ok(Session::ForeignGraphAuthority)
        | Ok(Session::UndeclaredOperationScope)
        | Ok(Session::ResourceEnvelopeMismatch)
        | Ok(Session::ActiveSnapshotCapacityExhausted { .. })
        | Ok(Session::RetentionCapacityExhausted)
        | Ok(Session::RetentionIdentityExhausted)
        | Ok(Session::SnapshotIdentityExhausted)
        | Ok(Session::CandidateIdentityExhausted)
        | Ok(Session::IndexMaintenanceBudgetExceeded)
        | Ok(Session::IndexGenerationIdentityExhausted)
        | Ok(Session::ProviderIdentityMismatch)
        | Ok(Session::ProviderGenerationMismatch)
        | Ok(Session::SessionProtocolUnsupported)
        | Ok(Session::AllocationDenied)
        | Ok(Session::ProviderRejected)
        | Ok(Session::ProviderPanicked)
        | Ok(Session::TokenNotMintedForPlan)
        | Ok(Session::EmptyPhysicalSessionIdentity)
        | Ok(Session::SessionIdentityExhausted)
        | Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled
            | crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
        ) => Kind::ProviderRejected,
    }
}

fn control_kind(
    stop: worth_relational::facade::mvcc::RelationalOperationInterruption,
) -> crate::domain_computation::WorthQueryProviderSessionControlStopKind {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    match stop {
        worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => {
            Control::Cancelled
        }
        worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => {
            Control::TimedOut
        }
    }
}
fn allocation_control(
    denial: Option<&worth_execution::ExecutionAllocationDenial>,
) -> Option<
    Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    >,
> {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    use worth_execution::{
        ExecutionAllocationDenialClass as Class, MapKernelStop as Stop,
        WorkCeilingDenial as Request,
    };
    match denial?.classify() {
        Class::Request(Request::Stopped(Stop::Cancelled)) => Some(Err(Control::Cancelled)),
        Class::Request(Request::Stopped(Stop::DeadlineElapsed)) => Some(Err(Control::TimedOut)),
        Class::Request(Request::Admission(_))
        | Class::Request(Request::Stopped(
            Stop::WorkCounterOverflow | Stop::AccountingUnavailable | Stop::WorkCeiling | Stop::NestedStopped,
        ))
        | Class::Request(Request::Panicked)
        | Class::Physical(_) => None,
    }
}

mod authority_cause;
use authority_cause::authority_execution;
