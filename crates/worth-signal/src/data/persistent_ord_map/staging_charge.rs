use super::{PersistentOrdMap, RetainedMapMutationDenial, SharedKey};
use crate::data::retained_storage::{
    arc_allocation_charge, ordered_index_charge, RetainedStorageCharge as Charge,
    RetainedStorageMeasurement, RetainedStoragePreparation as Work,
};
use std::sync::Arc;

impl<K: Clone + Ord + RetainedStorageMeasurement, V: Clone + RetainedStorageMeasurement>
    PersistentOrdMap<K, V>
{
    /// Conservative custody for the old root, staged root, and one-key edit.
    /// Existing payload charge is carried; only the incoming key/value is read.
    pub(crate) fn prepare_insert_staging_charge(
        &self,
        key: &K,
        value: &V,
        work: &mut Work,
    ) -> Result<Charge, RetainedMapMutationDenial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let retained = self.prepared_retained_charge()?;
        let extent = self.len().checked_add(1).ok_or(
            crate::data::retained_storage::RetainedStoragePreparationDenial::ChargeOverflow,
        )?;
        retained
            .checked_mul(2)?
            .checked_add(ordered_index_charge::<SharedKey<K>, Arc<V>>(extent)?)?
            .checked_add(ordered_index_charge::<SharedKey<K>, SharedKey<K>>(extent)?)?
            .checked_add(arc_allocation_charge::<K>()?)?
            .checked_add(key.retained_heap_charge(work)?)?
            .checked_add(arc_allocation_charge::<V>()?)?
            .checked_add(value.retained_heap_charge(work)?)
            .map_err(Into::into)
    }
}
