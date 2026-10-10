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
                _ => allocation_control(failure.allocation_denial()),
            },
            Some(Cause::ProviderSession(failure)) => {
                use crate::domain_computation::WorthQueryProviderSessionDenialKind as Session;
                let kind = crate::domain_computation::primary_graph::provider::relational_execution_denial::native_preparation_kind(failure).unwrap_or(failure.kind());
                match kind {
                    kind @ (Session::ExecutionResource { .. }
                    | Session::ExecutionNestedPatternStopped { .. }
                    | Session::ExecutionWorkerPanicked { .. }
                    | Session::ExecutionUncheckedCustomKernel { .. }
                    | Session::ExecutionIdentitiesNotCanonical { .. }) => Some(Ok(kind)),
                    _ => allocation_control(failure.allocation_denial()),
                }
            }
            Some(Cause::SourceRebase(_))
            | Some(Cause::DecisionReadSet(_))
            | Some(Cause::RequestAuthority(_))
            | None => None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn provider_execution_denied(
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
        _ => Kind::ProviderRejected,
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
    use worth_execution::ExecutionAllocationDenialKind as Kind;
    match denial?.kind() {
        Kind::Cancelled => Some(Err(Control::Cancelled)),
        Kind::DeadlineElapsed => Some(Err(Control::TimedOut)),
        Kind::Layout
        | Kind::Lease(_)
        | Kind::Allocator
        | Kind::CapacityMismatch
        | Kind::WriteBeyondReserved
        | Kind::IncompleteSeal => None,
    }
}
