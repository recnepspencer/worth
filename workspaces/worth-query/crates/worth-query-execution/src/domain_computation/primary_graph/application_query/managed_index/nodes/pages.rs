//! Immutable node pages; isolated editors freeze each touched page once.
use super::{Stop, WorthQueryManagedIndexControl};
use std::{
    alloc::Layout,
    cell::{Cell, RefCell},
    sync::Arc,
};
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
};

pub(super) const PAGE_BITS: u32 = 5;
pub(super) const PAGE_SIZE: usize = 1 << PAGE_BITS;

mod edit;
pub(in super::super::super) mod inspection;
pub(in super::super) use edit::PageEdit;

pub(in super::super) struct NodePages<Node> {
    root: Option<Arc<Directory<Node>>>,
}

struct Directory<Node> {
    prefix: usize,
    height: u32,
    entry: Entry<Node>,
    _memory: Option<ExecutionMemoryReservation>,
}
enum Entry<Node> {
    Page(ExecutionArray<Option<Node>>),
    Branch {
        left: Arc<Directory<Node>>,
        right: Arc<Directory<Node>>,
    },
}

impl<Node> Default for NodePages<Node> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<Node> Clone for NodePages<Node> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
        }
    }
}

impl<Node: Clone> NodePages<Node> {
    pub(in super::super) fn from_scratch(
        execution: &ExecutionRequest<'_, '_>,
        scratch: &[RefCell<Option<Node>>],
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self, Stop> {
        let page_count = scratch.len().div_ceil(PAGE_SIZE);
        let height = page_count
            .checked_next_power_of_two()
            .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?
            .trailing_zeros();
        let root = build(execution, scratch, 0, height, control)?;
        Ok(Self { root })
    }

    pub(in super::super) fn get(
        &self,
        ordinal: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<&Node>, Stop> {
        Ok(self
            .page_values(ordinal >> PAGE_BITS, control)?
            .and_then(|values| values[ordinal & (PAGE_SIZE - 1)].as_ref()))
    }

    fn page_values(
        &self,
        page: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<&ExecutionArray<Option<Node>>>, Stop> {
        let mut current = self.root.as_deref();
        while let Some(directory) = current {
            control.checkpoint(1)?;
            if page >> directory.height != directory.prefix >> directory.height {
                return Ok(None);
            }
            match &directory.entry {
                Entry::Page(values) => return Ok(Some(values)),
                Entry::Branch { left, right } => {
                    current = Some(if page & (1 << (directory.height - 1)) == 0 {
                        left.as_ref()
                    } else {
                        right.as_ref()
                    });
                }
            }
        }
        Ok(None)
    }
}

fn retain<Node>(
    execution: &ExecutionRequest<'_, '_>,
    entry: Entry<Node>,
    prefix: usize,
    height: u32,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Arc<Directory<Node>>, Stop> {
    let memory = header_memory::<Directory<Node>>(execution, control)?;
    Ok(Arc::new(Directory {
        prefix,
        height,
        entry,
        _memory: memory,
    }))
}

fn header_memory<T>(
    execution: &ExecutionRequest<'_, '_>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Option<ExecutionMemoryReservation>, Stop> {
    control.checkpoint(1)?;
    let bytes = Layout::new::<T>()
        .extend(Layout::new::<[usize; 2]>())
        .map_err(|_| control.refuse(Stop::CapacityOverflow))?
        .0
        .pad_to_align()
        .size();
    let bytes = u64::try_from(bytes).map_err(|_| control.refuse(Stop::CapacityOverflow))?;
    let memory = execution
        .reserve_retained_memory(bytes)
        .map_err(|cause| control.refuse(Stop::Request(cause)))?;
    Ok(memory)
}

fn build<Node: Clone>(
    execution: &ExecutionRequest<'_, '_>,
    scratch: &[RefCell<Option<Node>>],
    start: usize,
    height: u32,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Option<Arc<Directory<Node>>>, Stop> {
    control.checkpoint(1)?;
    if start >= scratch.len().div_ceil(PAGE_SIZE) {
        return Ok(None);
    }
    if height == 0 {
        return page(
            execution,
            start,
            |index| {
                scratch
                    .get(start * PAGE_SIZE + index)
                    .and_then(|cell| cell.borrow_mut().take())
            },
            control,
        );
    }
    let left = build(execution, scratch, start, height - 1, control)?;
    let right = build(
        execution,
        scratch,
        start + (1 << (height - 1)),
        height - 1,
        control,
    )?;
    branch(execution, left, right, control)
}

fn branch<Node>(
    execution: &ExecutionRequest<'_, '_>,
    left: Option<Arc<Directory<Node>>>,
    right: Option<Arc<Directory<Node>>>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Option<Arc<Directory<Node>>>, Stop> {
    match (left, right) {
        (Some(left), Some(right)) => {
            let height = usize::BITS - (left.prefix ^ right.prefix).leading_zeros();
            debug_assert!(left.height < height && right.height < height);
            // Page keys come from ordinal>>PAGE_BITS, so height is below word width.
            let prefix = left.prefix & (usize::MAX << height);
            retain(
                execution,
                Entry::Branch { left, right },
                prefix,
                height,
                control,
            )
            .map(Some)
        }
        (child, None) | (None, child) => Ok(child),
    }
}

fn page<Node: Clone>(
    execution: &ExecutionRequest<'_, '_>,
    key: usize,
    value: impl Fn(usize) -> Option<Node>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Option<Arc<Directory<Node>>>, Stop> {
    let mut values = ExecutionRequestArrayBuilder::allocate(*execution, PAGE_SIZE)
        .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
    let mut occupied = false;
    for index in 0..PAGE_SIZE {
        control.checkpoint(1)?;
        let value = value(index);
        occupied |= value.is_some();
        values
            .push(value)
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
    }
    let values = values
        .seal()
        .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
    if occupied {
        retain(execution, Entry::Page(values), key, 0, control).map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests;
