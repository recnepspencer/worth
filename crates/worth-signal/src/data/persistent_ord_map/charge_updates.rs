use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::Arc;

use crate::data::retained_storage::{
    btree_structure_charge, ordered_index_charge, RetainedStorageCharge as Charge,
    RetainedStorageMeasurement, RetainedStoragePreparation as Preparation,
    RetainedStoragePreparationDenial as Denial,
};

use super::{entry_handle, PersistentOrdMap, PersistentOrdMapStorage, SharedKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedMapMutationDenial {
    PreparationRequired,
    MissingKey,
    Accounting(Denial),
}

impl From<Denial> for RetainedMapMutationDenial {
    fn from(denial: Denial) -> Self {
        Self::Accounting(denial)
    }
}

/// Unaccounted preserves the actual mutation output. It is never a claim that
/// an edit was rolled back; the enclosing owner must retain that distinction.
#[derive(Debug)]
pub(crate) enum RetainedMapMutationOutcome<R> {
    Accounted { output: R, charge: Charge },
    Unaccounted { output: R, denial: Denial },
}

impl<K: Clone + Ord + RetainedStorageMeasurement, V: Clone + RetainedStorageMeasurement>
    PersistentOrdMap<K, V>
{
    pub(crate) fn edit_with_retained_charge<R>(
        &mut self,
        key: &K,
        work: &mut Preparation,
        edit: impl FnOnce(&mut V) -> R,
    ) -> Result<RetainedMapMutationOutcome<R>, RetainedMapMutationDenial> {
        if !self.contains_key(key) {
            return Err(RetainedMapMutationDenial::MissingKey);
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
    ) -> Result<RetainedMapMutationOutcome<Option<V>>, RetainedMapMutationDenial> {
        // The lookup copy lives only across this operation. Measure the actual
        // stored key afterward: K::clone need not preserve payload capacity.
        let lookup = key.clone();
        self.update_retained_charge(&lookup, work, |map| map.insert(key, value))
    }

    pub(crate) fn remove_with_retained_charge(
        &mut self,
        key: &K,
        work: &mut Preparation,
    ) -> Result<RetainedMapMutationOutcome<Option<V>>, RetainedMapMutationDenial> {
        self.update_retained_charge(key, work, |map| map.remove(key))
    }

    fn update_retained_charge<R>(
        &mut self,
        key: &K,
        work: &mut Preparation,
        mutate: impl FnOnce(&mut Self) -> R,
    ) -> Result<RetainedMapMutationOutcome<R>, RetainedMapMutationDenial> {
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
                Ok(RetainedMapMutationOutcome::Accounted { output, charge })
            }
            Err(denial) => Ok(RetainedMapMutationOutcome::Unaccounted { output, denial }),
        }
    }

    fn mutation_granule_charge(&self, key: &K, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match &self.storage {
            PersistentOrdMapStorage::Exclusive(values) => {
                let mut charge = btree_structure_charge::<K, V>(values.len())?;
                if let Some((stored_key, value)) = values.get_key_value(key) {
                    charge = charge
                        .checked_add(stored_key.retained_heap_charge(work)?)?
                        .checked_add(value.retained_heap_charge(work)?)?;
                }
                Ok(charge)
            }
            PersistentOrdMapStorage::ForkShared {
                base,
                changes,
                retired_base_intervals,
                ..
            } => {
                let mut charge = ordered_index_charge::<SharedKey<K>, Arc<V>>(changes.len())?
                    .checked_add(ordered_index_charge::<SharedKey<K>, SharedKey<K>>(
                        retired_base_intervals.len(),
                    )?)?;
                if let Some((stored_key, value)) = entry_handle::get_key_value(changes, key) {
                    charge = charge
                        .checked_add(stored_key.retained_heap_charge(work)?)?
                        .checked_add(value.retained_heap_charge(work)?)?;
                }
                charge.checked_add(interval_neighborhood_charge(
                    base,
                    retired_base_intervals,
                    key,
                    work,
                )?)
            }
        }
    }
}

/// Only intervals containing this key or its immediate base neighbors can
/// change during a single-key retirement/readmission. The immutable base fixes
/// those three lookup positions across the edit. Deduplicate containing
/// intervals so a merged interval is charged once, just as in full traversal.
fn interval_neighborhood_charge<K: Clone + Ord + RetainedStorageMeasurement, V>(
    base: &BTreeMap<K, V>,
    intervals: &im::OrdMap<SharedKey<K>, SharedKey<K>>,
    key: &K,
    work: &mut Preparation,
) -> Result<Charge, Denial> {
    let neighbors = [
        base.range(..key).next_back().map(|(key, _)| key),
        Some(key),
        base.range((Excluded(key), Unbounded))
            .next()
            .map(|(key, _)| key),
    ];
    let mut seen: [Option<&K>; 3] = [None; 3];
    let mut charge = Charge::ZERO;
    for (index, query) in neighbors.into_iter().enumerate() {
        work.visit()?;
        let Some(query) = query else {
            continue;
        };
        let candidate = entry_handle::get_key_value(intervals, query).or_else(|| {
            entry_handle::previous_after_exact_miss(intervals, query)
                .filter(|(_, end)| end.as_key() >= query)
        });
        if let Some((start, end)) = candidate {
            if seen.contains(&Some(start.as_key())) {
                continue;
            }
            seen[index] = Some(start.as_key());
            charge = charge
                .checked_add(start.retained_heap_charge(work)?)?
                .checked_add(end.retained_heap_charge(work)?)?;
        }
    }
    Ok(charge)
}
