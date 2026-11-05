//! The Query projection of Bridge's existing request execution carrier.
use worth_runtime_bridge::facade::BridgeExecutionDenial as Bridge;
use {crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationInterruption as Interruption, crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial as Resource, crate::domain_computation::request_execution::WorthQueryAdvancementDenial as Denial};

/// A carried Bridge denial is in flight; interruptions belong to the demand,
/// whereas resource and custody refusals keep their typed request carrier.
pub(super) fn query_cause(cause: Bridge) -> Result<Denial, Interruption> {
    Ok(match cause {
        Bridge::WorkerLimitExceedsParent => Denial::Resource(Resource::WorkerLimit),
        Bridge::MemoryLimitExceedsParent => Denial::Resource(Resource::PolicyMemoryLimit),
        Bridge::WorkLimitExceedsParent => Denial::Resource(Resource::WorkLimit),
        Bridge::MemoryExhausted(cause) => Denial::Resource(
            crate::domain_computation::request_execution::execution_stop::memory_denial(cause),
        ),
        Bridge::ChargedBytesOverflow => Denial::Resource(Resource::ChargedBytesOverflow),
        Bridge::UnrelatedNestedLease => Denial::Resource(Resource::NestedLeaseMisuse),
        Bridge::NoActiveExecutionScope => Denial::Resource(Resource::NoActiveExecutionScope),
        Bridge::EquivalenceContractUnavailable => Denial::Resource(Resource::EquivalenceContractUnavailable),
        Bridge::Cancelled => return Err(Interruption::Cancelled),
        Bridge::DeadlineElapsed => return Err(Interruption::DeadlineExceeded),
        Bridge::WorkCounterOverflow => Denial::Resource(Resource::WorkCounterOverflow),
        Bridge::AccountingUnavailable => Denial::Resource(Resource::AccountingUnavailable),
        Bridge::WorkCeiling => Denial::Resource(Resource::WorkExhausted),
        Bridge::NestedStopped => Denial::NestedStopped,
        Bridge::Panicked => Denial::Panicked,
    })
}
