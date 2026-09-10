mod fork_growth;
mod persistent_fork;
mod removal;
mod staging_charge;
use crate::data::retained_storage::RetainedStorageBacking;
use std::collections::BTreeMap;
use std::sync::Arc;

#[path = "persistent_ord_map/entry_handle.rs"]
mod entry_handle;
#[path = "persistent_ord_map/fork_overlay.rs"]
mod fork_overlay;
#[path = "persistent_ord_map/iteration.rs"]
mod iteration;
#[path = "persistent_ord_map/serialization.rs"]
mod serialization;
#[path = "persistent_ord_map/traits.rs"]
mod traits;

#[path = "persistent_ord_map/charge_updates.rs"]
mod charge_updates;
#[path = "persistent_ord_map/retained_charge.rs"]
mod retained_charge;
#[cfg(test)]
#[path = "persistent_ord_map/retained_charge_tests.rs"]
mod retained_charge_tests;

use crate::data::retained_storage::RetainedStorageCharge;
pub(crate) use charge_updates::{RetainedMapMutationDenial, RetainedMapMutationOutcome};
use entry_handle::SharedKey;
use fork_overlay::{base_value_if_live, record_base_readmission, record_base_retirement};
use iteration::{LiveBaseIter, PersistentOrdMapIter};

enum PersistentOrdMapStorage<K, V> {
    Exclusive(BTreeMap<K, V>),
    ForkShared {
        base: Arc<RetainedStorageBacking<BTreeMap<K, V>>>,
        changes: im::OrdMap<SharedKey<K>, Arc<V>>,
        retired_base_intervals: im::OrdMap<SharedKey<K>, SharedKey<K>>,
        len: usize,
    },
}

/// An ordered map with flat ordinary storage and per-key fork overlays.
pub(crate) struct PersistentOrdMap<K: Clone + Ord, V: Clone> {
    storage: PersistentOrdMapStorage<K, V>,
    retained_charge: Option<RetainedStorageCharge>,
}

impl<K: Clone + Ord, V: Clone> PersistentOrdMap<K, V> {
    pub(crate) fn new() -> Self {
        Self {
            storage: PersistentOrdMapStorage::Exclusive(BTreeMap::new()),
            retained_charge: crate::data::retained_storage::btree_structure_charge::<K, V>(0).ok(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        match &self.storage {
            PersistentOrdMapStorage::Exclusive(values) => values.len(),
            PersistentOrdMapStorage::ForkShared { len, .. } => *len,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Structural comparisons/navigation in `get`, including both retirement
    /// searches and its final endpoint comparison. Does not bound generic Ord.
    pub(crate) fn lookup_steps(&self) -> usize {
        use crate::data::retained_storage::ordered_lookup_steps as steps;
        match &self.storage {
            PersistentOrdMapStorage::Exclusive(values) => steps(values.len()),
            PersistentOrdMapStorage::ForkShared {
                base,
                changes,
                retired_base_intervals,
                ..
            } => {
                steps(base.len())
                    + steps(changes.len())
                    + 2 * steps(retired_base_intervals.len())
                    + 1
            }
        }
    }

    pub(crate) fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match &self.storage {
            PersistentOrdMapStorage::Exclusive(values) => values.get(key),
            PersistentOrdMapStorage::ForkShared {
                base,
                changes,
                retired_base_intervals,
                ..
            } => entry_handle::get(changes, key)
                .map(Arc::as_ref)
                .or_else(|| base_value_if_live(base, retired_base_intervals, key)),
        }
    }

    pub(crate) fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.retained_charge = None;
        match &mut self.storage {
            PersistentOrdMapStorage::Exclusive(values) => values.get_mut(key),
            PersistentOrdMapStorage::ForkShared {
                base,
                changes,
                retired_base_intervals,
                ..
            } => {
                if entry_handle::get(changes, key).is_none() {
                    let value =
                        Arc::new(base_value_if_live(base, retired_base_intervals, key)?.clone());
                    let shared_key = SharedKey::new(key.clone());
                    changes.insert(shared_key, value);
                }
                entry_handle::get_mut(changes, key).map(Arc::make_mut)
            }
        }
    }

    pub(crate) fn first_key_value(&self) -> Option<(&K, &V)> {
        match &self.storage {
            PersistentOrdMapStorage::Exclusive(values) => values.first_key_value(),
            PersistentOrdMapStorage::ForkShared { .. } => self.iter().next(),
        }
    }

    pub(crate) fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get(key).is_some()
    }

    pub(crate) fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.retained_charge = None;
        match &mut self.storage {
            PersistentOrdMapStorage::Exclusive(values) => values.insert(key, value),
            PersistentOrdMapStorage::ForkShared {
                base,
                changes,
                retired_base_intervals,
                len,
            } => {
                let previous = entry_handle::get(changes, &key)
                    .map(|value| value.as_ref())
                    .or_else(|| base_value_if_live(base, retired_base_intervals, &key))
                    .cloned();
                if previous.is_none() {
                    *len += 1;
                }
                if base.contains_key(&key) && previous.is_none() {
                    record_base_readmission(base, retired_base_intervals, &key);
                }
                let shared_key = SharedKey::new(key);
                changes.insert(shared_key, Arc::new(value));
                previous
            }
        }
    }

    pub(crate) fn remove(&mut self, key: &K) -> Option<V> {
        self.retained_charge = None;
        match &mut self.storage {
            PersistentOrdMapStorage::Exclusive(values) => values.remove(key),
            PersistentOrdMapStorage::ForkShared {
                base,
                changes,
                retired_base_intervals,
                len,
            } => {
                let previous = entry_handle::get(changes, key)
                    .map(|value| value.as_ref())
                    .or_else(|| base_value_if_live(base, retired_base_intervals, key))
                    .cloned();
                if previous.is_some() {
                    *len -= 1;
                    entry_handle::remove(changes, key);
                    if base.contains_key(key) {
                        record_base_retirement(base, retired_base_intervals, key);
                    }
                }
                previous
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        self.retained_charge = None;
        self.storage = PersistentOrdMapStorage::Exclusive(BTreeMap::new());
        self.retained_charge =
            crate::data::retained_storage::btree_structure_charge::<K, V>(0).ok();
    }

    pub(crate) fn iter(&self) -> PersistentOrdMapIter<'_, K, V> {
        match &self.storage {
            PersistentOrdMapStorage::Exclusive(values) => {
                PersistentOrdMapIter::Exclusive(values.iter())
            }
            PersistentOrdMapStorage::ForkShared {
                base,
                changes,
                retired_base_intervals,
                ..
            } => PersistentOrdMapIter::ForkShared {
                base: LiveBaseIter::new(base, retired_base_intervals).peekable(),
                changes: changes.iter().peekable(),
                remaining: self.len(),
            },
        }
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(key, _)| key)
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &V> {
        self.iter().map(|(_, value)| value)
    }

    pub(crate) fn entry(&mut self, key: K) -> PersistentOrdMapEntry<'_, K, V> {
        PersistentOrdMapEntry { map: self, key }
    }

    pub(crate) fn operational_clone(&self) -> Self {
        // Empty materialization has the constructor's exact charge, even when
        // the source retained retired backing. No payload is cloned or measured.
        if self.is_empty() {
            return Self::new();
        }
        match &self.storage {
            PersistentOrdMapStorage::Exclusive(values) => Self {
                storage: PersistentOrdMapStorage::Exclusive(values.clone()),
                retained_charge: None,
            },
            PersistentOrdMapStorage::ForkShared { .. } => self
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        }
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        match (&self.storage, &other.storage) {
            (
                PersistentOrdMapStorage::ForkShared {
                    base: left_base,
                    changes: left_changes,
                    retired_base_intervals: left_retired_base_intervals,
                    ..
                },
                PersistentOrdMapStorage::ForkShared {
                    base: right_base,
                    changes: right_changes,
                    retired_base_intervals: right_retired_base_intervals,
                    ..
                },
            ) => {
                Arc::ptr_eq(left_base, right_base)
                    && left_changes.ptr_eq(right_changes)
                    && left_retired_base_intervals.ptr_eq(right_retired_base_intervals)
            }
            _ => false,
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        match &self.storage {
            PersistentOrdMapStorage::Exclusive(_) => self.operational_clone(),
            PersistentOrdMapStorage::ForkShared {
                base,
                changes,
                retired_base_intervals,
                len,
            } => Self {
                storage: PersistentOrdMapStorage::ForkShared {
                    base: Arc::clone(base),
                    changes: changes.clone(),
                    retired_base_intervals: retired_base_intervals.clone(),
                    len: *len,
                },
                retained_charge: self.retained_charge,
            },
        }
    }

    #[cfg(not(test))]
    fn fork_storage_identity(&self) -> Self {
        match &self.storage {
            PersistentOrdMapStorage::Exclusive(_) => unreachable!("fork converts storage"),
            PersistentOrdMapStorage::ForkShared {
                base,
                changes,
                retired_base_intervals,
                len,
            } => Self {
                storage: PersistentOrdMapStorage::ForkShared {
                    base: Arc::clone(base),
                    changes: changes.clone(),
                    retired_base_intervals: retired_base_intervals.clone(),
                    len: *len,
                },
                retained_charge: self.retained_charge,
            },
        }
    }
}

pub(crate) struct PersistentOrdMapEntry<'a, K: Clone + Ord, V: Clone> {
    map: &'a mut PersistentOrdMap<K, V>,
    key: K,
}

impl<'a, K: Clone + Ord, V: Clone> PersistentOrdMapEntry<'a, K, V> {
    pub(crate) fn or_insert(self, value: V) -> &'a mut V {
        self.or_insert_with(|| value)
    }

    pub(crate) fn or_insert_with(self, make: impl FnOnce() -> V) -> &'a mut V {
        if !self.map.contains_key(&self.key) {
            self.map.insert(self.key.clone(), make());
        }
        self.map.get_mut(&self.key).expect("entry must exist")
    }

    pub(crate) fn and_modify(self, modify: impl FnOnce(&mut V)) -> Self {
        if let Some(value) = self.map.get_mut(&self.key) {
            modify(value);
        }
        self
    }
}

impl<'a, K: Clone + Ord, V: Clone + Default> PersistentOrdMapEntry<'a, K, V> {
    pub(crate) fn or_default(self) -> &'a mut V {
        self.or_insert_with(V::default)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "persistent_ord_map/fork_granule_tests.rs"]
mod fork_granule_tests;
