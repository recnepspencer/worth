use std::sync::Arc;

use crate::data::retained_storage::{
    arc_allocation_charge, ordered_index_charge, RetainedStorageCharge as Charge,
    RetainedStorageMeasurement, RetainedStoragePreparation as Preparation,
    RetainedStoragePreparationDenial as Denial,
};

use super::{ForkPage, PersistentVector, PersistentVectorStorage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedVectorMutationDenial {
    PreparationRequired,
    MissingElement { index: usize },
    Accounting(Denial),
}

impl From<Denial> for RetainedVectorMutationDenial {
    fn from(denial: Denial) -> Self {
        Self::Accounting(denial)
    }
}

/// A post-mutation accounting failure preserves the actual output. It is not
/// a pre-effect denial and cannot be interpreted as rollback of the edit.
#[derive(Debug)]
pub(crate) enum RetainedVectorMutationOutcome<R> {
    Accounted { output: R, charge: Charge },
    Unaccounted { output: R, denial: Denial },
}

impl<T: Clone + RetainedStorageMeasurement, const PAGE_LEN: usize> PersistentVector<T, PAGE_LEN> {
    pub(crate) fn edit_with_retained_charge<R>(
        &mut self,
        index: usize,
        work: &mut Preparation,
        edit: impl FnOnce(&mut T) -> R,
    ) -> Result<RetainedVectorMutationOutcome<R>, RetainedVectorMutationDenial> {
        if self.get(index).is_none() {
            return Err(RetainedVectorMutationDenial::MissingElement { index });
        }
        self.update_retained_charge(index, work, |values| {
            edit(
                values
                    .get_mut(index)
                    .expect("selected element remains installed"),
            )
        })
    }

    pub(crate) fn push_with_retained_charge(
        &mut self,
        value: T,
        work: &mut Preparation,
    ) -> Result<RetainedVectorMutationOutcome<()>, RetainedVectorMutationDenial> {
        self.update_retained_charge(self.len(), work, |values| values.push_back(value))
    }

    pub(crate) fn pop_with_retained_charge(
        &mut self,
        work: &mut Preparation,
    ) -> Result<RetainedVectorMutationOutcome<Option<T>>, RetainedVectorMutationDenial> {
        let charge = self.prepared_retained_charge()?;
        let Some(index) = self.len().checked_sub(1) else {
            return Ok(RetainedVectorMutationOutcome::Accounted {
                output: None,
                charge,
            });
        };
        self.update_retained_charge(index, work, |values| values.pop_back())
    }

    /// Measure only changed payload ownership and container capacities. The
    /// rest of the retained representation is carried unchanged. On unwind or
    /// post-mutation measurement failure, the fact remains unavailable rather
    /// than exposing the previous charge as though the edit had not happened.
    fn update_retained_charge<R>(
        &mut self,
        index: usize,
        work: &mut Preparation,
        mutate: impl FnOnce(&mut Self) -> R,
    ) -> Result<RetainedVectorMutationOutcome<R>, RetainedVectorMutationDenial> {
        let previous = self.prepared_retained_charge()?;
        let changed = self.mutation_granule_charge(index, work)?;
        let unchanged = previous.checked_sub(changed)?;
        self.retained_charge = None;
        let output = mutate(self);
        let updated = self
            .mutation_granule_charge(index, work)
            .and_then(|changed| unchanged.checked_add(changed));
        match updated {
            Ok(charge) => {
                self.retained_charge = Some(charge);
                Ok(RetainedVectorMutationOutcome::Accounted { output, charge })
            }
            Err(denial) => Ok(RetainedVectorMutationOutcome::Unaccounted { output, denial }),
        }
    }

    pub(super) fn mutation_granule_charge(
        &self,
        index: usize,
        work: &mut Preparation,
    ) -> Result<Charge, Denial> {
        work.visit()?;
        match &self.storage {
            PersistentVectorStorage::Exclusive(values) => {
                let payload = values
                    .get(index)
                    .map_or(Ok(Charge::ZERO), |value| value.retained_heap_charge(work))?;
                Charge::capacity::<T>(values.capacity())?.checked_add(payload)
            }
            PersistentVectorStorage::ForkShared { changed_pages, .. } => {
                let index_charge =
                    ordered_index_charge::<usize, Arc<ForkPage<T>>>(changed_pages.len())?;
                let page = changed_pages.get(&(index / PAGE_LEN));
                let page_charge = match page {
                    Some(page) => arc_allocation_charge::<ForkPage<T>>()?
                        .checked_add(page.mutation_granule_charge(index % PAGE_LEN, work)?)?,
                    None => Charge::ZERO,
                };
                index_charge.checked_add(page_charge)
            }
        }
    }
}
