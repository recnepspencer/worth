mod preflight;
pub(in crate::domain_computation::primary_graph) use preflight::preflight_execution_cause;
/// The only conversion from Relational execution causes to Query denial kinds.
pub fn relational_execution_kind(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    partition_identity: Option<u64>,
) -> Result<
    crate::domain_computation::WorthQueryProviderSessionDenialKind,
    crate::domain_computation::WorthQueryProviderSessionControlStopKind,
> {
    use crate::domain_computation::primary_graph::application_contribution::{
        WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
    };
    use crate::domain_computation::{
        WorthQueryProviderSessionControlStopKind as Control,
        WorthQueryProviderSessionDenialKind as Kind,
    };
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

    let resource = |denial, policy_ancestor| Kind::ExecutionResource {
        denial,
        partition_identity,
        policy_ancestor,
    };
    let kind = match cause {
        Cause::StorageUnavailable(failure) => Kind::StorageUnavailable(failure),
        Cause::WorkerLimitExceedsParent => resource(Resource::WorkerLimit, None),
        Cause::MemoryLimitExceedsParent => resource(Resource::PolicyMemoryLimit, None),
        Cause::WorkLimitExceedsParent => resource(Resource::WorkLimit, None),
        Cause::PolicyMemoryExhausted {
            requested,
            admitted,
            ancestor,
        } => resource(
            Resource::MemoryLimit {
                requested,
                admitted,
                level: Level::Policy,
            },
            Some(ancestor),
        ),
        Cause::ProcessMemoryExhausted {
            requested,
            admitted,
        } => resource(
            Resource::MemoryLimit {
                requested,
                admitted,
                level: Level::Process,
            },
            None,
        ),
        Cause::DeclaredMemoryExhausted {
            requested,
            admitted,
        } => resource(
            Resource::MemoryLimit {
                requested,
                admitted,
                level: Level::Declared,
            },
            None,
        ),
        Cause::ChargedBytesOverflow => resource(Resource::ChargedBytesOverflow, None),
        Cause::UnrelatedNestedLease => resource(Resource::NestedLeaseMisuse, None),
        Cause::NoActiveExecutionScope => resource(Resource::NoActiveExecutionScope, None),
        Cause::EquivalenceContractUnavailable => {
            resource(Resource::EquivalenceContractUnavailable, None)
        }
        Cause::Cancelled => return Err(Control::Cancelled),
        Cause::DeadlineElapsed => return Err(Control::TimedOut),
        Cause::WorkCounterOverflow => resource(Resource::WorkCounterOverflow, None),
        Cause::AccountingUnavailable => resource(Resource::AccountingUnavailable, None),
        Cause::WorkExhausted => resource(Resource::WorkExhausted, None),
        Cause::NestedStopped => Kind::ExecutionNestedPatternStopped { partition_identity },
        Cause::ResultCapacityExceeded => resource(Resource::ResultCapacityExceeded, None),
        Cause::ScratchCapacityExceeded => resource(Resource::ScratchCapacityExceeded, None),
        Cause::AllocationFailed => Kind::AllocationDenied,
        Cause::WorkerFailed => Kind::ExecutionWorkerPanicked { partition_identity },
        Cause::UncheckedCustomKernel => Kind::ExecutionUncheckedCustomKernel { partition_identity },
        Cause::ExpectedIdentitiesNotCanonical => {
            Kind::ExecutionIdentitiesNotCanonical { partition_identity }
        }
        Cause::MemoryOverflow => resource(Resource::CapacityOverflow, None),
    };
    Ok(kind)
}

/// Preparation executes before publication; a refusal never asks for recovery.
pub(in crate::domain_computation::primary_graph) fn relational_execution_stop(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    partition_identity: Option<u64>,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::{
        WorthQueryProviderSessionCommitControlStopped, WorthQueryProviderSessionCommitStop as Stop,
        WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
    };
    let detail = format!("Relational execution stopped: {cause:?} at {partition_identity:?}");
    match relational_execution_kind(cause, partition_identity) {
        Ok(kind) => Stop::PreEffectDenied(WorthQueryProviderSessionFailure::new(
            kind,
            WorthQueryProviderSessionProtocolStage::Commit,
            detail,
            Default::default(),
        )),
        Err(kind) => Stop::ControlStopped(
            WorthQueryProviderSessionCommitControlStopped::execution_preparation(kind, detail),
        ),
    }
}

/// Native preparation evidence keeps the typed cause at the application boundary.
pub(in crate::domain_computation::primary_graph) fn native_preparation_kind(
    failure: &crate::domain_computation::WorthQueryProviderSessionFailure,
) -> Result<
    Option<crate::domain_computation::WorthQueryProviderSessionDenialKind>,
    crate::domain_computation::WorthQueryProviderSessionControlStopKind,
> {
    if let Some(stop) = failure.checked_read_stop() {
        return stop.map(Some);
    }
    use worth_relational::facade::transactions::{
        CommitExecutionDenialKind, TransactionCommitError,
    };
    let Some(TransactionCommitError::Execution { denial, .. }) = failure.native_preparation_error()
    else {
        return Ok(None);
    };
    let CommitExecutionDenialKind::Cause(cause) = denial.kind;
    relational_execution_kind(cause, denial.partition_identity).map(Some)
}

/// Query reads retain native resource and control causes through this same conversion.
pub(in crate::domain_computation::primary_graph) fn query_read_execution_kind(
    stop: &worth_relational::facade::runtime::QueryReadExecutionStop,
) -> Result<
    Option<crate::domain_computation::WorthQueryProviderSessionDenialKind>,
    crate::domain_computation::WorthQueryProviderSessionControlStopKind,
> {
    let Some(denial) = stop.execution_denial() else {
        return Ok(None);
    };
    let worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause) =
        denial.kind;
    relational_execution_kind(cause, denial.partition_identity).map(Some)
}

#[cfg(test)]
#[path = "relational_execution_denial/tests.rs"]
mod tests;
