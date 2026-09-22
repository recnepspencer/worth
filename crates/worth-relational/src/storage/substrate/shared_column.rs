//! Persistent structure-of-arrays column. A write detaches an index path and
//! one value, never the other records in a retained partition.

use super::StorageAllocation as Arc;

mod allocation;
mod allocation_delta;
mod iteration;
mod node;
#[cfg(test)]
mod tests;
mod traits;
mod value_delta;

pub(crate) use iteration::SharedColumnIter;
use node::ColumnNode;

const PAGE_LEN: usize = 32;

#[derive(Debug, Clone)]
pub(crate) struct SharedColumn<T: Clone> {
    root: Option<Arc<ColumnNode<T>>>,
    height: usize,
    len: usize,
    default: Option<Arc<T>>,
}

impl<T: Clone> SharedColumn<T> {
    pub(crate) fn with_capacity(_capacity: usize) -> Self {
        // Pages are allocated only as records are installed. A reserved logical
        // shape must not allocate proportional to a sparse record address.
        Self {
            root: None,
            height: 0,
            len: 0,
            default: None,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    #[cfg(test)]
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn last_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.len.checked_sub(1)?)
    }

    pub(crate) fn partition_point(&self, mut predicate: impl FnMut(&T) -> bool) -> usize {
        let (mut low, mut high) = (0, self.len);
        while low < high {
            let middle = low + (high - low) / 2;
            if predicate(&self[middle]) {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        low
    }

    /// Explicit retention compaction. Shifted values receive new allocations;
    /// unchanged prefixes preserve their owners and positional identity.
    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        let Some(first_removed) = self.iter().position(|value| !keep(value)) else {
            return;
        };
        let mut retained = Self::default();
        for index in 0..first_removed {
            retained.push_shared(self.get_shared(index).unwrap().clone());
        }
        for index in first_removed + 1..self.len {
            if keep(&self[index]) {
                retained.push(self[index].clone());
            }
        }
        *self = retained;
    }

    /// A logical overlay shape with one shared empty value and no materialized pages.
    pub(crate) fn with_default(len: usize, value: T) -> Self {
        let pages = len.div_ceil(PAGE_LEN).max(1);
        Self {
            root: None,
            height: pages.next_power_of_two().trailing_zeros() as usize,
            len,
            default: Some(Arc::new(value)),
        }
    }

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        self.get_shared(index).map(Arc::as_ref)
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        if self
            .root
            .as_ref()
            .and_then(|root| root.get(index, self.height))
            .is_none()
        {
            self.set_shared(
                index,
                self.default
                    .as_ref()
                    .expect("sparse column default")
                    .clone(),
            );
        }
        Some(Arc::make_mut(self.root.as_mut().unwrap()).get_mut(index, self.height))
    }

    pub(crate) fn push(&mut self, value: T) -> u64 {
        self.push_shared(Arc::new(value))
    }

    fn push_shared(&mut self, value: Arc<T>) -> u64 {
        let next_len = self.len.checked_add(1).expect("column length exhausted");
        if self.len / PAGE_LEN == 1usize << self.height {
            self.root = Some(Arc::new(ColumnNode::branch(self.root.take(), None)));
            self.height += 1;
        }
        let copied_bytes = self.set_shared(self.len, value);
        self.len = next_len;
        copied_bytes
    }

    /// Publish a selected value without copying its payload or the displaced one.
    pub(crate) fn copy_value_from(
        &mut self,
        index: usize,
        source: &Self,
        source_index: usize,
    ) -> u64 {
        let value = source
            .get_shared(source_index)
            .expect("source column index out of bounds")
            .clone();
        if self
            .get_shared(index)
            .is_some_and(|current| current.id() == value.id())
        {
            return 0;
        }
        if index == self.len {
            self.push_shared(value)
        } else {
            assert!(index < self.len, "column index out of bounds");
            self.set_shared(index, value)
        }
    }

    fn get_shared(&self, index: usize) -> Option<&Arc<T>> {
        if index >= self.len {
            return None;
        }
        self.root
            .as_ref()
            .and_then(|root| root.get(index, self.height))
            .or(self.default.as_ref())
    }

    fn set_shared(&mut self, index: usize, value: Arc<T>) -> u64 {
        let root = self
            .root
            .get_or_insert_with(|| Arc::new(ColumnNode::empty(self.height)));
        let mut copied_bytes = 0;
        ColumnNode::detach(root, &mut copied_bytes).set(
            index,
            self.height,
            value,
            &mut copied_bytes,
        );
        copied_bytes
    }

    /// Replace without detaching or cloning the displaced value.
    pub(crate) fn set(&mut self, index: usize, value: T) {
        assert!(index < self.len, "column index out of bounds");
        self.set_shared(index, Arc::new(value));
    }

    pub(crate) fn iter(&self) -> SharedColumnIter<'_, T> {
        SharedColumnIter::new(
            self.root.as_deref(),
            self.height,
            self.len,
            self.default.as_deref(),
        )
    }

    pub(crate) fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    /// Explicit whole-column materialization for checkpoint/export consumers.
    pub(crate) fn into_vec(self) -> Vec<T> {
        if self.default.is_some() {
            return self.to_vec();
        }
        let mut values = Vec::with_capacity(self.len);
        if let Some(root) = self.root {
            ColumnNode::into_values(root, &mut values);
        }
        values
    }

    /// Backing allocations, excluding the separately measured owned payloads
    /// inside T. The column object itself is inline in its owning arena.
    pub(crate) fn allocation_bytes(&self) -> u64 {
        let default_bytes = self
            .default
            .as_ref()
            .map_or(0, |_| Arc::<T>::layout_bytes());
        self.root.as_ref().map_or(default_bytes, |root| {
            let node_bytes = Arc::<ColumnNode<T>>::layout_bytes();
            let value_bytes = Arc::<T>::layout_bytes();
            (root.node_count as u64)
                .saturating_mul(node_bytes as u64)
                .saturating_add(
                    (root.page_count as u64)
                        .saturating_mul((PAGE_LEN * std::mem::size_of::<Arc<T>>()) as u64),
                )
                .saturating_add((root.value_count as u64).saturating_mul(value_bytes as u64))
                .saturating_add(default_bytes)
        })
    }
}
