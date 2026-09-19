use std::cmp::Ordering;
use std::fmt;
use std::rc::Rc;

use super::mutation_work::UiPersistentIndexMutationWork;

pub(super) type Link<K, V> = Option<Rc<Node<K, V>>>;

/// Immutable AVL index used by replacement truth that must fork without
/// copying unaffected rows. Updates allocate only the search path.
pub(crate) struct UiPersistentOrdMap<K, V> {
    pub(super) root: Link<K, V>,
}

pub(super) struct Node<K, V> {
    pub(super) key: K,
    pub(super) value: Rc<V>,
    pub(super) left: Link<K, V>,
    pub(super) right: Link<K, V>,
    pub(super) height: u16,
    pub(super) len: usize,
}

impl<K, V> Clone for UiPersistentOrdMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
        }
    }
}

impl<K, V> Default for UiPersistentOrdMap<K, V> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<K, V> UiPersistentOrdMap<K, V> {
    pub(crate) const fn new() -> Self {
        Self { root: None }
    }
}

impl<K: Ord + Clone, V> UiPersistentOrdMap<K, V> {
    pub(crate) fn len(&self) -> usize {
        node_len(&self.root)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub(crate) fn get<Q: Ord + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
    {
        self.get_with_probes(key).0
    }

    pub(crate) fn get_with_probes<Q: Ord + ?Sized>(&self, key: &Q) -> (Option<&V>, usize)
    where
        K: std::borrow::Borrow<Q>,
    {
        let mut cursor = self.root.as_deref();
        let mut probes = 0;
        while let Some(node) = cursor {
            probes += 1;
            match key.cmp(node.key.borrow()) {
                Ordering::Less => cursor = node.left.as_deref(),
                Ordering::Greater => cursor = node.right.as_deref(),
                Ordering::Equal => {
                    #[cfg(test)]
                    super::test_observation::observe_lookup(
                        std::ptr::from_ref(self).cast(),
                        probes,
                    );
                    return (Some(node.value.as_ref()), probes);
                }
            }
        }
        #[cfg(test)]
        super::test_observation::observe_lookup(std::ptr::from_ref(self).cast(), probes);
        (None, probes)
    }

    pub(crate) fn predecessor(&self, key: &K) -> Option<(&K, &V)> {
        let mut cursor = self.root.as_deref();
        let mut predecessor = None;
        while let Some(node) = cursor {
            match key.cmp(&node.key) {
                Ordering::Less | Ordering::Equal => cursor = node.left.as_deref(),
                Ordering::Greater => {
                    predecessor = Some((&node.key, node.value.as_ref()));
                    cursor = node.right.as_deref();
                }
            }
        }
        predecessor
    }

    pub(crate) fn successor(&self, key: &K) -> Option<(&K, &V)> {
        let mut cursor = self.root.as_deref();
        let mut successor = None;
        while let Some(node) = cursor {
            match key.cmp(&node.key) {
                Ordering::Less => {
                    successor = Some((&node.key, node.value.as_ref()));
                    cursor = node.left.as_deref();
                }
                Ordering::Equal | Ordering::Greater => cursor = node.right.as_deref(),
            }
        }
        successor
    }

    pub(crate) fn last_key_value(&self) -> Option<(&K, &V)> {
        let mut cursor = self.root.as_deref()?;
        while let Some(right) = cursor.right.as_deref() {
            cursor = right;
        }
        Some((&cursor.key, cursor.value.as_ref()))
    }

    pub(crate) fn insert(&mut self, key: K, value: V) {
        self.insert_with_work(key, value);
    }

    pub(crate) fn insert_with_work(&mut self, key: K, value: V) -> UiPersistentIndexMutationWork {
        let mut work = UiPersistentIndexMutationWork::default();
        self.root = Some(super::ordered_map_mutation::insert(
            self.root.take(),
            key,
            Rc::new(value),
            &mut work,
        ));
        work
    }

    pub(crate) fn remove(&mut self, key: &K) -> bool {
        self.remove_with_work(key).0
    }

    pub(crate) fn remove_with_work(&mut self, key: &K) -> (bool, UiPersistentIndexMutationWork) {
        let mut work = UiPersistentIndexMutationWork::default();
        let (root, removed) = super::ordered_map_mutation::remove(self.root.take(), key, &mut work);
        self.root = root;
        (removed, work)
    }

    pub(crate) fn iter(&self) -> UiPersistentOrdMapIter<'_, K, V> {
        UiPersistentOrdMapIter::new(
            self.root.as_deref(),
            #[cfg(test)]
            super::test_observation::observes(std::ptr::from_ref(self).cast()),
        )
    }

    pub(crate) fn retained_structural_bytes(&self) -> Option<usize> {
        let value_allocation = std::mem::size_of::<V>()
            .checked_add(2usize.checked_mul(std::mem::size_of::<usize>())?)?;
        let bytes_per_entry = std::mem::size_of::<Node<K, V>>().checked_add(value_allocation)?;
        self.len().checked_mul(bytes_per_entry)
    }

    pub(crate) fn root_is_shared_with(&self, other: &Self) -> bool {
        match (&self.root, &other.root) {
            (Some(left), Some(right)) => Rc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }

    #[cfg(test)]
    pub(crate) fn shared_node_count_with(&self, other: &Self) -> usize {
        let mut predecessor_nodes = std::collections::HashSet::new();
        collect_node_addresses(other.root.as_deref(), &mut predecessor_nodes);
        count_shared_nodes(self.root.as_deref(), &predecessor_nodes)
    }
}

#[cfg(test)]
fn collect_node_addresses<K, V>(
    node: Option<&Node<K, V>>,
    addresses: &mut std::collections::HashSet<*const Node<K, V>>,
) {
    let Some(node) = node else {
        return;
    };
    addresses.insert(std::ptr::from_ref(node));
    collect_node_addresses(node.left.as_deref(), addresses);
    collect_node_addresses(node.right.as_deref(), addresses);
}

#[cfg(test)]
fn count_shared_nodes<K, V>(
    node: Option<&Node<K, V>>,
    addresses: &std::collections::HashSet<*const Node<K, V>>,
) -> usize {
    let Some(node) = node else {
        return 0;
    };
    usize::from(addresses.contains(&std::ptr::from_ref(node)))
        + count_shared_nodes(node.left.as_deref(), addresses)
        + count_shared_nodes(node.right.as_deref(), addresses)
}

pub(super) fn height<K, V>(node: &Link<K, V>) -> u16 {
    node.as_ref().map_or(0, |node| node.height)
}

pub(super) fn node_len<K, V>(node: &Link<K, V>) -> usize {
    node.as_ref().map_or(0, |node| node.len)
}

pub(crate) struct UiPersistentOrdMapIter<'a, K, V> {
    stack: Vec<&'a Node<K, V>>,
    remaining: usize,
    #[cfg(test)]
    observed: bool,
}

impl<'a, K, V> UiPersistentOrdMapIter<'a, K, V> {
    fn new(root: Option<&'a Node<K, V>>, #[cfg(test)] observed: bool) -> Self {
        let mut iter = Self {
            stack: Vec::new(),
            remaining: root.map_or(0, |node| node.len),
            #[cfg(test)]
            observed,
        };
        iter.push_left(root);
        iter
    }

    fn push_left(&mut self, mut node: Option<&'a Node<K, V>>) {
        while let Some(current) = node {
            self.stack.push(current);
            node = current.left.as_deref();
        }
    }
}

impl<'a, K, V> Iterator for UiPersistentOrdMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        #[cfg(test)]
        super::test_observation::observe_iteration(self.observed);
        self.remaining -= 1;
        self.push_left(node.right.as_deref());
        Some((&node.key, node.value.as_ref()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<K, V> ExactSizeIterator for UiPersistentOrdMapIter<'_, K, V> {}

impl<K: fmt::Debug + Ord + Clone, V: fmt::Debug> fmt::Debug for UiPersistentOrdMap<K, V> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Ord + Clone + PartialEq, V: PartialEq> PartialEq for UiPersistentOrdMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<K: Ord + Clone + Eq, V: Eq> Eq for UiPersistentOrdMap<K, V> {}

#[cfg(test)]
mod tests {
    use super::UiPersistentOrdMap;

    #[test]
    fn updates_share_unaffected_tree_and_preserve_sorted_truth() {
        let mut map = UiPersistentOrdMap::default();
        for key in 0..1_000 {
            map.insert(key, key * 2);
        }
        let predecessor = map.clone();
        map.insert(500, 9_999);
        map.remove(&501);

        assert_eq!(predecessor.get(&500), Some(&1_000));
        assert_eq!(map.get(&500), Some(&9_999));
        assert_eq!(map.get(&501), None);
        assert_eq!(predecessor.len(), 1_000);
        assert_eq!(map.len(), 999);
        assert!(map.iter().map(|(key, _)| *key).is_sorted());
        assert!(!map.root_is_shared_with(&predecessor));
    }

    #[test]
    fn cloning_an_unchanged_catalog_is_constant_identity_sharing() {
        let mut map = UiPersistentOrdMap::default();
        map.insert(7, "seven");
        let clone = map.clone();
        assert!(map.root_is_shared_with(&clone));
    }

    #[test]
    fn mixed_local_churn_retains_the_unrelated_persistent_catalog() {
        const ROWS: usize = 4_096;
        let mut map = UiPersistentOrdMap::default();
        for key in 0..ROWS {
            map.insert(key, key * 2);
        }
        let predecessor = map.clone();

        for key in 2_040..2_048 {
            assert!(map.remove(&key));
        }
        for key in 2_048..2_056 {
            map.insert(key, key * 3);
        }
        for key in ROWS..ROWS + 8 {
            map.insert(key, key * 2);
        }

        assert_eq!(predecessor.len(), ROWS);
        assert_eq!(map.len(), ROWS);
        assert!(map.iter().map(|(key, _)| *key).is_sorted());
        assert!(
            map.shared_node_count_with(&predecessor) > ROWS - 128,
            "a bounded local delta must retain almost all unrelated predecessor nodes"
        );
    }

    #[test]
    fn mutation_work_counts_exact_comparisons_and_allocated_nodes() {
        let mut map = UiPersistentOrdMap::default();
        let first = map.insert_with_work(2, "two");
        assert_eq!(first.key_probes(), 0);
        assert_eq!(first.node_copies(), 1);

        map.insert(1, "one");
        map.insert(3, "three");
        let (value, probes) = map.get_with_probes(&3);
        assert_eq!(value, Some(&"three"));
        assert_eq!(probes, 2);

        let (removed, work) = map.remove_with_work(&1);
        assert!(removed);
        assert_eq!(work.key_probes(), 2);
        assert_eq!(work.node_copies(), 1);
    }

    #[test]
    fn last_key_value_reads_the_greatest_entry_without_iteration() {
        let mut map = UiPersistentOrdMap::default();
        for key in [8, 3, 13, 1, 5, 11, 21] {
            map.insert(key, key * 2);
        }
        assert_eq!(map.last_key_value(), Some((&21, &42)));
        assert_eq!(
            UiPersistentOrdMap::<u8, u8>::default().last_key_value(),
            None
        );
    }
}
