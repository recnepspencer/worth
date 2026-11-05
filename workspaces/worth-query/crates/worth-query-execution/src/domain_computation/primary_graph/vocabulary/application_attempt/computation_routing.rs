//! Moved whole from `domain_computation::primary_graph::application_attempt::read_set::computation_routing`: the items below and every impl they own.
use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;

/// One owner call of a partitioned computation that reads through the lent
/// reader.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ComputationRead {
    /// `partitions`: which items the input holds.
    Membership,
    /// `partition_key` for one item.
    ItemKey(PartitionItemId),
    /// `gather` for one partition.
    Partition(PartitionIdentity),
}

impl worth_execution::ChargedBytes for ComputationRead {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
