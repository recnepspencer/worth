//! Persistent partition descriptors; each new cell owns direct native funding.
use super::retained::RetainedPartition;
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::{ExecutionAllocationPolicy, ExecutionOrderedMap, ExecutionOrderedMapDenial};
use worth_foundational::facade::PartitionIdentity;

pub(super) struct PartitionMetadata<Key> {
    entries: ExecutionOrderedMap<PartitionIdentity, RetainedPartition<Key>>,
}
impl<Key> PartitionMetadata<Key> {
    pub(super) fn from_recorded(
        entries:BTreeMap<PartitionIdentity,(Arc<Key>,crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge)>,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, ExecutionOrderedMapDenial> {
        let mut result = Self {
            entries: Default::default(),
        };
        for (id, (key, gather)) in entries {
            result.put(id, RetainedPartition { key, gather }, policy)?;
        }
        Ok(result)
    }
    pub(super) fn fork(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
    pub(super) fn put(
        &mut self,
        id: PartitionIdentity,
        value: RetainedPartition<Key>,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), ExecutionOrderedMapDenial> {
        self.entries.insert_cloned_native_in(&id, &value, policy)
    }
    pub(super) fn remove(
        &mut self,
        id: PartitionIdentity,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), ExecutionOrderedMapDenial> {
        self.entries.remove_in(&id, policy).map(|_| ())
    }
    pub(super) fn get(&self, id: &PartitionIdentity) -> Option<&RetainedPartition<Key>> {
        self.entries.get(id)
    }
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
    pub(super) fn iter(
        &self,
    ) -> impl Iterator<Item = (&PartitionIdentity, &RetainedPartition<Key>)> {
        self.entries.iter()
    }
    #[cfg(test)]
    pub(super) fn values(&self) -> impl Iterator<Item = &RetainedPartition<Key>> {
        self.entries.values()
    }
    pub(super) fn keys(&self) -> impl Iterator<Item = &PartitionIdentity> {
        self.entries.keys()
    }
    #[cfg(test)]
    pub(super) fn bytes(&self) -> Option<u64> {
        self.entries.accounted_bytes()
    }
}
