//! Immutable node pages; isolated editors freeze each touched page once.
use super::{Stop, WorthQueryManagedIndexControl};
use std::{alloc::Layout, cell::Cell, sync::Arc};
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
};

pub(super) const PAGE_BITS: u32 = 5;
pub(super) const PAGE_SIZE: usize = 1 << PAGE_BITS;

mod edit;
pub(in super::super) use edit::PageEdit;

pub(in super::super) struct NodePages<Node> {
    root: Option<Arc<Directory<Node>>>,
    height: u32,
}

struct Directory<Node> {
    entry: Entry<Node>,
    _memory: Option<ExecutionMemoryReservation>,
}
enum Entry<Node> {
    Page(ExecutionArray<Option<Node>>),
    Branch {
        left: Option<Arc<Directory<Node>>>,
        right: Option<Arc<Directory<Node>>>,
    },
}

impl<Node> Default for NodePages<Node> {
    fn default() -> Self {
        Self {
            root: None,
            height: 0,
        }
    }
}

impl<Node> Clone for NodePages<Node> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            height: self.height,
        }
    }
}

impl<Node: Copy> NodePages<Node> {
    pub(super) fn from_scratch(
        execution: &ExecutionRequest<'_, '_>,
        scratch: &[Cell<Option<Node>>],
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self, Stop> {
        let page_count = scratch.len().div_ceil(PAGE_SIZE);
        let height = page_count
            .checked_next_power_of_two()
            .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?
            .trailing_zeros();
        let root = build(execution, scratch, 0, height, control)?;
        Ok(Self { root, height })
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
        if page.checked_shr(self.height).unwrap_or(0) > 0 {
            return Ok(None);
        }
        let mut current = self.root.as_deref();
        let mut height = self.height;
        while let Some(directory) = current {
            control.checkpoint(1)?;
            match &directory.entry {
                Entry::Page(values) => return Ok(Some(values)),
                Entry::Branch { left, right } => {
                    height -= 1;
                    current = if page & (1 << height) == 0 {
                        left.as_deref()
                    } else {
                        right.as_deref()
                    };
                }
            }
        }
        Ok(None)
    }

    fn install(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        page: usize,
        replacement: Option<Arc<Directory<Node>>>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        let mut root = self.root.clone();
        let mut height = self.height;
        while page >> height > 0 {
            root = Some(retain(
                execution,
                Entry::Branch {
                    left: root,
                    right: None,
                },
                control,
            )?);
            height += 1;
        }
        let root = install(
            execution,
            root.as_deref(),
            height,
            page,
            replacement,
            control,
        )?;
        control.poll()?;
        self.root = root;
        self.height = height;
        Ok(())
    }
}

fn retain<Node>(
    execution: &ExecutionRequest<'_, '_>,
    entry: Entry<Node>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Arc<Directory<Node>>, Stop> {
    let memory = header_memory::<Directory<Node>>(execution, control)?;
    Ok(Arc::new(Directory {
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

fn build<Node: Copy>(
    execution: &ExecutionRequest<'_, '_>,
    scratch: &[Cell<Option<Node>>],
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
            |index| scratch.get(start * PAGE_SIZE + index).and_then(Cell::get),
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

fn install<Node: Copy>(
    execution: &ExecutionRequest<'_, '_>,
    previous: Option<&Directory<Node>>,
    height: u32,
    page: usize,
    replacement: Option<Arc<Directory<Node>>>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Option<Arc<Directory<Node>>>, Stop> {
    control.checkpoint(1)?;
    if height == 0 {
        return Ok(replacement);
    }
    let (mut left, mut right) = previous
        .map(|directory| match &directory.entry {
            Entry::Branch { left, right } => (left.clone(), right.clone()),
            Entry::Page(_) => unreachable!("private branch directory height"),
        })
        .unwrap_or_default();
    let selected = if page & (1 << (height - 1)) == 0 {
        &mut left
    } else {
        &mut right
    };
    *selected = install(
        execution,
        selected.as_deref(),
        height - 1,
        page,
        replacement,
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
    if left.is_none() && right.is_none() {
        Ok(None)
    } else {
        retain(execution, Entry::Branch { left, right }, control).map(Some)
    }
}

fn page<Node: Copy>(
    execution: &ExecutionRequest<'_, '_>,
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
        retain(execution, Entry::Page(values), control).map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests;
