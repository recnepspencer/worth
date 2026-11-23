use super::fork_overlay::{ForkSubscriptions, ReverseSubscriptionStorage};
use super::ReverseSubscriptionIndex;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial as Denial, RetainedStoragePreparation as Work,
    RetainedStoragePreparationDenial,
};

impl RetainedStorageForkGrowth for ReverseSubscriptionIndex {
    fn prepare_fork_growth(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let Self { storage, valid: _ } = self;
        let growth = match storage {
            ReverseSubscriptionStorage::Exclusive(_) => ForkSubscriptions::conversion_charge()?,
            ReverseSubscriptionStorage::ForkShared(_) => Charge::ZERO,
        };
        work.reserve_visits(
            usize::try_from(growth.bytes())
                .map_err(|_| RetainedStoragePreparationDenial::ChargeOverflow)?,
        )?;
        Ok(growth)
    }
}
