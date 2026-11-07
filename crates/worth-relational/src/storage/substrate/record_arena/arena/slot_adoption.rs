//! Sparse slot coordinates select whole immutable stored carriers.
use super::{RecordSlotDirectory, SharedColumn};
use crate::storage::substrate::StoredValue;

pub(super) struct SlotAdoption {
    selected: usize,
    destination: usize,
    append: bool,
}
impl SlotAdoption {
    pub(super) fn prepare(
        logical: usize,
        destination: &RecordSlotDirectory,
        selected: &RecordSlotDirectory,
    ) -> Option<Self> {
        let selected = selected.physical_index(logical)?;
        let existing = destination.physical_index(logical);
        Some(Self {
            selected,
            destination: existing.unwrap_or_else(|| destination.len()),
            append: existing.is_none(),
        })
    }
    pub(super) fn selected_physical(&self) -> usize {
        self.selected
    }
    pub(super) fn destination_physical(&self) -> usize {
        self.destination
    }
    pub(super) fn appends(&self) -> bool {
        self.append
    }
    pub(super) fn copy_selected_value<T: StoredValue>(
        &self,
        destination: &mut SharedColumn<T>,
        selected: &SharedColumn<T>,
    ) -> u64 {
        destination.copy_value_from(self.destination, selected, self.selected)
    }
}
#[cfg(test)]
mod tests;
