mod fork_growth;
mod persistent_fork;
use crate::data::retained_storage::RetainedStorageBacking;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

#[path = "persistent_hash_map/entry_handle.rs"]
mod entry_handle;
#[path = "persistent_hash_map/iteration.rs"]
mod iteration;
#[path = "persistent_hash_map/serialization.rs"]
mod serialization;
#[path = "persistent_hash_map/traits.rs"]
mod traits;

#[cfg(test)]
#[path = "persistent_hash_map/collision_extent_tests.rs"]
mod collision_extent_tests;
#[path = "persistent_hash_map/collision_extents.rs"]
mod collision_extents;

#[path = "persistent_hash_map/charge_updates.rs"]
mod charge_updates;
#[path = "persistent_hash_map/retained_charge.rs"]
mod retained_charge;
#[cfg(test)]
#[path = "persistent_hash_map/retained_charge_tests.rs"]
mod retained_charge_tests;

use crate::data::retained_storage::RetainedStorageCharge;
pub(crate) use charge_updates::{RetainedHashMutationDenial, RetainedHashMutationOutcome};
use collision_extents::CollisionExtents;
use entry_handle::SharedKey;
use iteration::PersistentHashMapIter;

#[cfg(test)]
#[path = "persistent_hash_map/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "persistent_hash_map/fork_granule_tests.rs"]
mod fork_granule_tests;

enum PersistentHashMapStorage<K, V> {
    Exclusive(HashMap<K, V>),
    ForkShared {
        base: Arc<RetainedStorageBacking<HashMap<K, V>>>,
        changes: im::HashMap<SharedKey<K>, Option<Arc<V>>>,
        collision_extents: Option<CollisionExtents>,
        len: usize,
    },
}

/// A hash map with flat ordinary storage and per-key fork overlays.
pub(crate) struct PersistentHashMap<K, V> {
    storage: PersistentHashMapStorage<K, V>,
    base_capacity: Option<usize>,
    retained_charge: Option<RetainedStorageCharge>,
}

impl<K, V> PersistentHashMap<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    pub(crate) fn new() -> Self {
        Self {
            storage: PersistentHashMapStorage::Exclusive(HashMap::new()),
            base_capacity: Some(0),
            retained_charge: Some(RetainedStorageCharge::ZERO),
        }
    }

    pub(crate) fn len(&self) -> usize {
        match &self.storage {
            PersistentHashMapStorage::Exclusive(values) => values.len(),
            PersistentHashMapStorage::ForkShared { len, .. } => *len,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub(crate) fn get(&self, key: &K) -> Option<&V> {
        match &self.storage {
            PersistentHashMapStorage::Exclusive(values) => values.get(key),
            PersistentHashMapStorage::ForkShared { base, changes, .. } => changes
                .get(key)
                .map_or_else(|| base.get(key), |value| value.as_ref().map(Arc::as_ref)),
        }
    }

    pub(crate) fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.retained_charge = None;
        match &mut self.storage {
            PersistentHashMapStorage::Exclusive(values) => values.get_mut(key),
            PersistentHashMapStorage::ForkShared {
                base,
                changes,
                collision_extents,
                ..
            } => {
                if !changes.contains_key(key) {
                    let value = Arc::new(base.get(key).cloned()?);
                    collision_extents::insert(
                        changes,
                        collision_extents,
                        SharedKey::new(key.clone()),
                        Some(value),
                    );
                }
                changes
                    .get_mut(key)
                    .and_then(Option::as_mut)
                    .map(Arc::make_mut)
            }
        }
    }

    #[inline]
    pub(crate) fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.retained_charge = None;
        match &mut self.storage {
            PersistentHashMapStorage::Exclusive(values) => {
                let previous_capacity = self.base_capacity.take();
                let previous = values.insert(key, value);
                self.base_capacity =
                    previous_capacity.map(|capacity| capacity.max(values.capacity()));
                previous
            }
            PersistentHashMapStorage::ForkShared {
                base,
                changes,
                collision_extents,
                len,
            } => {
                let prior = changes
                    .get(&key)
                    .map_or_else(|| base.get(&key), |value| value.as_ref().map(Arc::as_ref))
                    .cloned();
                if prior.is_none() {
                    *len += 1;
                }
                collision_extents::insert(
                    changes,
                    collision_extents,
                    SharedKey::new(key),
                    Some(Arc::new(value)),
                );
                prior
            }
        }
    }

    pub(crate) fn remove(&mut self, key: &K) -> Option<V> {
        self.retained_charge = None;
        match &mut self.storage {
            PersistentHashMapStorage::Exclusive(values) => values.remove(key),
            PersistentHashMapStorage::ForkShared {
                base,
                changes,
                collision_extents,
                len,
            } => {
                let prior = changes
                    .get(key)
                    .map_or_else(|| base.get(key), |value| value.as_ref().map(Arc::as_ref))
                    .cloned();
                if prior.is_some() {
                    *len -= 1;
                    if base.contains_key(key) {
                        let shared_key = changes
                            .get_key_value(key)
                            .map(|(stored, _)| stored.clone())
                            .unwrap_or_else(|| SharedKey::new(key.clone()));
                        collision_extents::insert(changes, collision_extents, shared_key, None);
                    } else {
                        collision_extents::remove(changes, collision_extents, key);
                    }
                }
                prior
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        self.retained_charge = None;
        self.base_capacity = None;
        self.storage = PersistentHashMapStorage::Exclusive(HashMap::new());
        self.base_capacity = Some(0);
        self.retained_charge = Some(RetainedStorageCharge::ZERO);
    }

    pub(crate) fn entry(&mut self, key: K) -> PersistentHashMapEntry<'_, K, V> {
        PersistentHashMapEntry { map: self, key }
    }

    pub(crate) fn iter(&self) -> PersistentHashMapIter<'_, K, V> {
        match &self.storage {
            PersistentHashMapStorage::Exclusive(values) => {
                PersistentHashMapIter::Exclusive(values.iter())
            }
            PersistentHashMapStorage::ForkShared { base, changes, .. } => {
                PersistentHashMapIter::ForkShared {
                    base: base.iter(),
                    changes: changes.iter(),
                    changed_keys: changes,
                    remaining: self.len(),
                }
            }
        }
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        match (&self.storage, &other.storage) {
            (
                PersistentHashMapStorage::ForkShared {
                    base: left_base,
                    changes: left_changes,
                    ..
                },
                PersistentHashMapStorage::ForkShared {
                    base: right_base,
                    changes: right_changes,
                    ..
                },
            ) => Arc::ptr_eq(left_base, right_base) && left_changes.ptr_eq(right_changes),
            _ => false,
        }
    }

    pub(crate) fn operational_clone(&self) -> Self {
        match &self.storage {
            PersistentHashMapStorage::Exclusive(values) => Self {
                storage: PersistentHashMapStorage::Exclusive(values.clone()),
                base_capacity: self.base_capacity,
                retained_charge: None,
            },
            PersistentHashMapStorage::ForkShared { .. } => self
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        match &self.storage {
            PersistentHashMapStorage::Exclusive(_) => self.operational_clone(),
            PersistentHashMapStorage::ForkShared {
                base,
                changes,
                collision_extents,
                len,
            } => Self {
                storage: PersistentHashMapStorage::ForkShared {
                    base: Arc::clone(base),
                    changes: changes.clone(),
                    collision_extents: collision_extents.clone(),
                    len: *len,
                },
                base_capacity: self.base_capacity,
                retained_charge: self.retained_charge,
            },
        }
    }

    #[cfg(not(test))]
    fn fork_storage_identity(&self) -> Self {
        match &self.storage {
            PersistentHashMapStorage::Exclusive(_) => unreachable!("fork converts storage"),
            PersistentHashMapStorage::ForkShared {
                base,
                changes,
                collision_extents,
                len,
            } => Self {
                storage: PersistentHashMapStorage::ForkShared {
                    base: Arc::clone(base),
                    changes: changes.clone(),
                    collision_extents: collision_extents.clone(),
                    len: *len,
                },
                base_capacity: self.base_capacity,
                retained_charge: self.retained_charge,
            },
        }
    }
}

pub(crate) struct PersistentHashMapEntry<'a, K, V> {
    map: &'a mut PersistentHashMap<K, V>,
    key: K,
}

impl<'a, K, V> PersistentHashMapEntry<'a, K, V>
where
    K: Clone + Eq + Hash,
    V: Clone,
{
    pub(crate) fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        if self.map.get(&self.key).is_none() {
            self.map.insert(self.key.clone(), V::default());
        }
        self.map.get_mut(&self.key).expect("entry must exist")
    }
}
