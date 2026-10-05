//! Routes items to partitions by key digest, through `worth-execution`'s
//! keyed partitioner.

use worth_execution::{
    KeyedDenial, KeyedItem, KeyedPartitioner, PartitionItemId, PartitionWork, SourceFactId,
};
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_operation::ApplicationComputationPartitionIdentity;

/// Why an item has no partition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ComputationPartitionRoutingDenial {
    /// The plan named this item identity twice.
    DuplicateItem(PartitionItemId),
    /// A different key digest already owns this partition identity.
    IdentityCollision(PartitionIdentity),
}

/// The partitioner's key is the 32-byte digest of a partition key's canonical
/// encoding, so two keys are one partition exactly when their digests are
/// equal, and two digests that share a partition identity are a collision.
#[derive(Default)]
pub(super) struct ComputationPartitionRouting {
    partitioner: KeyedPartitioner<[u8; 32]>,
}

impl ComputationPartitionRouting {
    /// Routes one item and returns its partition with the partitioner's work.
    pub(super) fn route(
        &mut self,
        item: PartitionItemId,
        digest: [u8; 32],
    ) -> Result<(PartitionIdentity, PartitionWork), ComputationPartitionRoutingDenial> {
        if self.partitioner.route(item).is_some() {
            return Err(ComputationPartitionRoutingDenial::DuplicateItem(item));
        }
        let partition = ApplicationComputationPartitionIdentity::partition_of(&digest);
        self.partitioner
            .upsert(KeyedItem {
                item,
                // Every run recomputes every partition, so nothing reads the
                // fact an item came from yet.
                source_fact: SourceFactId(item.0),
                key: digest,
                partition,
            })
            .map(|work| (partition, work))
            .map_err(|denial| match denial {
                // A partition identity is a function of the digest, so one
                // digest never changes identity: both refusals are two digests
                // meeting in one identity.
                KeyedDenial::IdentityCollision { partition }
                | KeyedDenial::KeyIdentityChanged { partition } => {
                    ComputationPartitionRoutingDenial::IdentityCollision(partition)
                }
            })
    }

    /// A partition's items in ascending item identity order.
    pub(super) fn members(
        &self,
        partition: PartitionIdentity,
    ) -> impl Iterator<Item = PartitionItemId> + '_ {
        self.partitioner
            .members(partition)
            .into_iter()
            .flat_map(|members| members.iter().copied())
    }
}
