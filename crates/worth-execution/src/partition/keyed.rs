//! Ordered keyed grouping. Runtime edits admit only new path and group copies.
mod retained;
use super::{PartitionItemId, PartitionWork};
use crate::{
    ChargedBytes, ExecutionAllocationPolicy, ExecutionOrderedMap, ExecutionOrderedMapDenial,
};
use std::collections::BTreeSet;
use std::sync::Arc;
use worth_foundational::PartitionIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyedItem<K> {
    pub item: PartitionItemId,
    pub key: K,
    pub partition: PartitionIdentity,
}
impl<K: ChargedBytes> ChargedBytes for KeyedItem<K> {
    fn additional_charged_bytes(&self) -> u64 {
        self.key.additional_charged_bytes()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyedDenial {
    IdentityCollision { partition: PartitionIdentity },
    KeyIdentityChanged { partition: PartitionIdentity },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyedEditDenial<A> {
    Keyed(KeyedDenial),
    Admission(A),
    BoundOverflow,
}
#[derive(Clone)]
struct Group {
    partition: PartitionIdentity,
    members: Arc<retained::Members>,
}
impl ChargedBytes for Group {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
fn member_bytes() -> u64 {
    (11 * size_of::<PartitionItemId>() + 16 * size_of::<usize>()) as u64
}
/// One routing authority. A clone shares exact ordered cells and their original
/// grants. Owned key clones must not exceed the key's ChargedBytes bound.
pub struct KeyedPartitioner<K> {
    items: ExecutionOrderedMap<PartitionItemId, KeyedItem<K>>,
    keys: ExecutionOrderedMap<K, Group>,
    identities: ExecutionOrderedMap<PartitionIdentity, K>,
    key_bytes: u64,
}
impl<K> Default for KeyedPartitioner<K> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            keys: Default::default(),
            identities: Default::default(),
            key_bytes: 0,
        }
    }
}
impl<K> Clone for KeyedPartitioner<K> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            keys: self.keys.clone(),
            identities: self.identities.clone(),
            key_bytes: self.key_bytes,
        }
    }
}
impl<K: Ord + Clone + ChargedBytes> KeyedPartitioner<K> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn route(&self, item: PartitionItemId) -> Option<PartitionIdentity> {
        self.items.get(&item).map(|v| v.partition)
    }
    pub fn key(&self, partition: PartitionIdentity) -> Option<&K> {
        self.identities.get(&partition)
    }
    pub fn members(&self, partition: PartitionIdentity) -> Option<&BTreeSet<PartitionItemId>> {
        self.key(partition)
            .and_then(|key| self.keys.get(key).map(|group| &group.members.values))
    }
    pub fn partitions(&self) -> impl ExactSizeIterator<Item = PartitionIdentity> + '_ {
        self.identities.keys().copied()
    }
    /// Complete reachable capacity, separate from actual new allocation.
    pub fn charged_bytes(&self) -> Option<u64> {
        Self::retained_bytes(self.items.len(), self.keys.len(), self.key_bytes)
    }
    pub fn retained_bytes(items: usize, keys: usize, key_bytes: u64) -> Option<u64> {
        use crate::storage::ordered::node::cell_bytes;
        let item = cell_bytes::<PartitionItemId, KeyedItem<K>>()
            .ok()?
            .checked_add(member_bytes())?;
        let group = cell_bytes::<K, Group>()
            .ok()?
            .checked_add(cell_bytes::<PartitionIdentity, K>().ok()?)?
            .checked_add((size_of::<retained::Members>() + 2 * size_of::<usize>()) as u64)?;
        (items as u64)
            .checked_mul(item)?
            .checked_add((keys as u64).checked_mul(group)?)?
            .checked_add(key_bytes)
    }
    /// Explicit offline construction with a complete caller-held capacity
    /// admission. Native runtime callers use upsert_in, whose cells own grants.
    pub fn upsert<A>(
        &mut self,
        entry: KeyedItem<K>,
        admit: impl FnOnce(u64) -> Result<(), A>,
    ) -> Result<PartitionWork, KeyedEditDenial<A>> {
        self.check(&entry).map_err(KeyedEditDenial::Keyed)?;
        let bytes = self.quote(&entry).ok_or(KeyedEditDenial::BoundOverflow)?;
        admit(bytes).map_err(KeyedEditDenial::Admission)?;
        self.upsert_in(&entry, ExecutionAllocationPolicy::SystemAllocation)
            .map_err(|why| match why {
                KeyedEditDenial::Keyed(k) => KeyedEditDenial::Keyed(k),
                _ => KeyedEditDenial::BoundOverflow,
            })
    }
    /// Explicit offline removal. Admitted runtime path removal can allocate
    /// replacement ancestors and therefore uses remove_in.
    pub fn remove(&mut self, item: PartitionItemId) -> PartitionWork {
        self.remove_in(item, ExecutionAllocationPolicy::SystemAllocation)
            .expect("offline grouping capacity remains representable")
    }
    /// Honest full subset reconstruction; it is not the ordinary rekey path.
    /// `keep` returns the same answer during the quote and construction passes.
    pub fn kept<A>(
        &self,
        mut keep: impl FnMut(PartitionItemId) -> bool,
        admit: impl FnOnce(u64) -> Result<(), A>,
    ) -> Result<Self, KeyedEditDenial<A>> {
        let (mut items, mut groups, mut heap) = (0usize, 0usize, 0u64);
        for (key, group) in &self.keys {
            let mut first = true;
            for item in &group.members.values {
                if keep(*item) {
                    items = items.checked_add(1).ok_or(KeyedEditDenial::BoundOverflow)?;
                    groups = groups
                        .checked_add(usize::from(first))
                        .ok_or(KeyedEditDenial::BoundOverflow)?;
                    heap = heap
                        .checked_add(
                            key.additional_charged_bytes()
                                .checked_mul(if first { 3 } else { 1 })
                                .ok_or(KeyedEditDenial::BoundOverflow)?,
                        )
                        .ok_or(KeyedEditDenial::BoundOverflow)?;
                    first = false;
                }
            }
        }
        admit(Self::retained_bytes(items, groups, heap).ok_or(KeyedEditDenial::BoundOverflow)?)
            .map_err(KeyedEditDenial::Admission)?;
        let mut next = Self::new();
        for entry in self.items.values().filter(|v| keep(v.item)) {
            next.upsert_in(entry, ExecutionAllocationPolicy::SystemAllocation)
                .map_err(|_| KeyedEditDenial::BoundOverflow)?;
        }
        Ok(next)
    }
    fn check(&self, entry: &KeyedItem<K>) -> Result<(), KeyedDenial> {
        if let Some(group) = self.keys.get(&entry.key)
            && group.partition != entry.partition
        {
            return Err(KeyedDenial::KeyIdentityChanged {
                partition: group.partition,
            });
        }
        if self.key(entry.partition).is_some_and(|k| k != &entry.key) {
            return Err(KeyedDenial::IdentityCollision {
                partition: entry.partition,
            });
        }
        Ok(())
    }
    fn quote(&self, entry: &KeyedItem<K>) -> Option<u64> {
        let old = self
            .items
            .get(&entry.item)
            .map_or(0, |v| v.key.additional_charged_bytes());
        let fresh = !self.keys.contains_key(&entry.key);
        let heap = self.key_bytes.checked_sub(old)?.checked_add(
            entry
                .key
                .additional_charged_bytes()
                .checked_mul(if fresh { 3 } else { 1 })?,
        )?;
        Self::retained_bytes(
            self.items.len() + usize::from(!self.items.contains_key(&entry.item)),
            self.keys.len() + usize::from(fresh),
            heap,
        )
    }
}
#[cfg(test)]
mod tests;
