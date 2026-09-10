use super::{FlatSegments, SegmentedStorage, SegmentedStore};
use crate::data::persistent_vector::PersistentVector;
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageBacking, RetainedStorageCharge as Charge,
    RetainedStorageForkGrowth, RetainedStorageForkGrowthDenial as Denial,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial,
};

impl<T: Clone, Id: Clone> RetainedStorageForkGrowth for SegmentedStore<T, Id> {
    fn prepare_fork_growth(&mut self, work: &mut Work) -> Result<Charge, Denial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let Self {
            storage,
            interner,
            id: _,
        } = self;
        let growth = match storage {
            SegmentedStorage::Exclusive(_) => {
                let backing = arc_allocation_charge::<RetainedStorageBacking<FlatSegments<T>>>()?;
                work.reserve_visits(
                    usize::try_from(backing.bytes())
                        .map_err(|_| RetainedStoragePreparationDenial::ChargeOverflow)?,
                )?;
                // The construction path creates and immediately forks this empty
                // appended vector. Its constructor carries the empty charge.
                let mut appended = PersistentVector::<Vec<T>>::new();
                backing.checked_add(appended.prepare_fork_growth(work)?)?
            }
            SegmentedStorage::ForkShared { base: _, appended } => {
                appended.prepare_fork_growth(work)?
            }
        };
        Ok(growth.checked_add(interner.prepare_fork_growth(work)?)?)
    }
}
