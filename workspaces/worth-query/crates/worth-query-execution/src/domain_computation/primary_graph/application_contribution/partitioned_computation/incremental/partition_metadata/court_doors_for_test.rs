//! Sealed test doors over this module's private members.
use super::*;

impl<Key> PartitionMetadata<Key> {
    #[doc(hidden)]
    pub fn from_recorded_for_test(
        entries:BTreeMap<PartitionIdentity,(Arc<Key>,crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge)>,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, ExecutionOrderedMapDenial> {
        Self::from_recorded(entries, policy)
    }
}

impl<Key> PartitionMetadata<Key> {
    #[doc(hidden)]
    pub fn values_for_test(&self) -> impl Iterator<Item = &RetainedPartition<Key>> {
        Self::values(self)
    }
}

impl<Key> PartitionMetadata<Key> {
    #[doc(hidden)]
    pub fn bytes_for_test(&self) -> Option<u64> {
        Self::bytes(self)
    }
}

impl<Key> PartitionMetadata<Key> {
    #[doc(hidden)]
    pub fn fork_for_test(&self) -> Self {
        Self::fork(self)
    }
}

impl<Key> PartitionMetadata<Key> {
    #[doc(hidden)]
    pub fn get_for_test(&self, id: &PartitionIdentity) -> Option<&RetainedPartition<Key>> {
        Self::get(self, id)
    }
}

impl<Key> PartitionMetadata<Key> {
    #[doc(hidden)]
    pub fn iter_for_test(
        &self,
    ) -> impl Iterator<Item = (&PartitionIdentity, &RetainedPartition<Key>)> {
        Self::iter(self)
    }
}

impl<Key> PartitionMetadata<Key> {
    #[doc(hidden)]
    pub fn keys_for_test(&self) -> impl Iterator<Item = &PartitionIdentity> {
        Self::keys(self)
    }
}

impl<Key> PartitionMetadata<Key> {
    #[doc(hidden)]
    pub fn len_for_test(&self) -> usize {
        Self::len(self)
    }
}

impl<Key> PartitionMetadata<Key> {
    #[doc(hidden)]
    pub fn put_for_test(
        &mut self,
        id: PartitionIdentity,
        value: RetainedPartition<Key>,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), ExecutionOrderedMapDenial> {
        Self::put(self, id, value, policy)
    }
}
