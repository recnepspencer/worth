use crate::storage::substrate::{SharedColumn, SharedMap};

#[derive(Debug, Clone, Default)]
pub(crate) struct RecordSlotDirectory {
    logical_by_physical: SharedColumn<u64>,
    physical_by_logical: SharedMap<u64, usize>,
}

impl RecordSlotDirectory {
    pub(super) fn visit_new_allocations(
        &self,
        previous: &Self,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        self.logical_by_physical.visit_new_allocations(
            &previous.logical_by_physical,
            visitor,
            &mut crate::storage::substrate::visit_new_inline_value,
        );
        self.physical_by_logical.visit_new_allocations(
            &previous.physical_by_logical,
            visitor,
            &mut crate::storage::substrate::visit_new_inline_value,
        );
    }
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            logical_by_physical: SharedColumn::with_capacity(capacity),
            physical_by_logical: SharedMap::new(),
        }
    }

    pub(crate) fn restore(slots: Vec<u64>) -> Result<Self, &'static str> {
        let mut directory = Self::with_capacity(slots.len());
        for slot in slots {
            directory.insert(slot)?;
        }
        Ok(directory)
    }

    pub(crate) fn physical_index(&self, logical_slot: usize) -> Option<usize> {
        self.physical_by_logical
            .get(&(logical_slot as u64))
            .copied()
    }

    pub(super) fn logical_slot(&self, physical: usize) -> Option<usize> {
        self.logical_by_physical
            .get(physical)
            .map(|slot| *slot as usize)
    }

    /// Insert one slot and return the bytes of index paths actually copied.
    pub(crate) fn insert(&mut self, logical_slot: u64) -> Result<u64, &'static str> {
        if self.physical_by_logical.contains_key(&logical_slot) {
            return Err("record slot directory contains a duplicate logical slot");
        }
        let physical = self.logical_by_physical.len();
        let copied_bytes = self.logical_by_physical.push(logical_slot);
        Ok(copied_bytes.saturating_add(self.physical_by_logical.insert(logical_slot, physical)))
    }

    pub(crate) fn occupied_slots(&self) -> Vec<usize> {
        self.physical_by_logical
            .keys()
            .map(|slot| *slot as usize)
            .collect()
    }

    pub(crate) fn slots(&self) -> &SharedColumn<u64> {
        &self.logical_by_physical
    }

    pub(crate) fn len(&self) -> usize {
        self.logical_by_physical.len()
    }

    pub(crate) fn allocation_bytes(&self) -> u64 {
        self.logical_by_physical
            .allocation_bytes()
            .saturating_add(self.physical_by_logical.allocation_bytes())
    }

    pub(super) fn visit_allocations(
        &self,
        unique: bool,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        self.logical_by_physical.visit_allocations(
            unique,
            visitor,
            &mut crate::storage::substrate::visit_inline_value,
        );
        self.physical_by_logical.visit_allocations(
            unique,
            visitor,
            &mut crate::storage::substrate::visit_inline_value,
        );
    }
}
