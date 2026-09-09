use super::PersistentVector;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial as Denial, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial,
};

impl<T: Clone, const N: usize> RetainedStorageForkGrowth for PersistentVector<T, N> {
    fn prepare_fork_growth(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let source = self
            .prepared_retained_charge()
            .map_err(|_| Denial::PreparationRequired)?;
        let retained = self
            .charge_after_persistent_fork()
            .ok_or(RetainedStoragePreparationDenial::ChargeOverflow)?;
        let growth = retained.checked_sub(source)?;
        work.reserve_visits(
            usize::try_from(growth.bytes())
                .map_err(|_| RetainedStoragePreparationDenial::ChargeOverflow)?,
        )?;
        Ok(growth)
    }
}
