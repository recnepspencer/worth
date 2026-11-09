//! Persistent pages of explicit shallow element carriers.
use super::{StorageAllocation as Arc, StoredValue};
use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_execution::ExecutionRequest;
mod allocation;
mod allocation_delta;
mod bulk_bootstrap;
#[cfg(test)]
mod custody_tests;
mod iteration;
mod iteration_observation;
pub use iteration_observation::observe_history_prefix_traversal;
mod node;
mod preallocation;
#[cfg(test)]
mod tests;
mod traits;
mod value_delta;
pub(crate) use iteration::SharedColumnIter;
use node::ColumnNode;
const PAGE_LEN: usize = 32;

#[derive(Debug, Clone)]
pub(crate) struct SharedColumn<T: StoredValue> {
    root: Option<Arc<ColumnNode<T>>>,
    height: usize,
    len: usize,
    default: Option<T>,
}
impl<T: StoredValue> SharedColumn<T> {
    pub(crate) fn with_capacity(_capacity: usize) -> Self {
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
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub(crate) fn partition_point(&self, mut predicate: impl FnMut(&T::Value) -> bool) -> usize {
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
    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&T::Value) -> bool) {
        let Some(first_removed) = self.iter().position(|value| !keep(value)) else {
            return;
        };
        let mut retained = Self::default();
        for index in 0..first_removed {
            retained.push_shared(
                self.get_shared(index)
                    .expect("selected original carrier")
                    .clone(),
            );
        }
        for index in first_removed + 1..self.len {
            if keep(&self[index]) {
                retained.push(self[index].clone());
            }
        }
        *self = retained;
    }
    pub(crate) fn with_default(len: usize, value: T::Value) -> Self {
        Self::with_default_carrier(len, T::from_value(value))
    }
    pub(crate) fn with_default_carrier(len: usize, value: T) -> Self {
        let pages = len.div_ceil(PAGE_LEN).max(1);
        Self {
            root: None,
            height: pages.next_power_of_two().trailing_zeros() as usize,
            len,
            default: Some(value),
        }
    }
    pub(crate) fn get(&self, index: usize) -> Option<&T::Value> {
        self.get_shared(index).map(T::value)
    }
    pub(in crate::storage) fn retained_value(&self, index: usize) -> Option<T> {
        self.get_shared(index).cloned()
    }
    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut T::Value> {
        self.carrier_mut_using(index, None)
            .expect("ordinary node construction has no request refusal")
            .map(T::value_mut)
    }
    pub(crate) fn carrier_mut_for_request(
        &mut self,
        index: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<&mut T>, Cause> {
        self.carrier_mut_using(index, Some(request))
    }
    pub(in crate::storage::substrate) fn carrier_mut_using(
        &mut self,
        index: usize,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<Option<&mut T>, Cause> {
        if index >= self.len {
            return Ok(None);
        }
        if self
            .root
            .as_ref()
            .and_then(|root| root.get(index, self.height))
            .is_none()
        {
            let value = self
                .default
                .as_ref()
                .expect("sparse column retains a default")
                .clone();
            self.set_shared_using(index, value, request)?;
        }
        Ok(Some(ColumnNode::carrier_mut(
            self.root.as_mut().expect("materialized root"),
            index,
            self.height,
            &mut 0,
            request,
        )?))
    }
    pub(in crate::storage::substrate) fn push_value_using(
        &mut self,
        value: T::Value,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<u64, Cause> {
        if let Some(request) = request {
            request.consult()?;
        }
        self.push_using(T::from_value(value), request)
    }
    pub(in crate::storage::substrate) fn set_value_using(
        &mut self,
        index: usize,
        value: T::Value,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<(), Cause> {
        if let Some(request) = request {
            request.consult()?;
        }
        assert!(index < self.len, "column index out of bounds");
        self.set_shared_using(index, T::from_value(value), request)?;
        Ok(())
    }
    pub(crate) fn push(&mut self, value: T::Value) -> u64 {
        self.push_shared(T::from_value(value))
    }
    pub(in crate::storage) fn push_shared(&mut self, value: T) -> u64 {
        self.push_using(value, None)
            .expect("ordinary node construction has no request refusal")
    }
    pub(crate) fn push_for_request(
        &mut self,
        value: T::Value,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, Cause> {
        request.consult()?;
        self.push_using(T::from_value(value), Some(request))
    }
    pub(in crate::storage) fn push_shared_for_request(
        &mut self,
        value: T,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, Cause> {
        self.push_using(value, Some(request))
    }
    fn push_using(
        &mut self,
        value: T,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<u64, Cause> {
        let next = self.len.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
        if self.len / PAGE_LEN == 1usize << self.height {
            self.root = Some(ColumnNode::allocated_branch(self.root.clone(), request)?);
            self.height += 1;
        }
        let copied = self.set_shared_using(self.len, value, request)?;
        self.len = next;
        Ok(copied)
    }
    pub(crate) fn copy_value_from(
        &mut self,
        index: usize,
        source: &Self,
        source_index: usize,
    ) -> u64 {
        let value = source
            .get_shared(source_index)
            .expect("selected source carrier")
            .clone();
        if self
            .get_shared(index)
            .is_some_and(|current| current.same_storage(&value))
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
    fn get_shared(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        self.root
            .as_ref()
            .and_then(|root| root.get(index, self.height))
            .or(self.default.as_ref())
    }
    pub(in crate::storage) fn set_shared(&mut self, index: usize, value: T) -> u64 {
        self.set_shared_using(index, value, None)
            .expect("ordinary node construction has no request refusal")
    }
    pub(in crate::storage) fn set_shared_for_request(
        &mut self,
        index: usize,
        value: T,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, Cause> {
        self.set_shared_using(index, value, Some(request))
    }
    pub(in crate::storage::substrate) fn set_shared_using(
        &mut self,
        index: usize,
        value: T,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<u64, Cause> {
        if self.root.is_none() {
            self.root = Some(ColumnNode::allocate(self.height, request)?);
        }
        let mut copied = 0;
        ColumnNode::detach(
            self.root.as_mut().expect("installed root"),
            &mut copied,
            request,
        )?
        .set(index, self.height, value, &mut copied, request)?;
        Ok(copied)
    }
    pub(crate) fn set(&mut self, index: usize, value: T::Value) {
        assert!(index < self.len, "column index out of bounds");
        self.set_shared(index, T::from_value(value));
    }
    pub(crate) fn set_for_request(
        &mut self,
        index: usize,
        value: T::Value,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        request.consult()?;
        assert!(index < self.len, "column index out of bounds");
        self.set_shared_using(index, T::from_value(value), Some(request))?;
        Ok(())
    }
    pub(crate) fn iter(&self) -> SharedColumnIter<'_, T> {
        SharedColumnIter::new(
            self.root.as_deref(),
            self.height,
            self.len,
            self.default.as_ref().map(T::value),
        )
    }
    pub(crate) fn to_vec(&self) -> Vec<T::Value> {
        self.iter().cloned().collect()
    }
    pub(crate) fn into_vec(self) -> Vec<T::Value> {
        if self.default.is_some() {
            return self.to_vec();
        }
        let mut values = Vec::with_capacity(self.len);
        if let Some(root) = self.root {
            ColumnNode::into_values(root, &mut values);
        }
        values
    }
    pub(crate) fn allocation_bytes(&self) -> u64 {
        let default = self.default.as_ref().map_or(0, |_| T::allocation_bytes());
        self.root.as_ref().map_or(default, |root| {
            (root.node_count as u64)
                .saturating_mul(Arc::<ColumnNode<T>>::layout_bytes())
                .saturating_add(
                    (root.page_count as u64)
                        .saturating_mul((PAGE_LEN * std::mem::size_of::<Option<T>>()) as u64),
                )
                .saturating_add((root.value_count as u64).saturating_mul(T::allocation_bytes()))
                .saturating_add(default)
        })
    }
}
impl<T: StoredValue> StoredValue for SharedColumn<T> {
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
        self.len == other.len
            && self.height == other.height
            && self.root.as_ref().map(Arc::id) == other.root.as_ref().map(Arc::id)
            && match (&self.default, &other.default) {
                (None, None) => true,
                (Some(a), Some(b)) => a.same_storage(b),
                _ => false,
            }
    }
    fn allocation(&self, _: bool) -> Option<super::StorageAllocationObservation> {
        None
    }
    fn allocation_bytes() -> u64 {
        0
    }
}
