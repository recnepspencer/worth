pub(crate) use {std::num::NonZeroUsize};

pub(crate) use {worth_execution::Bisection, worth_execution::BisectionDenial, worth_execution::CancellationToken, worth_execution::ChargedBytes, worth_execution::ComponentPartitioner, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap, worth_execution::KeyedDenial, worth_execution::KeyedEditDenial, worth_execution::KeyedItem, worth_execution::KeyedPartitioner, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition, worth_execution::MapStop, worth_execution::PartitionItemId, worth_execution::WeightedEdge, worth_execution::WeightedItem};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

pub(crate) fn item(id: u64) -> PartitionItemId {
    PartitionItemId(id)
}
pub(crate) fn admit(_: u64) -> Result<(), ()> {
    Ok(())
}

/// A key that owns 100 bytes of heap.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct OwnsHeap(pub(crate) &'static str);

impl ChargedBytes for OwnsHeap {
    fn additional_charged_bytes(&self) -> u64 {
        100
    }
}