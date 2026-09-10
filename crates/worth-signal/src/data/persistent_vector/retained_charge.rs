use crate::data::retained_storage::RetainedStorageBacking;
use std::sync::Arc;

use crate::data::retained_storage::{
    arc_allocation_charge, ordered_index_charge, RetainedStorageCharge as Charge,
    RetainedStorageMeasurement, RetainedStoragePreparation as Preparation,
    RetainedStoragePreparationDenial as Denial,
};

use super::{ForkPage, PersistentVector, PersistentVectorStorage};

impl<T: Clone, const PAGE_LEN: usize> PersistentVector<T, PAGE_LEN> {
    /// Constant-time carried fact. Missing accounting never triggers a scan.
    pub(crate) fn prepared_retained_charge(
        &self,
    ) -> Result<Charge, super::RetainedVectorMutationDenial> {
        self.retained_charge
            .ok_or(super::RetainedVectorMutationDenial::PreparationRequired)
    }

    pub(super) fn charge_after_persistent_fork(&self) -> Option<Charge> {
        let charge = self.retained_charge?;
        match &self.storage {
            PersistentVectorStorage::Exclusive(_) => charge
                .checked_add(arc_allocation_charge::<RetainedStorageBacking<Vec<T>>>().ok()?)
                .ok()?
                .checked_add(ordered_index_charge::<usize, Arc<ForkPage<T>>>(0).ok()?)
                .ok(),
            PersistentVectorStorage::ForkShared { .. } => Some(charge),
        }
    }
}

impl<T: Clone + RetainedStorageMeasurement, const PAGE_LEN: usize> PersistentVector<T, PAGE_LEN> {
    pub(crate) fn prepare_retained_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<Charge, Denial> {
        if let Some(charge) = self.retained_charge {
            return Ok(charge);
        }
        let charge = self.retained_heap_charge(work)?;
        self.retained_charge = Some(charge);
        Ok(charge)
    }
}

impl<T: Clone + RetainedStorageMeasurement, const PAGE_LEN: usize> RetainedStorageMeasurement
    for PersistentVector<T, PAGE_LEN>
{
    /// Explicit preparation traversal, never an ordinary charge lookup. The
    /// representation includes hidden base values and every retained overlay.
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match &self.storage {
            PersistentVectorStorage::Exclusive(values) => values.retained_heap_charge(work),
            PersistentVectorStorage::ForkShared {
                base,
                changed_pages,
                ..
            } => {
                let mut charge =
                    base.retained_heap_charge(work)?
                        .checked_add(ordered_index_charge::<usize, Arc<ForkPage<T>>>(
                            changed_pages.len(),
                        )?)?;
                for page in changed_pages.values() {
                    charge = charge.checked_add(page.retained_heap_charge(work)?)?;
                }
                Ok(charge)
            }
        }
    }
}

use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl<T: Clone + RetainedStorageMeasurement, const PAGE_LEN: usize> RetainedStorageForkPreparation
    for PersistentVector<T, PAGE_LEN>
{
    fn prepare_fork_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedStorageForkCharge, Denial> {
        work.visit()?;
        let source = self.prepare_retained_charge(work)?;
        // Successful preparation above establishes the source fact. The only
        // remaining failure in the prospective formula is checked overflow.
        let retained = self
            .charge_after_persistent_fork()
            .ok_or(Denial::ChargeOverflow)?;
        RetainedStorageForkCharge::from_charges(source, retained)
    }
}
