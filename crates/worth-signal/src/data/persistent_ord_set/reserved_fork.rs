use super::PersistentOrdSet;
use crate::data::retained_storage::SignalConditionalRetentionReservation;

impl<T: Clone + Ord> PersistentOrdSet<T> {
    pub(crate) fn fork_reserved(
        &mut self,
        resources: &mut SignalConditionalRetentionReservation,
    ) -> Self {
        Self {
            values: self.values.fork_reserved(resources),
        }
    }
}

impl<T: Clone + Ord> crate::data::retained_storage::RetainedStorageForkGrowth
    for super::PersistentOrdSet<T>
{
    fn prepare_fork_growth(
        &mut self,
        work: &mut crate::data::retained_storage::RetainedStoragePreparation,
    ) -> Result<
        crate::data::retained_storage::RetainedStorageCharge,
        crate::data::retained_storage::RetainedStorageForkGrowthDenial,
    > {
        self.values.prepare_fork_growth(work)
    }
}
