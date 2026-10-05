//! Why a partitioned computation has no result.

use worth_execution::{
    LeaseDenial, MapDenial, MapKernelFailure, MapKernelStop, MapStop, PartitionItemId,
    ReduceInputDenial, ReductionDenial, ReductionRunFailure, ReductionRunStop, WorkCeilingDenial,
};
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_schema::ApplicationValueEncodeDenial;

use super::super::{
    WorthQueryManagedComputationCheckpointDenial, WorthQueryManagedComputationInterruption,
    WorthQueryManagedComputationResourceDenial,
};
use super::reader::{WorthQueryComputationInputDenial, WorthQueryComputationReadDenial};
use super::routing::ComputationPartitionRoutingDenial;

/// Why one partition stopped.
#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryComputationPartitionStop<Stopped> {
    /// The owner refused the partition, gathering it or computing it.
    Owner(Stopped),
    /// A read was refused while the owner gathered the partition.
    Read(WorthQueryComputationReadDenial),
    /// The owner's kernel panicked.
    Panicked,
    /// The partition did not fit the computation's declared work or bytes.
    Resource(WorthQueryManagedComputationResourceDenial),
    Interrupted(WorthQueryManagedComputationInterruption),
    /// A pattern the kernel ran inside the partition stopped.
    NestedPatternStopped,
}

/// Why a partitioned computation has no result.
#[derive(Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedComputationDenial<Stopped> {
    /// The owner refused to name the input's items, to key one, or to
    /// complete the reduced result.
    Owner(Stopped),
    /// A read was refused while the owner named the input's items or keyed
    /// one.
    Read(WorthQueryComputationReadDenial),
    /// A partition stopped. When several did, this is the one with the least
    /// partition identity. No partition is computed unless every one gathers.
    Partition {
        partition: PartitionIdentity,
        cause: WorthQueryComputationPartitionStop<Stopped>,
    },
    /// Two different keys derive this one partition identity. Every run is
    /// denied while both keys are in the input.
    PartitionIdentityCollision { partition: PartitionIdentity },
    /// The plan named one item identity twice.
    DuplicateItem { item: PartitionItemId },
    /// The input value refused to encode, so it has no digest to name it.
    InputNotEncodable(ApplicationValueEncodeDenial),
    /// This item's partition key refused to encode. When several do, this is
    /// the one with the least item identity.
    KeyNotEncodable {
        item: PartitionItemId,
        denial: ApplicationValueEncodeDenial,
    },
    /// Deriving partition identities or reducing did not fit the computation's
    /// declared work or bytes.
    Resource(WorthQueryManagedComputationResourceDenial),
    /// The request was cancelled or ran out of time outside any one partition.
    Interrupted(WorthQueryManagedComputationInterruption),
    /// The reducer panicked.
    ReducerPanicked,
    /// A reduced value's canonical bits disagree with the length it declares.
    ReducedEncodingInvalid,
    /// Execution refused the run before any partition was dispatched.
    DeniedBeforeDispatch(LeaseDenial),
}

impl<Stopped> From<ComputationPartitionRoutingDenial>
    for WorthQueryPartitionedComputationDenial<Stopped>
{
    fn from(denial: ComputationPartitionRoutingDenial) -> Self {
        match denial {
            ComputationPartitionRoutingDenial::DuplicateItem(item) => Self::DuplicateItem { item },
            ComputationPartitionRoutingDenial::IdentityCollision(partition) => {
                Self::PartitionIdentityCollision { partition }
            }
        }
    }
}

impl<Stopped> From<WorthQueryComputationInputDenial<Stopped>>
    for WorthQueryPartitionedComputationDenial<Stopped>
{
    fn from(denial: WorthQueryComputationInputDenial<Stopped>) -> Self {
        match denial {
            WorthQueryComputationInputDenial::Owner(stopped) => Self::Owner(stopped),
            WorthQueryComputationInputDenial::Read(denial) => Self::Read(denial),
        }
    }
}

impl<Stopped> WorthQueryPartitionedComputationDenial<Stopped> {
    /// A partition whose gathering has no answer.
    pub(super) fn gathering(
        partition: PartitionIdentity,
        denial: WorthQueryComputationInputDenial<Stopped>,
    ) -> Self {
        Self::Partition {
            partition,
            cause: match denial {
                WorthQueryComputationInputDenial::Owner(stopped) => {
                    WorthQueryComputationPartitionStop::Owner(stopped)
                }
                WorthQueryComputationInputDenial::Read(denial) => {
                    WorthQueryComputationPartitionStop::Read(denial)
                }
            },
        }
    }

    const WORK_EXHAUSTED: Self =
        Self::Resource(WorthQueryManagedComputationResourceDenial::WorkExhausted);
    const BYTES_EXHAUSTED: Self =
        Self::Resource(WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted);

    /// A stop that belongs to no one partition.
    fn from_kernel_stop(stop: MapKernelStop) -> Self {
        match WorthQueryManagedComputationCheckpointDenial::from_kernel_stop(stop) {
            WorthQueryManagedComputationCheckpointDenial::Resource(denial) => {
                Self::Resource(denial)
            }
            WorthQueryManagedComputationCheckpointDenial::Interrupted(interruption) => {
                Self::Interrupted(interruption)
            }
        }
    }

    /// Every partition may hold the declared bytes, and the map admits only
    /// capacities that have a sum.
    pub(super) fn from_map(denial: MapDenial) -> Self {
        match denial {
            MapDenial::MemoryOverflow => Self::BYTES_EXHAUSTED,
            MapDenial::ExpectedIdentitiesNotCanonical
            | MapDenial::CoverageMismatch
            | MapDenial::ReadKeysNotCanonical { .. }
            | MapDenial::WriteKeysNotCanonical { .. }
            | MapDenial::WriteSetOverlap { .. }
            | MapDenial::ReadWriteConflict { .. } => {
                unreachable!("partitions are named once each in identity order with no access keys")
            }
        }
    }

    pub(super) fn from_work_ceiling(denial: WorkCeilingDenial) -> Self {
        match denial {
            WorkCeilingDenial::Admission(denial) => Self::DeniedBeforeDispatch(denial),
            WorkCeilingDenial::Stopped(stop) => Self::from_kernel_stop(stop),
            // Only the reduce pattern runs under the ceiling, and it contains
            // the kernel's panics: what is left is the reducer's.
            WorkCeilingDenial::Panicked => Self::ReducerPanicked,
        }
    }

    pub(super) fn from_reduce(denial: ReduceInputDenial<Stopped>) -> Self {
        match denial {
            ReduceInputDenial::ScopeAdmission { denial, .. } => Self::DeniedBeforeDispatch(denial),
            ReduceInputDenial::MapStopped { reason, .. } => Self::from_map_stop(reason),
            ReduceInputDenial::ReductionStopped { failure, .. } => Self::from_reduction(failure),
        }
    }

    /// Why a map stopped, as the partition it stopped at.
    pub(super) fn from_map_stop(reason: MapStop<Stopped>) -> Self {
        match reason {
            MapStop::WorkExhausted { identity } => Self::Partition {
                partition: identity,
                cause: WorthQueryComputationPartitionStop::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkExhausted,
                ),
            },
            MapStop::Failure { identity, cause } => Self::Partition {
                partition: identity,
                cause: WorthQueryComputationPartitionStop::from_kernel_failure(cause),
            },
            MapStop::Admission(denial) => Self::DeniedBeforeDispatch(denial),
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
                Self::BYTES_EXHAUSTED
            }
            ReductionRunStop::WorkCounterOverflow
            | ReductionRunStop::Denial(ReductionDenial::WorkCounterOverflow) => {
                Self::WORK_EXHAUSTED
            }
            ReductionRunStop::Denial(ReductionDenial::InvalidCanonicalEncoding) => {
                Self::ReducedEncodingInvalid
            }
            ReductionRunStop::Denial(
                ReductionDenial::IdentitiesNotCanonical
                | ReductionDenial::ValueCountMismatch
                | ReductionDenial::CoverageMismatch
                | ReductionDenial::UnknownIdentity(_)
                | ReductionDenial::IdentityAlreadyPresent(_),
            ) => unreachable!("the map reduces exactly the identities it admitted"),
        }
    }
}

impl<Stopped> WorthQueryComputationPartitionStop<Stopped> {
    fn from_kernel_failure(cause: MapKernelFailure<Stopped>) -> Self {
        match cause {
            MapKernelFailure::Domain(stopped) => Self::Owner(stopped),
            MapKernelFailure::Panic => Self::Panicked,
            MapKernelFailure::ResultCapacityExceeded => {
                Self::Resource(WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted)
            }
            MapKernelFailure::Stop(MapKernelStop::NestedStopped) => Self::NestedPatternStopped,
            MapKernelFailure::Stop(stop) => {
                match WorthQueryManagedComputationCheckpointDenial::from_kernel_stop(stop) {
                    WorthQueryManagedComputationCheckpointDenial::Resource(denial) => {
                        Self::Resource(denial)
                    }
                    WorthQueryManagedComputationCheckpointDenial::Interrupted(interruption) => {
                        Self::Interrupted(interruption)
                    }
                }
            }
        }
    }
}
