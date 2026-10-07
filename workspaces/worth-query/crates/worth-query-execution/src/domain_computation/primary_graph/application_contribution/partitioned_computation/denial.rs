//! Why a partitioned computation has no result.

use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_schema::ApplicationValueEncodeDenial;

use super::super::{
    WorthQueryManagedComputationInterruption, WorthQueryManagedComputationResourceDenial,
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
    /// The partition did not fit the computation's declared work or bytes,
    /// or the request's execution refused it.
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
    /// The plan named one item identity twice. When several are, this is
    /// the least.
    DuplicateItem { item: PartitionItemId },
    /// The input value refused to encode, so it has no digest to name it.
    InputNotEncodable(ApplicationValueEncodeDenial),
    /// This item refused to encode, so a producer's run has no digest to
    /// know it by. When several do, this is the one with the least item
    /// identity.
    ItemNotEncodable {
        item: PartitionItemId,
        denial: ApplicationValueEncodeDenial,
    },
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
    /// A pattern the run started inside a partition's kernel stopped, outside
    /// any one partition.
    NestedPatternStopped,
    /// The reducer panicked.
    ReducerPanicked,
    /// A reduced value's canonical bits disagree with the length it declares.
    ReducedEncodingInvalid,
    /// The reduction refused the identities or values the map gave it.
    /// Execution builds both from one canonical list, so no run has met it.
    ReductionInputInvalid(WorthQueryReductionInputDenial),
}

/// Why the reduction refused the map's identities or values. Each is its own
/// cause, as execution names it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryReductionInputDenial {
    /// The identities were not sorted and unique.
    IdentitiesNotCanonical,
    /// The map gave a different number of values than identities.
    ValueCountMismatch,
    /// The values did not cover the identities the reduction holds.
    CoverageMismatch,
    /// A value named a partition the reduction does not hold.
    UnknownPartition { partition: PartitionIdentity },
    /// A value named a partition the reduction already holds a value for.
    PartitionAlreadyPresent { partition: PartitionIdentity },
}

impl<Stopped> From<ComputationPartitionRoutingDenial>
    for WorthQueryPartitionedComputationDenial<Stopped>
{
    fn from(denial: ComputationPartitionRoutingDenial) -> Self {
        match denial {
            ComputationPartitionRoutingDenial::Resource(denial) => Self::Resource(denial),
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
}
