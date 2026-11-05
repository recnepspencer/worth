//! The door from execution's lease refusals and kernel stops to Query's execution refusals.
//!
//! Every match here is exhaustive and names each cause, so a cause execution
//! adds is a compile error here, never a cause folded into a neighbor.

use worth_execution::{LeaseDenial, MapKernelStop, MemoryLimitDenial, MemoryLimitLevel};

use {
    crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationCheckpointDenial,
    crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationInterruption,
    crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial,
    crate::domain_computation::request_execution::execution_refusal::WorthQueryMemoryLimitLevel,
};

type Resource = WorthQueryManagedComputationResourceDenial;

/// What a lease refusal is to the computation that asked for the lease.
pub(in crate::domain_computation) const fn lease_denial(denial: LeaseDenial) -> Resource {
    match denial {
        LeaseDenial::WorkerLimitExceedsParent => Resource::WorkerLimit,
        LeaseDenial::MemoryLimitExceedsParent => Resource::PolicyMemoryLimit,
        LeaseDenial::WorkLimitExceedsParent => Resource::WorkLimit,
        LeaseDenial::MemoryExhausted(denial) => memory_denial(denial),
        LeaseDenial::ChargedBytesOverflow => Resource::ChargedBytesOverflow,
        LeaseDenial::UnrelatedNestedLease => Resource::NestedLeaseMisuse,
        LeaseDenial::NoActiveExecutionScope => Resource::NoActiveExecutionScope,
        LeaseDenial::EquivalenceContractUnavailable => Resource::EquivalenceContractUnavailable,
    }
}

/// A refused reservation, with the bytes it asked for, the limit that
/// refused and the room it left. A run's refusal and a reservation's have
/// this one spelling.
pub(in crate::domain_computation) const fn memory_denial(denial: MemoryLimitDenial) -> Resource {
    match denial {
        MemoryLimitDenial::CounterOverflow { .. } => Resource::ChargedBytesOverflow,
        MemoryLimitDenial::Capacity {
            requested,
            admitted,
            level,
        } => Resource::MemoryLimit {
            requested,
            admitted,
            level: match level {
                MemoryLimitLevel::Policy { .. } => WorthQueryMemoryLimitLevel::Policy,
                MemoryLimitLevel::Process => WorthQueryMemoryLimitLevel::Process,
                MemoryLimitLevel::Declared => WorthQueryMemoryLimitLevel::Declared,
            },
        },
    }
}

/// What an owner learns at a checkpoint from the kernel stop it met. Each
/// stop is its own cause.
pub(in crate::domain_computation) const fn checkpoint_denial(
    stop: MapKernelStop,
) -> WorthQueryManagedComputationCheckpointDenial {
    match stop {
        MapKernelStop::Cancelled => WorthQueryManagedComputationCheckpointDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled,
        ),
        MapKernelStop::DeadlineElapsed => {
            WorthQueryManagedComputationCheckpointDenial::Interrupted(
                WorthQueryManagedComputationInterruption::DeadlineExceeded,
            )
        }
        MapKernelStop::WorkCeiling => {
            WorthQueryManagedComputationCheckpointDenial::Resource(Resource::WorkExhausted)
        }
        MapKernelStop::WorkCounterOverflow => {
            WorthQueryManagedComputationCheckpointDenial::Resource(Resource::WorkCounterOverflow)
        }
        MapKernelStop::AccountingUnavailable => {
            WorthQueryManagedComputationCheckpointDenial::Resource(Resource::AccountingUnavailable)
        }
        MapKernelStop::NestedStopped => {
            WorthQueryManagedComputationCheckpointDenial::NestedPatternStopped
        }
    }
}

/// The kernel stop an interruption is, for a safe point outside any kernel.
pub(in crate::domain_computation) const fn interruption_stop(
    interruption: WorthQueryManagedComputationInterruption,
) -> MapKernelStop {
    match interruption {
        WorthQueryManagedComputationInterruption::Cancelled => MapKernelStop::Cancelled,
        WorthQueryManagedComputationInterruption::DeadlineExceeded => {
            MapKernelStop::DeadlineElapsed
        }
    }
}
