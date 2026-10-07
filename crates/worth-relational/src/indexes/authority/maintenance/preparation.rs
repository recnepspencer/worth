use super::work::MaintenanceWork;
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use std::mem::size_of;

impl MaintenanceWork<'_> {
    pub(super) fn lookup(&mut self, entries: usize, width: usize) -> Result<(), Denial> {
        let levels = if entries == 0 {
            0
        } else {
            entries.ilog2() as usize + 1
        };
        let work = levels
            .checked_mul(11)
            .and_then(|n| n.checked_mul(width))
            .and_then(|n| n.checked_add(1))
            .ok_or(Denial::WorkBudgetExceeded)?;
        self.prepare(work as u64, 0)
    }
    pub(super) fn locator(
        &mut self,
        locator: &worth_foundational::facade::AspectFieldLocator,
    ) -> Result<(), Denial> {
        self.prepare(locator.field_path().fields().len() as u64 + 1, 0)?;
        let width = locator
            .field_path()
            .fields()
            .iter()
            .try_fold(locator.aspect().aspect_key().as_str().len(), |n, field| {
                n.checked_add(field.as_str().len())
            })
            .ok_or(Denial::WorkBudgetExceeded)?;
        self.prepare(
            width as u64,
            locator.owned_allocation_capacity_bytes() as u64,
        )
    }

    pub(super) fn array<T>(&mut self, entries: usize) -> Result<(), Denial> {
        let bytes = entries
            .checked_mul(size_of::<T>())
            .ok_or(Denial::WorkBudgetExceeded)?;
        self.prepare(0, bytes as u64)
    }

    pub(super) fn grow_vec<T>(&mut self, values: &mut Vec<T>) -> Result<(), Denial> {
        if values.len() < values.capacity() {
            return Ok(());
        }
        let next = values
            .capacity()
            .checked_mul(2)
            .ok_or(Denial::WorkBudgetExceeded)?
            .max(4);
        self.array::<T>(next)?;
        self.prepare(values.len() as u64, 0)?;
        values.reserve_exact(next - values.len());
        Ok(())
    }

    pub(super) fn ordered<K, V>(
        &mut self,
        entries: usize,
        comparison_width: usize,
        owned_key_bytes: usize,
    ) -> Result<(), Denial> {
        let levels = if entries == 0 {
            0
        } else {
            entries.ilog2() as usize + 1
        };
        let visits = levels
            .checked_mul(11)
            .and_then(|n| n.checked_mul(comparison_width))
            .and_then(|n| n.checked_add(1))
            .ok_or(Denial::WorkBudgetExceeded)?;
        let node_bytes = size_of::<(K, V)>()
            .checked_mul(11)
            .and_then(|n| n.checked_add(size_of::<usize>() * 14))
            .and_then(|n| n.checked_mul(levels + 2))
            .and_then(|n| n.checked_add(owned_key_bytes))
            .ok_or(Denial::WorkBudgetExceeded)?;
        self.prepare(visits as u64, node_bytes as u64)
    }
}
