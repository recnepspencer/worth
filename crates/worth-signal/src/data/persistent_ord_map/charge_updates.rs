use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::Arc;

use crate::data::retained_storage::{
    arc_allocation_charge, btree_structure_charge, ordered_edit_growth_charge,
    ordered_index_charge, RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

use super::{entry_handle, PersistentOrdMap, PersistentOrdMapStorage, SharedKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedMapMutationDenial {
    PreparationRequired,
    #[cfg(test)]
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
    /// One selected persistent key edit after a fork, excluding the incoming
    /// value. The immutable base is borrowed; only two overlay paths and the
    /// old selected value can be copied.
    pub(crate) fn selected_edit_growth_bound_with_growth(
        &self,
        key: &K,
        prospective_edits: usize,
        work: &mut Preparation,
    ) -> Result<Charge, Denial> {
        work.visit()?;
        let (changes, retired) = match &self.storage {
            PersistentOrdMapStorage::Exclusive(_) => (0, 0),
            PersistentOrdMapStorage::ForkShared {
                changes,
                retired_base_intervals,
                ..
            } => (changes.len(), retired_base_intervals.len()),
        };
        let old_value = self
            .get(key)
            .map_or(Ok(Charge::ZERO), |value| value.retained_heap_charge(work))?;
        let future_changes = changes
            .checked_add(prospective_edits)
            .ok_or(Denial::ChargeOverflow)?;
        let future_retired = retired
            .checked_add(prospective_edits)
            .ok_or(Denial::ChargeOverflow)?;
        ordered_edit_growth_charge::<SharedKey<K>, Arc<V>>(future_changes)?
            .checked_add(ordered_edit_growth_charge::<SharedKey<K>, SharedKey<K>>(
                future_retired,
            )?)?
            .checked_add(arc_allocation_charge::<K>()?)?
            .checked_add(arc_allocation_charge::<V>()?)?
            .checked_add(key.retained_heap_charge(work)?)?
            .checked_add(old_value)
    }

    #[cfg(test)]
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

impl<K: Clone + Ord, V: Clone> PersistentOrdMap<K, V> {
    /// Heap that a bounded run of edits can allocate in the fork overlay.
    /// The immutable base and selected value payloads remain shared; callers
    /// account incoming values and copies of selected old values separately.
    pub(crate) fn batch_edit_structure_growth_bound(
        &self,
        prospective_edits: usize,
    ) -> Result<Charge, Denial> {
        if prospective_edits == 0 {
            return Ok(Charge::ZERO);
        }
        let (changes, retired) = match &self.storage {
            PersistentOrdMapStorage::Exclusive(_) => (0, 0),
            PersistentOrdMapStorage::ForkShared {
                changes,
                retired_base_intervals,
                ..
            } => (changes.len(), retired_base_intervals.len()),
        };
        let future_changes = changes
            .checked_add(prospective_edits)
            .ok_or(Denial::ChargeOverflow)?;
        let future_retired = retired
            .checked_add(prospective_edits)
            .and_then(|count| count.checked_add(1))
            .ok_or(Denial::ChargeOverflow)?;
        // A readmission or retirement edits one changed-key path and at most
        // three interval paths. One overlay value and up to three key cells
        // may be new during that edit.
        ordered_edit_growth_charge::<SharedKey<K>, Arc<V>>(future_changes)?
            .checked_add(
                ordered_edit_growth_charge::<SharedKey<K>, SharedKey<K>>(future_retired)?
                    .checked_mul(3)?,
            )?
            .checked_add(arc_allocation_charge::<K>()?.checked_mul(3)?)?
            .checked_add(arc_allocation_charge::<V>()?)?
            .checked_mul(prospective_edits)
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
