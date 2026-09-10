//! Retained collision-vector extents for the installed im 15.1 HAMT.
//!
//! Its full hash is u32 (config::HashLevelSize = U5). CollisionNode starts
//! with two elements, grows its Vec geometrically, keeps capacity on removal,
//! and releases the vector when one element remains. Public iteration cannot
//! recover this allocation history. Track it at every overlay mutation.

use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::Arc;

use super::SharedKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CollisionExtent {
    entries: usize,
    capacity: usize,
}

#[derive(Clone, Debug, Default)]
pub(super) struct CollisionExtents {
    groups: im::OrdMap<u32, CollisionExtent>,
    retained_capacity: usize,
    collision_groups: usize,
}

impl CollisionExtents {
    pub(super) fn empty_structure_charge() -> Result<
        crate::data::retained_storage::RetainedStorageCharge,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    > {
        crate::data::retained_storage::ordered_index_charge::<u32, CollisionExtent>(0)
    }

    /// A conservative capacity, not an allocator observation. COW can shrink
    /// the actual Vec to its current length; retaining its ceiling is safe.
    fn insert(&mut self, hash: u32) -> Option<()> {
        let previous = self.groups.get(&hash).copied().unwrap_or(CollisionExtent {
            entries: 0,
            capacity: 0,
        });
        let entries = previous.entries.checked_add(1)?;
        let capacity = if entries < 2 {
            0
        } else if entries == 2 {
            2
        } else {
            // COW may copy a non-power-of-two length, then double it on the
            // next insertion. Twice the largest entry count in this collision
            // lifetime bounds both that growth and ordinary Vec doubling.
            previous.capacity.max(entries.checked_mul(2)?)
        };
        let retained_capacity = self
            .retained_capacity
            .checked_sub(previous.capacity)?
            .checked_add(capacity)?;
        let collision_groups = self
            .collision_groups
            .checked_add(usize::from(entries == 2))?;
        self.groups
            .insert(hash, CollisionExtent { entries, capacity });
        self.retained_capacity = retained_capacity;
        self.collision_groups = collision_groups;
        Some(())
    }

    fn remove(&mut self, hash: u32) -> Option<()> {
        let previous = *self.groups.get(&hash)?;
        let entries = previous.entries.checked_sub(1)?;
        let capacity = if entries < 2 { 0 } else { previous.capacity };
        let retained_capacity = self
            .retained_capacity
            .checked_sub(previous.capacity)?
            .checked_add(capacity)?;
        let collision_groups = self
            .collision_groups
            .checked_sub(usize::from(previous.entries == 2))?;
        if entries == 0 {
            self.groups.remove(&hash);
        } else {
            self.groups
                .insert(hash, CollisionExtent { entries, capacity });
        }
        self.retained_capacity = retained_capacity;
        self.collision_groups = collision_groups;
        Some(())
    }

    pub(super) fn retained_structure_charge<K, V>(
        &self,
    ) -> Result<
        crate::data::retained_storage::RetainedStorageCharge,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    > {
        use crate::data::retained_storage::{
            arc_allocation_charge, ordered_index_charge, RetainedStorageCharge as Charge,
        };
        ordered_index_charge::<u32, CollisionExtent>(self.groups.len())?
            .checked_add(Charge::capacity::<(SharedKey<K>, Option<Arc<V>>)>(
                self.retained_capacity,
            )?)?
            .checked_add(
                arc_allocation_charge::<(u32, Vec<(SharedKey<K>, Option<Arc<V>>)>)>()?
                    .checked_mul(self.collision_groups)?,
            )
    }

    #[cfg(test)]
    pub(super) fn counts(&self) -> (usize, usize, usize) {
        (
            self.groups.len(),
            self.collision_groups,
            self.retained_capacity,
        )
    }
}

fn full_hash<K: Hash, V>(changes: &im::HashMap<SharedKey<K>, Option<Arc<V>>>, key: &K) -> u32 {
    let mut hasher = changes.hasher().build_hasher();
    key.hash(&mut hasher);
    hasher.finish() as u32
}

pub(super) fn insert<K: Clone + Eq + Hash, V: Clone>(
    changes: &mut im::HashMap<SharedKey<K>, Option<Arc<V>>>,
    extents: &mut Option<CollisionExtents>,
    key: SharedKey<K>,
    value: Option<Arc<V>>,
) {
    let is_new = !changes.contains_key(key.as_key());
    let hash = full_hash(changes, key.as_key());
    // Unwind during mutation must not leave history describing an earlier
    // representation. Missing history cannot be reconstructed from live len.
    let mut updated = extents.take();
    if is_new {
        updated = updated.and_then(|mut extents| {
            extents.insert(hash)?;
            Some(extents)
        });
    }
    changes.insert(key, value);
    *extents = updated;
}

pub(super) fn remove<K: Clone + Eq + Hash, V: Clone>(
    changes: &mut im::HashMap<SharedKey<K>, Option<Arc<V>>>,
    extents: &mut Option<CollisionExtents>,
    key: &K,
) {
    if !changes.contains_key(key) {
        return;
    }
    let hash = full_hash(changes, key);
    let updated = extents.take().and_then(|mut extents| {
        extents.remove(hash)?;
        Some(extents)
    });
    changes.remove(key);
    *extents = updated;
}
