//! Immutable node pages. An edit replaces one page and its directory path.
use super::{Stop, WorthQueryManagedIndexControl};
use std::{alloc::Layout, cell::Cell, sync::Arc};
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
};

const PAGE_BITS: u32 = 5;
const PAGE_SIZE: usize = 1 << PAGE_BITS;

pub(super) struct NodePages<Node> {
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

    pub(super) fn get(
        &self,
        ordinal: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<&Node>, Stop> {
        let page = ordinal >> PAGE_BITS;
        if page.checked_shr(self.height).unwrap_or(0) > 0 {
            return Ok(None);
        }
        let mut current = self.root.as_deref();
        let mut height = self.height;
        while let Some(directory) = current {
            control.checkpoint(1)?;
            match &directory.entry {
                Entry::Page(values) => return Ok(values[ordinal & (PAGE_SIZE - 1)].as_ref()),
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

    pub(super) fn replace(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        ordinal: usize,
        value: Option<Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        let page = ordinal >> PAGE_BITS;
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
        let root = replace(execution, root.as_deref(), height, ordinal, value, control)?;
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
    control.checkpoint(1)?;
    let bytes = Layout::new::<Directory<Node>>()
        .extend(Layout::new::<[usize; 2]>())
        .map_err(|_| control.refuse(Stop::CapacityOverflow))?
        .0
        .pad_to_align()
        .size();
    let bytes = u64::try_from(bytes).map_err(|_| control.refuse(Stop::CapacityOverflow))?;
    let memory = execution
        .reserve_retained_memory(bytes)
        .map_err(|cause| control.refuse(Stop::Request(cause)))?;
    Ok(Arc::new(Directory {
        entry,
        _memory: memory,
    }))
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

fn replace<Node: Copy>(
    execution: &ExecutionRequest<'_, '_>,
    previous: Option<&Directory<Node>>,
    height: u32,
    ordinal: usize,
    value: Option<Node>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<Option<Arc<Directory<Node>>>, Stop> {
    control.checkpoint(1)?;
    if height == 0 {
        let values = previous.map(|directory| match &directory.entry {
            Entry::Page(values) => values,
            Entry::Branch { .. } => unreachable!("private page directory height"),
        });
        return page(
            execution,
            |index| {
                if index == ordinal & (PAGE_SIZE - 1) {
                    value
                } else {
                    values.and_then(|values| values[index])
                }
            },
            control,
        );
    }
    let (mut left, mut right) = previous
        .map(|directory| match &directory.entry {
            Entry::Branch { left, right } => (left.clone(), right.clone()),
            Entry::Page(_) => unreachable!("private branch directory height"),
        })
        .unwrap_or_default();
    let selected = if (ordinal >> PAGE_BITS) & (1 << (height - 1)) == 0 {
        &mut left
    } else {
        &mut right
    };
    *selected = replace(
        execution,
        selected.as_deref(),
        height - 1,
        ordinal,
        value,
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
