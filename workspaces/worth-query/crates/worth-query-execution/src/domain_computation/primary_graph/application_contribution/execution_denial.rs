//! The one door from execution's stops and refusals to Query's denials.
//!
//! Every match here is exhaustive and names each cause, so a cause execution
//! adds is a compile error here, never a cause folded into a neighbor. No
//! public Query denial carries an execution type.

use worth_execution::{
    ChargedBytes, LeaseDenial, MapKernelFailure, MapKernelStop, MapMemoryOverflow, MapStop,
    MemoryLimitDenial, MemoryLimitLevel, ReduceInputDenial, ReductionDenial, ReductionRunFailure,
    ReductionRunStop, WorkCeilingDenial,
};

use super::partitioned_computation::{
    WorthQueryComputationPartitionStop, WorthQueryPartitionedComputationDenial,
    WorthQueryReductionInputDenial as ReductionInput,
};
use super::{
    WorthQueryManagedComputationCheckpointDenial, WorthQueryManagedComputationDenial,
    WorthQueryManagedComputationInterruption, WorthQueryManagedComputationResourceDenial,
    WorthQueryMemoryLimitLevel,
};

type Resource = WorthQueryManagedComputationResourceDenial;

/// What a lease refusal is to the computation that asked for the lease.
pub(super) const fn lease_denial(denial: LeaseDenial) -> Resource {
    match denial {
        LeaseDenial::WorkerLimitExceedsParent => Resource::WorkerLimit,
        LeaseDenial::MemoryLimitExceedsParent => Resource::PolicyMemoryLimit,
        LeaseDenial::WorkLimitExceedsParent => Resource::WorkLimit,
        LeaseDenial::MemoryExhausted(denial) => memory_denial(denial),
        LeaseDenial::ChargedBytesOverflow => Resource::ChargedBytesOverflow,
        LeaseDenial::UnrelatedNestedLease => Resource::NestedLeaseMisuse,
        LeaseDenial::EquivalenceContractUnavailable => Resource::EquivalenceContractUnavailable,
    }
}

/// A refused reservation, with the bytes it asked for, the limit that
/// refused and the room it left. A run's refusal and a reservation's have
/// this one spelling.
pub(super) const fn memory_denial(denial: MemoryLimitDenial) -> Resource {
    Resource::MemoryLimit {
        requested: denial.requested,
        admitted: denial.admitted,
        level: match denial.level {
            MemoryLimitLevel::Policy { .. } => WorthQueryMemoryLimitLevel::Policy,
            MemoryLimitLevel::Process => WorthQueryMemoryLimitLevel::Process,
            MemoryLimitLevel::Declared => WorthQueryMemoryLimitLevel::Declared,
        },
    }
}

/// What an owner learns at a checkpoint from the kernel stop it met. Each
/// stop is its own cause.
pub(super) const fn checkpoint_denial(
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
        MapKernelStop::NestedStopped => {
            WorthQueryManagedComputationCheckpointDenial::NestedPatternStopped
        }
    }
}

/// The kernel stop an interruption is, for a safe point outside any kernel.
pub(super) const fn interruption_stop(
    interruption: WorthQueryManagedComputationInterruption,
) -> MapKernelStop {
    match interruption {
        WorthQueryManagedComputationInterruption::Cancelled => MapKernelStop::Cancelled,
        WorthQueryManagedComputationInterruption::DeadlineExceeded => {
            MapKernelStop::DeadlineElapsed
        }
    }
}

/// An owner's refusal inside a kernel, carried through the kernel unchanged.
#[derive(Debug, Eq, PartialEq)]
pub(super) enum PartitionRefusal<Stopped> {
    Owner(Stopped),
    Resource(Resource),
}

impl<Stopped: ChargedBytes> ChargedBytes for PartitionRefusal<Stopped> {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Owner(stopped) => stopped.additional_charged_bytes(),
            Self::Resource(_) => 0,
        }
    }
}

/// What a partition's refusal is to the execution kernel. A stop the kernel
/// itself would make stays that stop, so the map settles its canonical
/// boundary on it; every other refusal travels as the partition's own.
pub(super) fn kernel_failure<Stopped>(
    denial: WorthQueryManagedComputationDenial<Stopped>,
) -> MapKernelFailure<PartitionRefusal<Stopped>> {
    match denial {
        WorthQueryManagedComputationDenial::Owner(stopped) => {
            MapKernelFailure::Domain(PartitionRefusal::Owner(stopped))
        }
        WorthQueryManagedComputationDenial::Interrupted(interruption) => {
            MapKernelFailure::Stop(interruption_stop(interruption))
        }
        WorthQueryManagedComputationDenial::NestedPatternStopped => {
            MapKernelFailure::Stop(MapKernelStop::NestedStopped)
        }
        WorthQueryManagedComputationDenial::Resource(denial) => match denial {
            Resource::WorkExhausted => MapKernelFailure::Stop(MapKernelStop::WorkCeiling),
            Resource::WorkCounterOverflow => {
                MapKernelFailure::Stop(MapKernelStop::WorkCounterOverflow)
            }
            Resource::RetainedBytesExhausted
            | Resource::ScratchCapacityExceeded
            | Resource::ChargedBytesOverflow
            | Resource::ResultCapacityExceeded
            | Resource::CapacityOverflow
            | Resource::MemoryLimit { .. }
            | Resource::WorkerLimit
            | Resource::PolicyMemoryLimit
            | Resource::WorkLimit
            | Resource::NestedLeaseMisuse
            | Resource::EquivalenceContractUnavailable => {
                MapKernelFailure::Domain(PartitionRefusal::Resource(denial))
            }
        },
    }
}

impl<Stopped> WorthQueryPartitionedComputationDenial<Stopped> {
    /// A stop that belongs to no one partition.
    pub(super) const fn from_kernel_stop(stop: MapKernelStop) -> Self {
        match checkpoint_denial(stop) {
            WorthQueryManagedComputationCheckpointDenial::Resource(denial) => {
                Self::Resource(denial)
            }
            WorthQueryManagedComputationCheckpointDenial::Interrupted(interruption) => {
                Self::Interrupted(interruption)
            }
            WorthQueryManagedComputationCheckpointDenial::NestedPatternStopped => {
                Self::NestedPatternStopped
            }
        }
    }

    /// A keyless map's declared bytes that have no sum.
    pub(super) const fn from_map_overflow(_: MapMemoryOverflow) -> Self {
        Self::Resource(Resource::CapacityOverflow)
    }

    /// A ceiling's refusal. Its patterns contain their own panics, so a
    /// panic here is in the code the ceiling ran around the pattern, and the
    /// caller names it by `panicked`.
    pub(super) fn from_work_ceiling(denial: WorkCeilingDenial, panicked: Self) -> Self {
        match denial {
            WorkCeilingDenial::Admission(denial) => Self::Resource(lease_denial(denial)),
            WorkCeilingDenial::Stopped(stop) => Self::from_kernel_stop(stop),
            WorkCeilingDenial::Panicked => panicked,
        }
    }

    pub(super) fn from_reduce(denial: ReduceInputDenial<PartitionRefusal<Stopped>>) -> Self {
        match denial {
            ReduceInputDenial::ScopeAdmission { denial, .. } => {
                Self::Resource(lease_denial(denial))
            }
            ReduceInputDenial::MapStopped { reason, .. } => Self::from_map_stop(reason),
            ReduceInputDenial::ReductionStopped { failure, .. } => Self::from_reduction(failure),
        }
    }

    /// Why a map stopped, as the partition it stopped at.
    pub(super) fn from_map_stop(reason: MapStop<PartitionRefusal<Stopped>>) -> Self {
        match reason {
            MapStop::WorkExhausted { identity } => Self::Partition {
                partition: identity,
                cause: WorthQueryComputationPartitionStop::Resource(Resource::WorkExhausted),
            },
            MapStop::Failure { identity, cause } => Self::Partition {
                partition: identity,
                cause: WorthQueryComputationPartitionStop::from_kernel_failure(cause),
            },
            MapStop::Admission(denial) => Self::Resource(lease_denial(denial)),
        }
    }

    /// Why the reduction over the canonical tree stopped.
    pub(super) fn from_reduction(failure: ReductionRunFailure<MapKernelStop>) -> Self {
        match failure.reason {
            ReductionRunStop::Hook(stop) => Self::from_kernel_stop(stop),
            ReductionRunStop::Panic | ReductionRunStop::Denial(ReductionDenial::ReducerPanic) => {
                Self::ReducerPanicked
            }
            ReductionRunStop::ResultCapacityExceeded
            | ReductionRunStop::Denial(ReductionDenial::ResultCapacityExceeded) => {
                Self::Resource(Resource::ResultCapacityExceeded)
            }
            ReductionRunStop::WorkCounterOverflow
            | ReductionRunStop::Denial(ReductionDenial::WorkCounterOverflow) => {
                Self::Resource(Resource::WorkCounterOverflow)
            }
            ReductionRunStop::Denial(ReductionDenial::InvalidCanonicalEncoding) => {
                Self::ReducedEncodingInvalid
            }
            ReductionRunStop::Denial(ReductionDenial::IdentitiesNotCanonical) => {
                Self::ReductionInputInvalid(ReductionInput::IdentitiesNotCanonical)
            }
            ReductionRunStop::Denial(ReductionDenial::ValueCountMismatch) => {
                Self::ReductionInputInvalid(ReductionInput::ValueCountMismatch)
            }
            ReductionRunStop::Denial(ReductionDenial::CoverageMismatch) => {
                Self::ReductionInputInvalid(ReductionInput::CoverageMismatch)
            }
            ReductionRunStop::Denial(ReductionDenial::UnknownIdentity(partition)) => {
                Self::ReductionInputInvalid(ReductionInput::UnknownPartition { partition })
            }
            ReductionRunStop::Denial(ReductionDenial::IdentityAlreadyPresent(partition)) => {
                Self::ReductionInputInvalid(ReductionInput::PartitionAlreadyPresent { partition })
            }
        }
    }
}

impl<Stopped> WorthQueryComputationPartitionStop<Stopped> {
    fn from_kernel_failure(cause: MapKernelFailure<PartitionRefusal<Stopped>>) -> Self {
        match cause {
            MapKernelFailure::Domain(PartitionRefusal::Owner(stopped)) => Self::Owner(stopped),
            MapKernelFailure::Domain(PartitionRefusal::Resource(denial)) => Self::Resource(denial),
            MapKernelFailure::Panic => Self::Panicked,
            MapKernelFailure::ResultCapacityExceeded => {
                Self::Resource(Resource::ResultCapacityExceeded)
            }
            MapKernelFailure::Stop(stop) => match checkpoint_denial(stop) {
                WorthQueryManagedComputationCheckpointDenial::Resource(denial) => {
                    Self::Resource(denial)
                }
                WorthQueryManagedComputationCheckpointDenial::Interrupted(interruption) => {
                    Self::Interrupted(interruption)
                }
                WorthQueryManagedComputationCheckpointDenial::NestedPatternStopped => {
                    Self::NestedPatternStopped
                }
            },
        }
    }
}

#[cfg(test)]
mod tests;
