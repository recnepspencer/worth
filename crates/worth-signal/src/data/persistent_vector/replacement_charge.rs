//! Charge-preserving replacement of an already materialized payload.
use super::{
    ForkPage, PersistentVector, PersistentVectorStorage, RetainedVectorCapacityDenial,
    RetainedVectorMutationDenial,
};
use crate::data::retained_storage::{
    arc_allocation_charge, ordered_index_charge, RetainedStorageCharge as Charge,
    RetainedStorageMeasurement, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial as Denial,
};
use std::sync::Arc;

/// An exclusive destination borrow keeps the prepared shape current. This
/// value owns replacement storage, not owner authority or resource custody.
pub(crate) struct PreparedRetainedVectorReplacement<'a, T: Clone, const PAGE_LEN: usize> {
    target: &'a mut PersistentVector<T, PAGE_LEN>,
    index: usize,
    value: T,
    unchanged: Charge,
    payload: Charge,
    required: Charge,
}

impl<T: Clone + RetainedStorageMeasurement, const PAGE_LEN: usize> PersistentVector<T, PAGE_LEN> {
    /// Measures the replacement before moving it; no old payload is cloned and
    /// no payload traversal can exhaust work after the write. A pre-write denial
    /// returns the unconsumed value. The caller checks aggregate capacity before
    /// publication; draft/payload disposal custody remains with that caller.
    pub(crate) fn prepare_retained_replacement(
        &mut self,
        index: usize,
        value: T,
        work: &mut Work,
    ) -> Result<PreparedRetainedVectorReplacement<'_, T, PAGE_LEN>, (T, RetainedVectorMutationDenial)>
    {
        let prepared = (|| {
            let previous = self.prepared_retained_charge()?;
            work.reserve_visits(
                self.indexed_mutation_work_bound(1)
                    .and_then(|n| n.checked_add(self.lookup_steps().checked_mul(3)?))
                    .ok_or(Denial::WorkExhausted {
                        maximum_visits: work.maximum_visits(),
                    })?,
            )?;
            if self.get(index).is_none() {
                return Err(RetainedVectorMutationDenial::MissingElement { index });
            }
            let changed = self.mutation_granule_charge(index, work)?;
            let unchanged = previous.checked_sub(changed)?;
            let payload = value.retained_heap_charge(work)?;
            let payload = match self.storage {
                PersistentVectorStorage::Exclusive(_) => payload,
                PersistentVectorStorage::ForkShared { .. } => {
                    arc_allocation_charge::<T>()?.checked_add(payload)?
                }
            };
            let required = unchanged
                .checked_add(payload)?
                .checked_add(self.prospective_replacement_structure_charge(index)?)?;
            Ok((unchanged, payload, required))
        })();
        let (unchanged, payload, required) = match prepared {
            Ok(prepared) => prepared,
            Err(denial) => return Err((value, denial)),
        };
        Ok(PreparedRetainedVectorReplacement {
            target: self,
            index,
            value,
            unchanged,
            payload,
            required,
        })
    }

    fn prospective_replacement_structure_charge(&self, index: usize) -> Result<Charge, Denial> {
        match &self.storage {
            PersistentVectorStorage::Exclusive(values) => Charge::capacity::<T>(values.capacity()),
            PersistentVectorStorage::ForkShared { changed_pages, .. } => {
                let page = changed_pages.get(&(index / PAGE_LEN));
                let pages = changed_pages
                    .len()
                    .checked_add(usize::from(page.is_none()))
                    .ok_or(Denial::ChargeOverflow)?;
                let page_charge = match page {
                    Some(page) => page.replacement_structure_bound(index % PAGE_LEN)?,
                    // A valid existing index absent from changed_pages is in
                    // the base. Its first override reserves four fixed entries.
                    None => Charge::capacity::<(usize, Arc<T>)>(4)?,
                };
                ordered_index_charge::<usize, Arc<ForkPage<T>>>(pages)?
                    .checked_add(arc_allocation_charge::<ForkPage<T>>()?)?
                    .checked_add(page_charge)
            }
        }
    }

    fn replacement_structure_charge(&self, index: usize) -> Result<Charge, Denial> {
        match &self.storage {
            PersistentVectorStorage::Exclusive(values) => Charge::capacity::<T>(values.capacity()),
            PersistentVectorStorage::ForkShared { changed_pages, .. } => {
                let page = changed_pages
                    .get(&(index / PAGE_LEN))
                    .expect("replacement installed its selected page");
                ordered_index_charge::<usize, Arc<ForkPage<T>>>(changed_pages.len())?
                    .checked_add(arc_allocation_charge::<ForkPage<T>>()?)?
                    .checked_add(page.mutation_structure_charge()?)
            }
        }
    }
}

impl<T: Clone + RetainedStorageMeasurement, const PAGE_LEN: usize>
    PreparedRetainedVectorReplacement<'_, T, PAGE_LEN>
{
    pub(crate) fn required_charge(&self) -> Charge {
        self.required
    }

    /// Checks the admitted ceiling before allocating any replacement storage.
    /// Successful publication reads capacities only, never payloads.
    pub(crate) fn publish(
        self,
        maximum: Charge,
    ) -> Result<Charge, (T, RetainedVectorCapacityDenial)> {
        if self.required > maximum {
            return Err((
                self.value,
                RetainedVectorCapacityDenial::CapacityExhausted {
                    maximum,
                    required: self.required,
                },
            ));
        }
        self.target.replace_discard(self.index, self.value);
        let charge = self
            .target
            .replacement_structure_charge(self.index)
            .and_then(|structure| structure.checked_add(self.payload))
            .and_then(|changed| self.unchanged.checked_add(changed))
            .expect("prepared replacement bounds all charge arithmetic");
        assert!(
            charge <= self.required,
            "replacement exceeded prepared structure bound"
        );
        self.target.retained_charge = Some(charge);
        Ok(charge)
    }
}

#[cfg(test)]
mod prospective_tests;
#[cfg(test)]
mod tests;
