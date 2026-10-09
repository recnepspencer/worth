//! Typed execution refusals before application publication.
use super::{
    WorthQueryApplicationCommitDenial as Denial, WorthQueryApplicationCommitDenialKind as Kind,
    WorthQueryApplicationCommitDenialStage as Stage,
};
use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;

impl Denial {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn with_provider_execution_cause(
        mut self,
        kind: crate::domain_computation::WorthQueryProviderSessionDenialKind,
    ) -> Self {
        self.cause = Some(Box::new(super::denial_cause::DenialCause::Execution(Ok(
            kind,
        ))));
        self
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_resource(
        denial: Resource,
        partition_identity: Option<u64>,
        policy_ancestor: Option<u32>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::execution_denial(
            Kind::ExecutionResource {
                denial,
                partition_identity,
                policy_ancestor,
            },
            detail,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_nested_stopped(
        partition_identity: Option<u64>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::execution_denial(
            Kind::ExecutionNestedPatternStopped { partition_identity },
            detail,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_worker_panicked(
        partition_identity: Option<u64>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::execution_denial(Kind::ExecutionWorkerPanicked { partition_identity }, detail)
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_unchecked_custom_kernel(
        partition_identity: Option<u64>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::execution_denial(
            Kind::ExecutionUncheckedCustomKernel { partition_identity },
            detail,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_attempt) fn execution_identities_not_canonical(
        partition_identity: Option<u64>,
        detail: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Self::execution_denial(
            Kind::ExecutionIdentitiesNotCanonical { partition_identity },
            detail,
        )
    }
    fn execution_denial(kind: Kind, detail: impl Into<std::sync::Arc<str>>) -> Self {
        Self {
            kind,
            stage: Stage::ProviderCommit,
            detail: Some(detail.into()),
            cause: None,
        }
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
                match failure.kind() {
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
            kind: Kind::ProviderRejected,
            stage,
            detail: Some(detail.into()),
            cause: Some(Box::new(super::denial_cause::DenialCause::Execution(cause))),
        }
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
