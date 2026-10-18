//! Persistent partition keys and call charges; native results live in Execution.
use super::super::super::{
    request_execution::{QueryMemoryReservation, QueryRequestExecution},
    WorthQueryManagedComputationResourceDenial as Resource,
};
use super::retained::RetainedPartition;
use std::{collections::BTreeMap, sync::Arc};
use worth_foundational::facade::PartitionIdentity;

/// Each immutable view owns its cached backing grant. A fresh fork admits
/// that bound before sharing a root; im copies only changed paths. No parent
/// view or request/session borrow is retained for funding.
pub(super) struct PartitionMetadata<Key> {
    entries: im::OrdMap<PartitionIdentity, Arc<RetainedPartition<Key>>>,
    payload_bytes: u64,
    memory: QueryMemoryReservation,
}
impl<Key> PartitionMetadata<Key> {
    pub(super) fn from_recorded(
        entries: BTreeMap<PartitionIdentity, (Arc<Key>, u64, crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge)>,
        execution: &QueryRequestExecution<'_>,
    ) -> Result<Self, Resource> {
        let payload_bytes = entries
            .values()
            .try_fold(0_u64, |sum, (_, key_bytes, gather)| {
                sum.checked_add(entry_parts::<Key>(*key_bytes, gather)?)
            })
            .ok_or(Resource::CapacityOverflow)?;
        let bytes = structure_bytes::<Key>(entries.len())
            .and_then(|v| v.checked_add(payload_bytes))
            .ok_or(Resource::CapacityOverflow)?;
        let memory = execution.reserve(bytes)?;
        Ok(Self {
            entries: entries
                .into_iter()
                .map(|(id, (key, key_bytes, gather))| {
                    (
                        id,
                        Arc::new(RetainedPartition {
                            key,
                            key_bytes,
                            gather,
                        }),
                    )
                })
                .collect(),
            payload_bytes,
            memory,
        })
    }
    pub(super) fn fork(&self, execution: &QueryRequestExecution<'_>) -> Result<Self, Resource> {
        let memory = execution.reserve(self.bytes().ok_or(Resource::CapacityOverflow)?)?;
        Ok(Self {
            entries: self.entries.clone(),
            payload_bytes: self.payload_bytes,
            memory,
        })
    }
    pub(super) fn put(
        &mut self,
        id: PartitionIdentity,
        value: RetainedPartition<Key>,
    ) -> Result<(), Resource> {
        let old = self
            .entries
            .get(&id)
            .map_or(Some(0), |v| entry_bytes(v))
            .ok_or(Resource::CapacityOverflow)?;
        let new = entry_bytes(&value).ok_or(Resource::CapacityOverflow)?;
        let count = self
            .entries
            .len()
            .checked_add(usize::from(!self.entries.contains_key(&id)))
            .ok_or(Resource::CapacityOverflow)?;
        let structure = structure_bytes::<Key>(count)
            .and_then(|v| v.checked_sub(structure_bytes::<Key>(self.entries.len())?))
            .ok_or(Resource::CapacityOverflow)?;
        let payload = self
            .payload_bytes
            .checked_sub(old)
            .and_then(|v| v.checked_add(new))
            .ok_or(Resource::CapacityOverflow)?;
        self.memory.grow(
            structure
                .checked_add(new)
                .ok_or(Resource::CapacityOverflow)?,
        )?;
        self.entries.insert(id, Arc::new(value));
        self.payload_bytes = payload;
        self.memory.resize(
            self.memory
                .bytes()
                .checked_sub(old)
                .ok_or(Resource::CapacityOverflow)?,
        )
    }
    pub(super) fn remove(&mut self, id: PartitionIdentity) -> Result<(), Resource> {
        let Some(value) = self.entries.get(&id) else {
            return Ok(());
        };
        let old = entry_bytes(value).ok_or(Resource::CapacityOverflow)?;
        let before = self.bytes().ok_or(Resource::CapacityOverflow)?;
        self.entries.remove(&id);
        self.payload_bytes = self
            .payload_bytes
            .checked_sub(old)
            .ok_or(Resource::CapacityOverflow)?;
        let release = before
            .checked_sub(self.bytes().ok_or(Resource::CapacityOverflow)?)
            .ok_or(Resource::CapacityOverflow)?;
        self.memory.resize(
            self.memory
                .bytes()
                .checked_sub(release)
                .ok_or(Resource::CapacityOverflow)?,
        )
    }
    pub(super) fn get(&self, id: &PartitionIdentity) -> Option<&Arc<RetainedPartition<Key>>> {
        self.entries.get(id)
    }
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
    pub(super) fn iter(
        &self,
    ) -> impl Iterator<Item = (&PartitionIdentity, &Arc<RetainedPartition<Key>>)> {
        self.entries.iter()
    }
    #[cfg(test)]
    pub(super) fn values(&self) -> impl Iterator<Item = &Arc<RetainedPartition<Key>>> {
        self.entries.values()
    }
    pub(super) fn keys(&self) -> impl Iterator<Item = &PartitionIdentity> {
        self.entries.keys()
    }
    pub(super) fn bytes(&self) -> Option<u64> {
        structure_bytes::<Key>(self.entries.len())?.checked_add(self.payload_bytes)
    }
}
fn entry_bytes<Key>(value: &RetainedPartition<Key>) -> Option<u64> {
    entry_parts::<Key>(value.key_bytes, &value.gather)
}
fn entry_parts<Key>(
    key_bytes: u64,
    gather: &crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge,
) -> Option<u64> {
    // Preserve the existing canonical key-capacity contract, without a global
    // heap-deduplication or process-RSS promise.
    (size_of::<RetainedPartition<Key>>() as u64)
        .checked_add(2 * size_of::<usize>() as u64)?
        .checked_add(key_bytes)?
        .checked_add(gather.additional_bytes()?)
}

fn structure_bytes<Key>(count: usize) -> Option<u64> {
    // Installed im15.1/no-pool: 64 entries, 65 optional Arc children, chunk
    // bounds and Arc header. Every nonempty node owns a key. One node per
    // entry plus its empty root bounds storage without walking the tree.
    let alignment =
        align_of::<(PartitionIdentity, Arc<RetainedPartition<Key>>)>().max(align_of::<usize>());
    let node = size_of::<(PartitionIdentity, Arc<RetainedPartition<Key>>)>()
        .checked_mul(64)?
        .checked_add(size_of::<Option<Arc<()>>>().checked_mul(65)?)?
        .checked_add(size_of::<usize>().checked_mul(6)?)?
        .checked_add(alignment.checked_mul(4)?)?;
    u64::try_from(node.checked_mul(count.checked_add(1)?)?).ok()
}
