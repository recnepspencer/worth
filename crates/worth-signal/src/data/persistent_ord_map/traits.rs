use std::fmt;

use super::{PersistentOrdMap, PersistentOrdMapIter, PersistentOrdMapStorage};

impl<K: Clone + Ord, V: Clone> Clone for PersistentOrdMap<K, V> {
    fn clone(&self) -> Self {
        match &self.storage {
            PersistentOrdMapStorage::Exclusive(_) => self.operational_clone(),
            PersistentOrdMapStorage::ForkShared {
                base,
                changes,
                retired_base_intervals,
                len,
            } => Self {
                storage: PersistentOrdMapStorage::ForkShared {
                    base: std::sync::Arc::clone(base),
                    changes: changes.clone(),
                    retired_base_intervals: retired_base_intervals.clone(),
                    len: *len,
                },
                retained_charge: self.retained_charge,
            },
        }
    }
}

impl<K: Clone + Ord, V: Clone> Default for PersistentOrdMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Clone + Ord, V: Clone> FromIterator<(K, V)> for PersistentOrdMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self {
            storage: PersistentOrdMapStorage::Exclusive(iter.into_iter().collect()),
            retained_charge: None,
        }
    }
}

impl<K: Clone + Ord + fmt::Debug, V: Clone + fmt::Debug> fmt::Debug for PersistentOrdMap<K, V> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Clone + Ord + PartialEq, V: Clone + PartialEq> PartialEq for PersistentOrdMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<K: Clone + Ord + Eq, V: Clone + Eq> Eq for PersistentOrdMap<K, V> {}

impl<'a, K: Clone + Ord, V: Clone> IntoIterator for &'a PersistentOrdMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = PersistentOrdMapIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
