//! The one door from execution's stops and refusals to Query's denials.
//!
//! Every match here is exhaustive and names each cause, so a cause execution
//! adds is a compile error here, never a cause folded into a neighbor. No
//! public Query denial carries an execution type.

use worth_execution::{
    LeaseDenial, MapKernelStop, MemoryLimitDenial, MemoryLimitLevel, WorkCeilingDenial,
};

use super::{
    WorthQueryManagedComputationCheckpointDenial, WorthQueryManagedComputationInterruption,
    WorthQueryManagedComputationResourceDenial, WorthQueryMemoryLimitLevel,
};

type Resource = WorthQueryManagedComputationResourceDenial;

/// What a lease refusal is to the computation that asked for the lease.
pub const fn lease_denial(denial: LeaseDenial) -> Resource {
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
pub const fn memory_denial(denial: MemoryLimitDenial) -> Resource {
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
pub const fn checkpoint_denial(
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
pub const fn interruption_stop(
    interruption: WorthQueryManagedComputationInterruption,
) -> MapKernelStop {
    match interruption {
        WorthQueryManagedComputationInterruption::Cancelled => MapKernelStop::Cancelled,
        WorthQueryManagedComputationInterruption::DeadlineExceeded => {
            MapKernelStop::DeadlineElapsed
        }
    }
}

/// Native lifecycle owners keep the original admitted request's exact stop cause.
pub fn advancement_scope_stop(
    denial: WorkCeilingDenial,
) -> super::WorthQueryAdvancementDenial {
    match denial {
        WorkCeilingDenial::Admission(denial) => {
            super::WorthQueryAdvancementDenial::Resource(lease_denial(denial))
        }
        WorkCeilingDenial::Stopped(stop) => advancement_checkpoint_stop(stop),
        WorkCeilingDenial::Panicked => super::WorthQueryAdvancementDenial::Panicked,
    }
}

pub fn advancement_checkpoint_stop(
    stop: MapKernelStop,
) -> super::WorthQueryAdvancementDenial {
    match checkpoint_denial(stop) {
        WorthQueryManagedComputationCheckpointDenial::Resource(resource) => {
            super::WorthQueryAdvancementDenial::Resource(resource)
        }
        WorthQueryManagedComputationCheckpointDenial::Interrupted(interruption) => {
            super::WorthQueryAdvancementDenial::Interrupted(interruption)
        }
        WorthQueryManagedComputationCheckpointDenial::NestedPatternStopped => {
            super::WorthQueryAdvancementDenial::NestedStopped
        }
    }
}
