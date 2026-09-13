use super::retained_charge::{base_structure_charge, overlay_structure_charge};
use super::{PersistentHashMap, PersistentHashMapStorage};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedHashMutationDenial {
    PreparationRequired,
    MissingKey,
    Accounting(Denial),
}

impl From<Denial> for RetainedHashMutationDenial {
    fn from(denial: Denial) -> Self {
        Self::Accounting(denial)
    }
}

/// Unaccounted preserves the actual mutation output. It is never a claim that
/// an edit was rolled back; the enclosing owner must retain that distinction.
#[derive(Debug)]
pub(crate) enum RetainedHashMutationOutcome<R> {
    Accounted { output: R, charge: Charge },
    Unaccounted { output: R, denial: Denial },
}

impl<K: Clone + Eq + Hash + RetainedStorageMeasurement, V: Clone + RetainedStorageMeasurement>
    PersistentHashMap<K, V>
{
    pub(crate) fn edit_with_retained_charge<R>(
        &mut self,
        key: &K,
        work: &mut Preparation,
        edit: impl FnOnce(&mut V) -> R,
    ) -> Result<RetainedHashMutationOutcome<R>, RetainedHashMutationDenial> {
        if self.get(key).is_none() {
            return Err(RetainedHashMutationDenial::MissingKey);
        }
        self.update_retained_charge(key, work, |map| {
            edit(map.get_mut(key).expect("selected key remains installed"))
        })
    }

    pub(crate) fn insert_with_retained_charge(
        &mut self,
        key: K,
        value: V,
        work: &mut Preparation,
    ) -> Result<RetainedHashMutationOutcome<Option<V>>, RetainedHashMutationDenial> {
        // The lookup copy lives only across this operation. Measure the actual
        // stored key afterward: K::clone need not preserve payload capacity.
        let lookup = key.clone();
        self.update_retained_charge(&lookup, work, |map| map.insert(key, value))
    }

    pub(crate) fn remove_with_retained_charge(
        &mut self,
        key: &K,
        work: &mut Preparation,
    ) -> Result<RetainedHashMutationOutcome<Option<V>>, RetainedHashMutationDenial> {
        self.update_retained_charge(key, work, |map| map.remove(key))
    }

    fn update_retained_charge<R>(
        &mut self,
        key: &K,
        work: &mut Preparation,
        mutate: impl FnOnce(&mut Self) -> R,
    ) -> Result<RetainedHashMutationOutcome<R>, RetainedHashMutationDenial> {
        let previous = self.prepared_retained_charge()?;
        let changed = self.mutation_granule_charge(key, work)?;
        let unchanged = previous.checked_sub(changed)?;
        self.retained_charge = None;
        let output = mutate(self);
        let updated = self
            .mutation_granule_charge(key, work)
            .and_then(|changed| unchanged.checked_add(changed));
        match updated {
            Ok(charge) => {
                self.retained_charge = Some(charge);
                Ok(RetainedHashMutationOutcome::Accounted { output, charge })
            }
            Err(denial) => Ok(RetainedHashMutationOutcome::Unaccounted { output, denial }),
        }
    }

    fn mutation_granule_charge(&self, key: &K, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match &self.storage {
            PersistentHashMapStorage::Exclusive(values) => {
                let mut charge = base_structure_charge::<K, V>(self.base_capacity)?;
                if let Some((stored_key, value)) = values.get_key_value(key) {
                    charge = charge
                        .checked_add(stored_key.retained_heap_charge(work)?)?
                        .checked_add(value.retained_heap_charge(work)?)?;
                }
                Ok(charge)
            }
            PersistentHashMapStorage::ForkShared {
                changes,
                collision_extents,
                ..
            } => {
                let mut charge =
                    overlay_structure_charge::<K, V>(changes.len(), collision_extents)?;
                if let Some((stored_key, value)) = changes.get_key_value(key) {
                    charge = charge
                        .checked_add(stored_key.retained_heap_charge(work)?)?
                        .checked_add(value.retained_heap_charge(work)?)?;
                }
                Ok(charge)
            }
        }
    }
}
