use super::{
    ForkPage, PersistentVector, PersistentVectorStorage, RetainedVectorMutationDenial,
    RetainedVectorStagingDenial,
};
use crate::data::retained_storage::{
    arc_allocation_charge, ordered_index_charge, RetainedStorageCharge as Charge,
    RetainedStorageMeasurement, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial,
};
use std::sync::Arc;

impl<T: Clone + RetainedStorageMeasurement, const PAGE_LEN: usize> PersistentVector<T, PAGE_LEN> {
    /// Conservative peak custody for an append staged beside the current root.
    /// Existing payload is represented by its carried fact; only the incoming
    /// value is traversed. Retained append callers always fork before mutation.
    pub(crate) fn prepare_push_staging_charge(
        &self,
        value: &T,
        work: &mut Work,
    ) -> Result<Charge, RetainedVectorStagingDenial> {
        work.reserve_visits(std::mem::size_of::<Self>())
            .map_err(RetainedVectorMutationDenial::from)
            .map_err(RetainedVectorStagingDenial::Mutation)?;
        let retained = self
            .prepared_retained_charge()
            .map_err(RetainedVectorStagingDenial::Mutation)?;
        let PersistentVectorStorage::ForkShared { changed_pages, .. } = &self.storage else {
            return Err(RetainedVectorStagingDenial::ForkPreparationRequired);
        };
        let page_count = changed_pages
            .len()
            .checked_add(1)
            .ok_or(RetainedStoragePreparationDenial::ChargeOverflow)
            .map_err(RetainedVectorMutationDenial::from)
            .map_err(RetainedVectorStagingDenial::Mutation)?;
        let page_capacity = PAGE_LEN
            .max(4)
            .checked_next_power_of_two()
            .ok_or(RetainedStoragePreparationDenial::ChargeOverflow)
            .map_err(RetainedVectorMutationDenial::from)
            .map_err(RetainedVectorStagingDenial::Mutation)?;
        let prospective_page = Charge::capacity::<(usize, Arc<T>)>(page_capacity)
            .and_then(|charge| charge.checked_add(Charge::capacity::<Arc<T>>(page_capacity)?))
            .map_err(RetainedVectorMutationDenial::from)
            .map_err(RetainedVectorStagingDenial::Mutation)?;
        retained
            .checked_mul(2)
            .and_then(|charge| {
                charge.checked_add(ordered_index_charge::<usize, Arc<ForkPage<T>>>(page_count)?)
            })
            .and_then(|charge| charge.checked_add(arc_allocation_charge::<ForkPage<T>>()?))
            .and_then(|charge| charge.checked_add(prospective_page))
            .and_then(|charge| charge.checked_add(arc_allocation_charge::<T>()?))
            .and_then(|charge| charge.checked_add(value.retained_heap_charge(work)?))
            .map_err(RetainedVectorMutationDenial::from)
            .map_err(RetainedVectorStagingDenial::Mutation)
    }
}
