//! The Query projection of Bridge's existing request execution carrier.
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryAdvancementDenial as Denial,
    WorthQueryManagedComputationInterruption as Interruption,
    WorthQueryManagedComputationResourceDenial as Resource,
};
use worth_runtime_bridge::facade::BridgeExecutionDenial as Bridge;

pub(super) fn query_cause(cause: Bridge) -> Denial {
    match cause {
        Bridge::WorkerLimitExceedsParent => Denial::Resource(Resource::WorkerLimit),
        Bridge::MemoryLimitExceedsParent => Denial::Resource(Resource::PolicyMemoryLimit),
        Bridge::WorkLimitExceedsParent => Denial::Resource(Resource::WorkLimit),
        Bridge::MemoryExhausted(cause) => Denial::Resource(
            crate::domain_computation::primary_graph::application_contribution::execution_denial::memory_denial(cause),
        ),
        Bridge::ChargedBytesOverflow => Denial::Resource(Resource::ChargedBytesOverflow),
        Bridge::UnrelatedNestedLease => Denial::Resource(Resource::NestedLeaseMisuse),
        Bridge::NoActiveExecutionScope => Denial::Resource(Resource::NoActiveExecutionScope),
        Bridge::EquivalenceContractUnavailable => Denial::Resource(Resource::EquivalenceContractUnavailable),
        Bridge::Cancelled => Denial::Interrupted(Interruption::Cancelled),
        Bridge::DeadlineElapsed => Denial::Interrupted(Interruption::DeadlineExceeded),
        Bridge::WorkCounterOverflow => Denial::Resource(Resource::WorkCounterOverflow),
        Bridge::WorkCeiling => Denial::Resource(Resource::WorkExhausted),
        Bridge::NestedStopped => Denial::NestedStopped,
        Bridge::Panicked => Denial::Panicked,
    }
}
