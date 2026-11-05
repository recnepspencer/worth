//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial;

impl ComputationPartitionRouting {
    #[doc(hidden)]
    pub fn route_for_test(
        &mut self,
        item: PartitionItemId,
        digest: [u8; 32],
        admit: impl FnOnce(u64) -> Result<(), WorthQueryManagedComputationResourceDenial>,
    ) -> Result<(PartitionIdentity, PartitionWork), ComputationPartitionRoutingDenial> {
        Self::route(self, item, digest, admit)
    }
}

impl ComputationPartitionRouting {
    #[doc(hidden)]
    pub fn members_for_test(
        &self,
        partition: PartitionIdentity,
    ) -> impl Iterator<Item = PartitionItemId> + '_ {
        Self::members(self, partition)
    }
}

impl ComputationPartitionRouting {
    #[doc(hidden)]
    pub fn partition_of_for_test(&self, item: PartitionItemId) -> Option<PartitionIdentity> {
        Self::partition_of(self, item)
    }
}

impl ComputationPartitionRouting {
    #[doc(hidden)]
    pub fn partitions_for_test(&self) -> impl ExactSizeIterator<Item = PartitionIdentity> + '_ {
        Self::partitions(self)
    }
}
