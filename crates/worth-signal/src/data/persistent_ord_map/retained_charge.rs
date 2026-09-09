use crate::data::retained_storage::RetainedStorageBacking;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::data::retained_storage::{
    arc_allocation_charge, btree_structure_charge, ordered_index_charge,
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::{PersistentOrdMap, PersistentOrdMapStorage, SharedKey};

impl<K: RetainedStorageMeasurement> RetainedStorageMeasurement for SharedKey<K> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        arc_allocation_charge::<K>()?.checked_add(self.as_key().retained_heap_charge(work)?)
    }
}

impl<K: Clone + Ord, V: Clone> PersistentOrdMap<K, V> {
    /// Missing accounting requires explicit preparation; this never traverses.
    pub(crate) fn prepared_retained_charge(
        &self,
    ) -> Result<Charge, super::RetainedMapMutationDenial> {
        self.retained_charge
            .ok_or(super::RetainedMapMutationDenial::PreparationRequired)
    }

    pub(super) fn charge_after_persistent_fork(&self) -> Option<Charge> {
        let charge = self.retained_charge?;
        match &self.storage {
            PersistentOrdMapStorage::Exclusive(_) => charge
                .checked_add(
                    arc_allocation_charge::<RetainedStorageBacking<BTreeMap<K, V>>>().ok()?,
                )
                .ok()?
                .checked_add(ordered_index_charge::<SharedKey<K>, Arc<V>>(0).ok()?)
                .ok()?
                .checked_add(ordered_index_charge::<SharedKey<K>, SharedKey<K>>(0).ok()?)
                .ok(),
            PersistentOrdMapStorage::ForkShared { .. } => Some(charge),
        }
    }
}

impl<K: Clone + Ord + RetainedStorageMeasurement, V: Clone + RetainedStorageMeasurement>
    PersistentOrdMap<K, V>
{
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

impl<K: Clone + Ord + RetainedStorageMeasurement, V: Clone + RetainedStorageMeasurement>
    RetainedStorageMeasurement for PersistentOrdMap<K, V>
{
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let base = match &self.storage {
            PersistentOrdMapStorage::Exclusive(base) => base,
            PersistentOrdMapStorage::ForkShared { base, .. } => base.as_ref(),
        };
        let mut charge = btree_structure_charge::<K, V>(base.len())?;
        for (key, value) in base {
            charge = charge
                .checked_add(key.retained_heap_charge(work)?)?
                .checked_add(value.retained_heap_charge(work)?)?;
        }
        if let PersistentOrdMapStorage::ForkShared {
            changes,
            retired_base_intervals,
            ..
        } = &self.storage
        {
            charge = charge
                .checked_add(arc_allocation_charge::<
                    RetainedStorageBacking<BTreeMap<K, V>>,
                >()?)?
                .checked_add(ordered_index_charge::<SharedKey<K>, Arc<V>>(changes.len())?)?
                .checked_add(ordered_index_charge::<SharedKey<K>, SharedKey<K>>(
                    retired_base_intervals.len(),
                )?)?;
            for (key, value) in changes {
                charge = charge
                    .checked_add(key.retained_heap_charge(work)?)?
                    .checked_add(value.retained_heap_charge(work)?)?;
            }
            for (start, end) in retired_base_intervals {
                charge = charge
                    .checked_add(start.retained_heap_charge(work)?)?
                    .checked_add(end.retained_heap_charge(work)?)?;
            }
        }
        Ok(charge)
    }
}

use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl<K: Clone + Ord + RetainedStorageMeasurement, V: Clone + RetainedStorageMeasurement>
    RetainedStorageForkPreparation for PersistentOrdMap<K, V>
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
