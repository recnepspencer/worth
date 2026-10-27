//! Upper bounds for adopting a shared value without copying its payload.
use super::{ColumnNode, SharedColumn, PAGE_LEN};
use crate::storage::substrate::StorageAllocation;

impl<T: Clone> SharedColumn<T> {
    pub(crate) fn lookup_work_bound(&self) -> u64 {
        self.height as u64 + 1
    }
    /// One selected path plus a possible detached page of shared value handles.
    pub(crate) fn overwrite_work_bound(&self) -> u64 {
        self.height as u64 + 1 + PAGE_LEN as u64
    }
    /// A replacement shares no displaced payload. Its new value owner coexists
    /// with the old column while the selected path/page is detached.
    pub(crate) fn overwrite_allocation_bound(&self, index: usize) -> Option<u64> {
        self.adoption_allocation_bound(index)?
            .checked_add(StorageAllocation::<T>::layout_bytes())
    }

    pub(crate) fn adoption_allocation_bound(&self, index: usize) -> Option<u64> {
        let pages = index.checked_add(1)?.div_ceil(PAGE_LEN).max(1);
        let height = pages.checked_next_power_of_two()?.trailing_zeros() as usize;
        let nodes = self.height.max(height).checked_add(2)?;
        StorageAllocation::<ColumnNode<T>>::layout_bytes()
            .checked_mul(nodes as u64)?
            .checked_add((PAGE_LEN * std::mem::size_of::<Option<StorageAllocation<T>>>()) as u64)
    }
    pub(crate) fn sparse_default_allocation_bytes() -> u64 {
        StorageAllocation::<T>::layout_bytes()
    }
}
