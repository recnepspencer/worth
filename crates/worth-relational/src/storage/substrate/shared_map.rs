//! Ordered storage index with path-copying updates and borrowed traversal.
use super::{StorageAllocation as Arc, StoredValue};
use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_execution::ExecutionRequest;

mod allocation;
mod allocation_delta;
#[cfg(test)]
mod custody_tests;
mod iteration;
mod node;
mod preallocation;
#[cfg(test)]
mod tests;
mod traits;
mod value_delta;

pub(crate) use iteration::SharedMapIter;
use node::MapNode;

#[derive(Debug, Clone)]
pub(crate) struct SharedMap<K: Ord + Copy, V: StoredValue> {
    root: Option<Arc<MapNode<K, V>>>,
}

impl<K: Ord + Copy, V: StoredValue> SharedMap<K, V> {
    pub(crate) fn new() -> Self {
        Self { root: None }
    }
    /// Cold construction when the owner already has unique ordered keys.
    /// Incremental publication continues to use path-copying `insert`.
    pub(crate) fn from_sorted_unique(values: Vec<(K, V::Value)>) -> Self {
        debug_assert!(
            /* Proof: from_sorted_unique is private native map construction from sorted unique BTreeMap/arena keys; deserialized input is validated by its owner before that conversion. */
            values.windows(2).all(|pair| pair[0].0 < pair[1].0)
        );
        let len = values.len();
        let mut values = values.into_iter();
        Self {
            root: MapNode::from_sorted_unique(&mut values, len, None)
                .expect("ordinary sorted construction has no request refusal"),
        }
    }
    pub(crate) fn len(&self) -> usize {
        self.root.as_ref().map_or(0, |root| root.len)
    }
    /// One fixed-key comparison per node on the selected AVL search path.
    pub(crate) fn navigation_work_bound(&self) -> usize {
        self.root
            .as_ref()
            .map_or(1, |root| root.height.saturating_add(1))
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub(crate) fn get(&self, key: &K) -> Option<&V::Value> {
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            match key.cmp(&node.key) {
                std::cmp::Ordering::Less => current = node.left.as_deref(),
                std::cmp::Ordering::Equal => return Some(node.value.value()),
                std::cmp::Ordering::Greater => current = node.right.as_deref(),
            }
        }
        None
    }

    /// Borrow the first entry at or above a key without allocating traversal
    /// backing. The existing navigation bound covers this single AVL path.
    pub(crate) fn lower_bound(&self, key: &K) -> Option<(&K, &V::Value)> {
        let mut current = self.root.as_deref();
        let mut selected = None;
        while let Some(node) = current {
            match key.cmp(&node.key) {
                std::cmp::Ordering::Less => {
                    selected = Some((&node.key, node.value.value()));
                    current = node.left.as_deref();
                }
                std::cmp::Ordering::Equal => return Some((&node.key, node.value.value())),
                std::cmp::Ordering::Greater => current = node.right.as_deref(),
            }
        }
        selected
    }

    pub(crate) fn get_mut(&mut self, key: &K) -> Option<&mut V::Value> {
        if !self.contains_key(key) {
            return None;
        }
        MapNode::get_mut(&mut self.root, key)
    }

    /// Controlled mutable access is restricted to inline header carriers.
    pub(crate) fn get_mut_for_request(
        &mut self,
        key: &K,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<&mut V>, Cause>
    where
        V: StoredValue<Value = V>,
    {
        request.consult()?;
        if !self.contains_key(key) {
            return Ok(None);
        }
        MapNode::get_mut_for_request(&mut self.root, key, request)
    }

    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub(crate) fn insert(&mut self, key: K, value: V::Value) -> u64 {
        // Replacement never materializes the displaced shared payload.
        let mut copied_bytes = 0;
        MapNode::insert(
            &mut self.root,
            key,
            V::from_value(value),
            &mut copied_bytes,
            None,
        )
        .expect("ordinary insertion has no request refusal");
        copied_bytes
    }

    pub(crate) fn insert_for_request(
        &mut self,
        key: K,
        value: V::Value,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, Cause> {
        request.consult()?;
        let mut copied = 0;
        MapNode::insert(
            &mut self.root,
            key,
            V::from_value(value),
            &mut copied,
            Some(request),
        )?;
        Ok(copied)
    }
    pub(crate) fn remove_for_request(
        &mut self,
        key: &K,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(bool, u64), Cause> {
        request.consult()?;
        if !self.contains_key(key) {
            return Ok((false, 0));
        }
        let mut copied = 0;
        let removed = MapNode::remove(&mut self.root, key, &mut copied, Some(request))?.is_some();
        Ok((removed, copied))
    }

    /// Adopt only the selected source owner; all other destination keys retain
    /// their exact branch-local values. Missing source keys remove membership.
    pub(crate) fn copy_value_from(&mut self, key: K, source: &Self) -> u64 {
        let Some(source) = source.node_for(&key) else {
            return self.remove(&key).1;
        };
        if self
            .node_for(&key)
            .is_some_and(|current| current.value.same_storage(&source.value))
        {
            return 0;
        }
        let mut copied_bytes = 0;
        MapNode::insert(
            &mut self.root,
            key,
            source.value.clone(),
            &mut copied_bytes,
            None,
        )
        .expect("ordinary adoption has no request refusal");
        copied_bytes
    }

    pub(crate) fn remove(&mut self, key: &K) -> (bool, u64) {
        if !self.contains_key(key) {
            return (false, 0);
        }
        let mut copied_bytes = 0;
        let removed = MapNode::remove(&mut self.root, key, &mut copied_bytes, None)
            .expect("ordinary removal has no request refusal")
            .is_some();
        (removed, copied_bytes)
    }

    pub(crate) fn entry(&mut self, key: K) -> SharedMapEntry<'_, K, V> {
        SharedMapEntry { map: self, key }
    }

    pub(crate) fn iter(&self) -> SharedMapIter<'_, K, V> {
        SharedMapIter::new(self.root.as_deref())
    }
    pub(crate) fn try_iter<Stop>(
        &self,
        prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<SharedMapIter<'_, K, V>, Stop> {
        SharedMapIter::try_new(self.root.as_deref(), prepare)
    }
    pub(crate) fn keys(&self) -> impl Iterator<Item = &K> {
        self.iter().map(|(key, _)| key)
    }
    pub(crate) fn values(&self) -> impl Iterator<Item = &V::Value> {
        self.iter().map(|(_, value)| value)
    }

    /// Explicit full-index mutation used by cold checkpoint reconstruction.
    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V::Value)> {
        let mut values = Vec::with_capacity(self.len());
        MapNode::collect_mut(&mut self.root, &mut values);
        values.into_iter()
    }

    pub(crate) fn last_key_value(&self) -> Option<(&K, &V::Value)> {
        let mut node = self.root.as_deref()?;
        while let Some(right) = node.right.as_deref() {
            node = right;
        }
        Some((&node.key, node.value.value()))
    }

    pub(crate) fn allocation_bytes(&self) -> u64 {
        let per_entry = Arc::<MapNode<K, V>>::layout_bytes() + V::allocation_bytes();
        (self.len() as u64).saturating_mul(per_entry)
    }
}

pub(crate) struct SharedMapEntry<'a, K: Ord + Copy, V: StoredValue> {
    map: &'a mut SharedMap<K, V>,
    key: K,
}

impl<'a, K: Ord + Copy, V: StoredValue> SharedMapEntry<'a, K, V> {
    pub(crate) fn or_insert_with(self, value: impl FnOnce() -> V::Value) -> &'a mut V::Value {
        if !self.map.contains_key(&self.key) {
            self.map.insert(self.key, value());
        }
        self.map.get_mut(&self.key).expect(/* Proof: The entry constructor installs this key before returning its exclusive entry handle; the handle exposes no removal operation. */ "entry was installed")
    }
    pub(crate) fn or_insert(self, value: V::Value) -> &'a mut V::Value {
        self.or_insert_with(|| value)
    }
    pub(crate) fn or_default(self) -> &'a mut V::Value
    where
        V::Value: Default,
    {
        self.or_insert_with(V::Value::default)
    }
}

// A map header owns only its actual root handle; copying it never clones keys or values.
impl<K: Ord + Copy, V: StoredValue> StoredValue for SharedMap<K, V> {
    type Value = Self;
    fn from_value(value: Self) -> Self {
        value
    }
    fn value(&self) -> &Self {
        self
    }
    fn value_mut(&mut self) -> &mut Self {
        self
    }
    fn into_value(self) -> Self {
        self
    }
    fn same_storage(&self, other: &Self) -> bool {
        self.root.as_ref().map(Arc::id) == other.root.as_ref().map(Arc::id)
    }
    fn allocation(&self, _: bool) -> Option<super::StorageAllocationObservation> {
        None
    }
    fn allocation_bytes() -> u64 {
        0
    }
}
