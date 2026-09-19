use super::{UiPersistentIndexMutationWork, UiPersistentOrdMap};

/// Immutable ordered membership used by derived successor indexes.
/// Cloning shares the complete predecessor tree; one membership change copies
/// only the AVL search path.
#[derive(Clone)]
pub(crate) struct UiPersistentOrdSet<K> {
    entries: UiPersistentOrdMap<K, ()>,
}

impl<K> Default for UiPersistentOrdSet<K> {
    fn default() -> Self {
        Self {
            entries: UiPersistentOrdMap::default(),
        }
    }
}

impl<K: std::fmt::Debug + Ord + Clone> std::fmt::Debug for UiPersistentOrdSet<K> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_set().entries(self.iter()).finish()
    }
}

impl<K: Ord + Clone + PartialEq> PartialEq for UiPersistentOrdSet<K> {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl<K: Ord + Clone + Eq> Eq for UiPersistentOrdSet<K> {}

impl<K: Ord + Clone> UiPersistentOrdSet<K> {
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn contains_with_probes(&self, value: &K) -> (bool, usize) {
        let (entry, probes) = self.entries.get_with_probes(value);
        (entry.is_some(), probes)
    }

    pub(crate) fn contains(&self, value: &K) -> bool {
        self.contains_with_probes(value).0
    }

    pub(crate) fn remove(&mut self, value: &K) -> bool {
        self.remove_with_work(value).0
    }

    pub(crate) fn clear(&mut self) {
        self.entries = UiPersistentOrdMap::default();
    }

    pub(crate) fn extend(&mut self, values: impl IntoIterator<Item = K>) {
        for value in values {
            self.insert(value);
        }
    }

    pub(crate) fn changed_keys_with_work(
        &self,
        previous: &Self,
    ) -> (Vec<K>, super::UiPersistentMapComparisonWork) {
        self.entries.changed_keys_with_work(&previous.entries)
    }

    pub(crate) fn insert(&mut self, value: K) -> bool {
        self.insert_with_work(value).0
    }

    pub(crate) fn insert_with_work(&mut self, value: K) -> (bool, UiPersistentIndexMutationWork) {
        let (contains, probes) = self.contains_with_probes(&value);
        let mut work = UiPersistentIndexMutationWork::with_key_probes(probes);
        if contains {
            return (false, work);
        }
        work.merge(self.entries.insert_with_work(value, ()))
            .expect("one AVL operation cannot exhaust an address-sized work counter");
        (true, work)
    }

    pub(crate) fn remove_with_work(&mut self, value: &K) -> (bool, UiPersistentIndexMutationWork) {
        self.entries.remove_with_work(value)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|(value, ())| value)
    }

    pub(crate) fn retained_structural_bytes(&self) -> Option<usize> {
        self.entries.retained_structural_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::UiPersistentOrdSet;

    #[test]
    fn membership_changes_are_exact_and_sorted() {
        let mut set = UiPersistentOrdSet::default();
        assert!(set.insert(3));
        assert!(set.insert(1));
        assert!(!set.insert(3));
        assert_eq!(set.iter().copied().collect::<Vec<_>>(), vec![1, 3]);
        assert!(set.remove_with_work(&1).0);
        assert!(!set.remove_with_work(&1).0);
        assert_eq!(set.iter().copied().collect::<Vec<_>>(), vec![3]);
    }
}
