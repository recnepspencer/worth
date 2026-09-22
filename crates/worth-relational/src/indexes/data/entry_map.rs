use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// Immutable-generation entries. Cloning shares tree paths and row payloads;
/// changing a bucket copies only its search paths, never unrelated payloads.
/// Serialization remains the ordered map of ordered sequences used by indexes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DerivedIndexEntryMap<K: Ord + Clone, R: Clone>(im::OrdMap<Arc<K>, DerivedIndexRows<R>>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DerivedIndexRows<R>(im::Vector<Arc<R>>);

impl<K: Ord + Clone, R: Clone> Default for DerivedIndexEntryMap<K, R> {
    fn default() -> Self {
        Self(im::OrdMap::new())
    }
}

impl<R> Default for DerivedIndexRows<R> {
    fn default() -> Self {
        Self(im::Vector::new())
    }
}

impl<K: Ord + Clone, R: Clone> DerivedIndexEntryMap<K, R> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, key: &K) -> Option<&DerivedIndexRows<R>> {
        self.0.get(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &DerivedIndexRows<R>)> {
        self.0.iter().map(|(key, rows)| (key.as_ref(), rows))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.keys().map(Arc::as_ref)
    }

    pub fn values(&self) -> impl Iterator<Item = &DerivedIndexRows<R>> {
        self.0.values()
    }

    pub(crate) fn replace(&mut self, key: K, rows: DerivedIndexRows<R>) {
        if rows.is_empty() {
            self.0.remove(&key);
        } else {
            self.0.insert(Arc::new(key), rows);
        }
    }

    #[cfg(test)]
    pub(crate) fn remove(&mut self, key: &K) {
        self.0.remove(key);
    }

    #[cfg(test)]
    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }

    #[cfg(test)]
    pub(crate) fn get_mut(&mut self, key: &K) -> Option<&mut DerivedIndexRows<R>> {
        self.0.get_mut(key)
    }
}

impl<R> DerivedIndexRows<R> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&R> {
        self.0.get(index).map(Arc::as_ref)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &R> + DoubleEndedIterator {
        self.0.iter().map(Arc::as_ref)
    }

    pub fn binary_search_by(
        &self,
        mut compare: impl FnMut(&R) -> std::cmp::Ordering,
    ) -> Result<usize, usize> {
        self.0.binary_search_by(|row| compare(row))
    }

    pub(crate) fn insert(&mut self, index: usize, row: R) {
        self.0.insert(index, Arc::new(row));
    }

    pub(crate) fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }
}

impl<K: Ord + Clone, R: Clone> From<BTreeMap<K, Vec<R>>> for DerivedIndexEntryMap<K, R> {
    fn from(entries: BTreeMap<K, Vec<R>>) -> Self {
        Self(
            entries
                .into_iter()
                .map(|(key, rows)| (Arc::new(key), DerivedIndexRows::from(rows)))
                .collect(),
        )
    }
}

impl<R> From<Vec<R>> for DerivedIndexRows<R> {
    fn from(rows: Vec<R>) -> Self {
        Self(rows.into_iter().map(Arc::new).collect())
    }
}

impl<'a, R> IntoIterator for &'a DerivedIndexRows<R> {
    type Item = &'a R;
    type IntoIter = std::iter::Map<im::vector::Iter<'a, Arc<R>>, fn(&'a Arc<R>) -> &'a R>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().map(Arc::as_ref)
    }
}

#[cfg(test)]
impl<R> std::ops::Index<usize> for DerivedIndexRows<R> {
    type Output = R;
    fn index(&self, index: usize) -> &R {
        &self.0[index]
    }
}

#[cfg(test)]
impl<R: Clone> std::ops::IndexMut<usize> for DerivedIndexRows<R> {
    fn index_mut(&mut self, index: usize) -> &mut R {
        Arc::make_mut(&mut self.0[index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistent_entries_preserve_canonical_map_sequence_wire_shape() {
        let legacy = BTreeMap::from([(2_u64, vec![3_u64, 5]), (7, vec![11])]);
        let persistent = DerivedIndexEntryMap::from(legacy.clone());
        let legacy_bytes = rmp_serde::to_vec_named(&legacy).unwrap();
        let persistent_bytes = rmp_serde::to_vec_named(&persistent).unwrap();
        assert_eq!(persistent_bytes, legacy_bytes);
        let restored: DerivedIndexEntryMap<u64, u64> =
            rmp_serde::from_slice(&legacy_bytes).unwrap();
        assert_eq!(restored, persistent);
    }
}
