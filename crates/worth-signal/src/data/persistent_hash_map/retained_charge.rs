use crate::data::retained_storage::RetainedStorageBacking;
use std::collections::{hash_map::RandomState, HashMap};
use std::hash::Hash;
use std::sync::Arc;

use super::{PersistentHashMap, PersistentHashMapStorage, SharedKey};
use crate::data::retained_storage::{
    arc_allocation_charge, RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

/// Rust 1.94/hashbrown 0.16.1. Public capacity is items + growth_left and
/// can shrink as tombstones accumulate without releasing the allocation.
/// The owner carries the largest capacity observed since table construction.
/// Buckets are at most twice that ceiling. Add control bytes, the maximum
/// 16-byte group and alignment padding; payload allocations are separate.
pub(super) fn base_structure_charge<K, V>(capacity: Option<usize>) -> Result<Charge, Denial> {
    let capacity = capacity.ok_or(Denial::RetainedExtentHistoryUnavailable)?;
    if capacity == 0 {
        return Ok(Charge::ZERO);
    }
    let buckets = capacity.checked_mul(2).ok_or(Denial::ChargeOverflow)?;
    Charge::capacity::<(K, V)>(buckets)?
        .checked_add(Charge::capacity::<u8>(buckets)?)?
        .checked_add(Charge::capacity::<u8>(
            16 + std::mem::align_of::<(K, V)>().max(16),
        )?)
}

/// im 15.1 HAMT has 32 inline Entry slots per node and a u32 bitmap. Each
/// entry holds either the key/value pair and hash, or an Arc, plus enum layout.
/// Seven levels cover its 32-bit full hash. Charge at most seven nodes per
/// entry plus an empty root. Collision vectors are charged from carried extents.
pub(super) fn overlay_structure_charge<K, V>(
    entries: usize,
    extents: &Option<super::CollisionExtents>,
) -> Result<Charge, Denial> {
    let extents = extents
        .as_ref()
        .ok_or(Denial::RetainedExtentHistoryUnavailable)?;
    overlay_structure_with_extents::<K, V>(entries, extents.retained_structure_charge::<K, V>()?)
}

fn overlay_structure_with_extents<K, V>(entries: usize, extents: Charge) -> Result<Charge, Denial> {
    let nodes = entries
        .checked_mul(7)
        .and_then(|n| n.checked_add(1))
        .ok_or(Denial::ChargeOverflow)?;
    let node = Charge::capacity::<(SharedKey<K>, Option<Arc<V>>)>(32)?
        .checked_add(Charge::capacity::<usize>(32 * 3 + 6)?)?;
    node.checked_mul(nodes)?
        .checked_add(arc_allocation_charge::<RandomState>()?)?
        .checked_add(extents)
}

impl<K: RetainedStorageMeasurement> RetainedStorageMeasurement for SharedKey<K> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        arc_allocation_charge::<K>()?.checked_add(self.as_key().retained_heap_charge(work)?)
    }
}

impl<K: Clone + Eq + Hash, V: Clone> PersistentHashMap<K, V> {
    pub(crate) fn empty_persistent_overlay_charge() -> Result<Charge, Denial> {
        Self::new()
            .charge_after_persistent_fork()
            .ok_or(Denial::ChargeOverflow)
    }

    pub(crate) fn prepared_retained_charge(
        &self,
    ) -> Result<Charge, super::RetainedHashMutationDenial> {
        self.retained_charge
            .ok_or(super::RetainedHashMutationDenial::PreparationRequired)
    }

    pub(super) fn charge_after_persistent_fork(&self) -> Option<Charge> {
        let charge = self.retained_charge?;
        match &self.storage {
            PersistentHashMapStorage::Exclusive(_) => charge
                .checked_add(arc_allocation_charge::<RetainedStorageBacking<HashMap<K, V>>>().ok()?)
                .ok()?
                .checked_add(
                    overlay_structure_with_extents::<K, V>(
                        0,
                        super::CollisionExtents::empty_structure_charge().ok()?,
                    )
                    .ok()?,
                )
                .ok(),
            PersistentHashMapStorage::ForkShared { .. } => Some(charge),
        }
    }
}

impl<K: Clone + Eq + Hash + RetainedStorageMeasurement, V: Clone + RetainedStorageMeasurement>
    PersistentHashMap<K, V>
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

impl<K: Clone + Eq + Hash + RetainedStorageMeasurement, V: Clone + RetainedStorageMeasurement>
    RetainedStorageMeasurement for PersistentHashMap<K, V>
{
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let mut charge = base_structure_charge::<K, V>(self.base_capacity)?;
        // HashMap's iterator scans buckets, including vacancies/tombstones.
        // Reserve that work before entry; live payload visits alone are not
        // a bound on the traversal of a sparse retained table.
        let buckets = self
            .base_capacity
            .ok_or(Denial::RetainedExtentHistoryUnavailable)?
            .checked_mul(2)
            .ok_or(Denial::ChargeOverflow)?;
        work.reserve_visits(buckets)?;
        let base = match &self.storage {
            PersistentHashMapStorage::Exclusive(base) => base,
            PersistentHashMapStorage::ForkShared {
                base,
                changes,
                collision_extents,
                ..
            } => {
                charge = charge
                    .checked_add(arc_allocation_charge::<
                        RetainedStorageBacking<HashMap<K, V>>,
                    >()?)?
                    .checked_add(overlay_structure_charge::<K, V>(
                        changes.len(),
                        collision_extents,
                    )?)?;
                // Sparse HAMT traversal also walks intermediate nodes. Seven
                // levels per entry conservatively covers those visits.
                work.reserve_visits(
                    changes
                        .len()
                        .checked_mul(7)
                        .and_then(|count| count.checked_add(1))
                        .ok_or(Denial::ChargeOverflow)?,
                )?;
                for (key, value) in changes {
                    charge = charge
                        .checked_add(key.retained_heap_charge(work)?)?
                        .checked_add(value.retained_heap_charge(work)?)?;
                }
                base.as_ref()
            }
        };
        for (key, value) in base {
            charge = charge
                .checked_add(key.retained_heap_charge(work)?)?
                .checked_add(value.retained_heap_charge(work)?)?;
        }
        Ok(charge)
    }
}

use crate::data::retained_storage::{RetainedStorageForkCharge, RetainedStorageForkPreparation};

impl<K: Clone + Eq + Hash + RetainedStorageMeasurement, V: Clone + RetainedStorageMeasurement>
    RetainedStorageForkPreparation for PersistentHashMap<K, V>
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
