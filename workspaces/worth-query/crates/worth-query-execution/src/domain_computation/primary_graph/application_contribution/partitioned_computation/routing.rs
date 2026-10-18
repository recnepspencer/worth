//! Routes items to partitions by key digest, through `worth-execution`'s
//! keyed partitioner.

#[cfg(test)]
use worth_execution::KeyedDenial;
use worth_execution::{
    KeyedEditDenial, KeyedItem, KeyedPartitioner, PartitionItemId, PartitionWork,
};
use worth_foundational::facade::PartitionIdentity;
use worth_query_declaration::facade::application_operation::ApplicationComputationPartitionIdentity;

use super::super::WorthQueryManagedComputationResourceDenial;

/// Why an item has no partition.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ComputationPartitionRoutingDenial {
    /// The request's memory refused what the partitioner would retain.
    Resource(WorthQueryManagedComputationResourceDenial),
    /// A different key digest already owns this partition identity.
    IdentityCollision(PartitionIdentity),
}

/// The partitioner's key is the 32-byte digest of a partition key's canonical
/// encoding, so two keys are one partition exactly when their digests are
/// equal, and two digests that share a partition identity are a collision.
#[derive(Clone, Default)]
pub(super) struct ComputationPartitionRouting {
    partitioner: KeyedPartitioner<[u8; 32]>,
}

impl ComputationPartitionRouting {
    pub(super) fn route_in<Stopped>(
        &mut self,
        item: PartitionItemId,
        digest: [u8; 32],
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (PartitionIdentity, PartitionWork),
        super::WorthQueryPartitionedComputationDenial<Stopped>,
    > {
        use super::WorthQueryPartitionedComputationDenial as Denial;
        let partition = ApplicationComputationPartitionIdentity::partition_of(&digest);
        let mut next = self.partitioner.clone();
        next.remove_in(item, policy).map_err(Denial::from_ordered)?;
        let work = next
            .upsert_in(
                &KeyedItem {
                    item,
                    key: digest,
                    partition,
                },
                policy,
            )
            .map_err(|reason| match reason {
                KeyedEditDenial::Admission(cause) => Denial::from_ordered(cause),
                KeyedEditDenial::BoundOverflow => {
                    Denial::Resource(WorthQueryManagedComputationResourceDenial::CapacityOverflow)
                }
                KeyedEditDenial::Keyed(_) => Denial::PartitionIdentityCollision { partition },
            })?;
        self.partitioner = next;
        Ok((partition, work))
    }
    pub(super) fn kept_in<Stopped>(
        &self,
        keep: impl FnMut(PartitionItemId) -> bool,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, super::WorthQueryPartitionedComputationDenial<Stopped>> {
        use super::WorthQueryPartitionedComputationDenial as Denial;
        self.partitioner
            .kept_in(keep, policy)
            .map(|partitioner| Self { partitioner })
            .map_err(|why| match why {
                KeyedEditDenial::Admission(cause) => Denial::from_ordered(cause),
                _ => Denial::Resource(WorthQueryManagedComputationResourceDenial::CapacityOverflow),
            })
    }
    /// Routes one item and returns its partition with the work of routing it
    /// into a partitioner that did not hold it: an item routed again leaves
    /// its last partition first, uncharged, so every route charges what a
    /// fresh build's route of it charges. Before the partitioner grows,
    /// `admit` is offered all it would retain.
    #[cfg(test)]
    pub(super) fn route(
        &mut self,
        item: PartitionItemId,
        digest: [u8; 32],
        admit: impl FnOnce(u64) -> Result<(), WorthQueryManagedComputationResourceDenial>,
    ) -> Result<(PartitionIdentity, PartitionWork), ComputationPartitionRoutingDenial> {
        let partition = ApplicationComputationPartitionIdentity::partition_of(&digest);
        // The identity `item` leaves, when no other item holds it, is free
        // again for another digest.
        self.partitioner.remove(item);
        self.partitioner
            .upsert(
                KeyedItem {
                    item,
                    key: digest,
                    partition,
                },
                admit,
            )
            .map(|work| (partition, work))
            .map_err(|denial| match denial {
                // A partition identity is a function of the digest, so one
                // digest never changes identity: both refusals are two digests
                // meeting in one identity.
                KeyedEditDenial::Keyed(
                    KeyedDenial::IdentityCollision { partition }
                    | KeyedDenial::KeyIdentityChanged { partition },
                ) => ComputationPartitionRoutingDenial::IdentityCollision(partition),
                KeyedEditDenial::Admission(denial) => {
                    ComputationPartitionRoutingDenial::Resource(denial)
                }
                KeyedEditDenial::BoundOverflow => ComputationPartitionRoutingDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                ),
            })
    }

    /// The partition `item` routes to, when it is routed.
    pub(super) fn partition_of(&self, item: PartitionItemId) -> Option<PartitionIdentity> {
        self.partitioner.route(item)
    }

    /// The full digest that owns a compact partition identity.
    pub(super) fn digest(&self, partition: PartitionIdentity) -> Option<&[u8; 32]> {
        self.partitioner.key(partition)
    }

    /// Every partition in ascending identity order.
    pub(super) fn partitions(&self) -> impl ExactSizeIterator<Item = PartitionIdentity> + '_ {
        self.partitioner.partitions()
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

    /// What the routing holds, as the partitioner charges it, or `None` when
    /// the count overflows.
    pub(super) fn charged_bytes(&self) -> Option<u64> {
        self.partitioner.charged_bytes()
    }
}
