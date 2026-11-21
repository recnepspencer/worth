//! Original memory publication ownership outlives reclaimable live history.
use super::{lock_state, validate_owner, CompositeHistoryCatalog};
use crate::history::publication::memory_observation::MemoryPublicationRecord;
use crate::history::{WorldObservedMemoryPublication, WorldReadmittedPublicationDenial as Denial};
use std::sync::{atomic::Ordering, Arc, Mutex, Weak};
mod growth;
mod readmission;

#[derive(Debug)]
enum Slot {
    Vacant(Option<usize>),
    Original(Arc<MemoryPublicationRecord>),
}
#[derive(Debug)]
struct Records {
    receiver: ReceiverAdmission,
    slots: Vec<Slot>,
    vacant: Option<usize>,
    free_count: usize,
    pending_reservations: usize,
    pending_by_tier: [usize; growth::MAX_GROWTH_TIERS],
    _capacity_funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl Records {
    fn drop_pending(&mut self, tier: u8) {
        self.pending_reservations = self
            .pending_reservations
            .checked_sub(1)
            .expect("pending reservation was counted");
        self.pending_by_tier[tier as usize] = self.pending_by_tier[tier as usize]
            .checked_sub(1)
            .expect("pending reservation tier was counted");
    }
}
impl Default for Records {
    fn default() -> Self {
        Self {
            receiver: ReceiverAdmission::default(),
            slots: Vec::new(),
            vacant: None,
            free_count: 0,
            pending_reservations: 0,
            pending_by_tier: [0; growth::MAX_GROWTH_TIERS],
            _capacity_funding: None,
        }
    }
}
#[derive(Debug, Default)]
enum ReceiverAdmission {
    #[default]
    Available,
    Preparing,
    Installed,
    Refused,
}
type Registry = Mutex<Records>;
#[derive(Debug, Clone, Default)]
pub(super) struct MemoryOriginalRegistry(Arc<Registry>);
/// The exact canonical envelope owns its pending slot. Abandonment releases it;
/// successful movement leaves the compact original with the World owner.
#[derive(Debug)]
pub(crate) struct MemoryOriginalRegistration {
    registry: Weak<Registry>,
    slot: usize,
    record: Arc<MemoryPublicationRecord>,
}
/// One provisional registry reservation held before owner effects. Any growth
/// stays provisional until a sealed original is accepted; dropping it unfilled
/// refunds its backing and leaves the registry unchanged.
#[derive(Debug)]
pub(crate) struct MemoryOriginalSlot {
    registry: Weak<Registry>,
    growth: Option<growth::ProvisionalGrowth>,
    growth_tier: u8,
    filled: bool,
}
impl MemoryOriginalSlot {
    pub(crate) fn fill(
        mut self,
        record: Arc<MemoryPublicationRecord>,
    ) -> MemoryOriginalRegistration {
        self.filled = true;
        let growth = self.growth.take();
        let mut unused_growth = None;
        let slot = if let Some(registry) = self.registry.upgrade() {
            let mut records = registry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            records.drop_pending(self.growth_tier);
            let cur_cap = records.slots.capacity();
            let used_after = (records.slots.len() - records.free_count) + 1;
            let must_grow = records.vacant.is_none() && records.slots.len() == cur_cap;
            if let Some(mut growth) = growth {
                let gcap = growth.backing.capacity();
                if gcap > cur_cap
                    && (must_grow
                        || growth::backing_needed_for_pending(
                            &records.pending_by_tier,
                            cur_cap,
                            gcap,
                            used_after,
                        ))
                {
                    growth.backing.append(&mut records.slots);
                    records.slots = growth.backing;
                    records._capacity_funding = growth.funding;
                } else {
                    unused_growth = Some(growth);
                }
            }
            if let Some(slot) = records.vacant {
                let Slot::Vacant(next) = records.slots[slot] else {
                    unreachable!("free slot is vacant")
                };
                records.vacant = next;
                records.free_count -= 1;
                records.slots[slot] = Slot::Original(Arc::clone(&record));
                slot
            } else {
                assert!(
                    records.slots.len() < records.slots.capacity(),
                    "provisional reservation guarantees capacity at seal"
                );
                let slot = records.slots.len();
                records.slots.push(Slot::Original(Arc::clone(&record)));
                slot
            }
        } else {
            unused_growth = growth;
            0
        };
        drop(unused_growth);
        MemoryOriginalRegistration {
            registry: self.registry.clone(),
            slot,
            record,
        }
    }
}
impl Drop for MemoryOriginalSlot {
    fn drop(&mut self) {
        if self.filled {
            return;
        }
        drop(self.growth.take());
        if let Some(registry) = self.registry.upgrade() {
            let mut records = registry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            records.drop_pending(self.growth_tier);
        }
    }
}
impl MemoryOriginalRegistration {
    pub(crate) fn mark_committed(&self, movement: &crate::branch::ProductBranchReferenceMovement) {
        let registry = self.registry.upgrade();
        let _records = registry.as_ref().map(|registry| {
            registry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        self.record.mark_committed(movement);
    }
}
impl Drop for MemoryOriginalRegistration {
    fn drop(&mut self) {
        if self.record.committed.load(Ordering::Acquire) {
            return;
        }
        if let Some(registry) = self.registry.upgrade() {
            let mut records = registry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let matches = match &records.slots[self.slot] {
                Slot::Original(record) => Arc::ptr_eq(record, &self.record),
                Slot::Vacant(_) => false,
            };
            assert!(
                matches,
                "the pending slot belongs to this exact canonical attempt"
            );
            let vacant = records.vacant;
            let removed = std::mem::replace(&mut records.slots[self.slot], Slot::Vacant(vacant));
            records.vacant = Some(self.slot);
            records.free_count += 1;
            drop(records);
            drop(removed);
        }
    }
}
impl CompositeHistoryCatalog {
    pub(crate) fn snapshot_memory_originals(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_execution::ExecutionArray<Arc<MemoryPublicationRecord>>,
        crate::lifecycle::WorldInterimCheckpointRefusal,
    > {
        use crate::lifecycle::WorldInterimCheckpointRefusal as Stop;
        let records = self
            .originals
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if matches!(
            records.receiver,
            ReceiverAdmission::Preparing | ReceiverAdmission::Refused
        ) {
            return Err(Stop::OwnerUnavailable);
        }
        crate::lifecycle::interim_checkpoint::charge(request, records.slots.len())?;
        let committed = records
            .slots
            .iter()
            .filter(|slot| match slot {
                Slot::Original(record) => record.committed.load(Ordering::Acquire),
                Slot::Vacant(_) => false,
            })
            .count();
        let mut builder =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, committed)
                .map_err(Stop::Backing)?;
        for slot in &records.slots {
            crate::lifecycle::interim_checkpoint::charge(request, 1)?;
            match slot {
                Slot::Original(record) if record.committed.load(Ordering::Acquire) => {
                    builder.push(record.clone()).map_err(Stop::Backing)?
                }
                Slot::Original(_) | Slot::Vacant(_) => {}
            }
        }
        builder.seal().map_err(Stop::Backing)
    }

    pub(crate) fn matches_memory_originals(
        &self,
        originals: &[Arc<MemoryPublicationRecord>],
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, crate::lifecycle::WorldInterimCheckpointRefusal> {
        let records = self
            .originals
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        crate::lifecycle::interim_checkpoint::charge(request, records.slots.len())?;
        let mut originals = originals.iter();
        for slot in &records.slots {
            match slot {
                Slot::Original(record) if record.committed.load(Ordering::Acquire) => {
                    if originals
                        .next()
                        .is_none_or(|original| !Arc::ptr_eq(original, record))
                    {
                        return Ok(false);
                    }
                }
                Slot::Original(_) | Slot::Vacant(_) => {}
            }
        }
        Ok(originals.next().is_none())
    }

    /// Reserve one provisional original slot, funding any needed registry
    /// growth on this request without altering the registry until a sealed
    /// original is accepted.
    pub(crate) fn reserve_memory_original(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        MemoryOriginalSlot,
        worth_relational::facade::branch::RelationalRecordedComponentRefusal,
    > {
        use worth_relational::facade::branch::RelationalRecordedComponentRefusal as Refusal;
        let malformed = || Refusal::PayloadEncoding {
            detail: "original World registry capacity unavailable".into(),
        };
        let mut records = self
            .originals
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if matches!(
            records.receiver,
            ReceiverAdmission::Preparing | ReceiverAdmission::Refused
        ) {
            return Err(Refusal::Basis(
                worth_relational::facade::branch::RelationalBranchBasisDenial::OwnerUnavailable,
            ));
        }
        let used = records.slots.len() - records.free_count;
        let next_pending = records
            .pending_reservations
            .checked_add(1)
            .ok_or_else(malformed)?;
        let needed = used.checked_add(next_pending).ok_or_else(malformed)?;
        let (growth, growth_tier) = if needed > records.slots.capacity() {
            let (growth, tier) = growth::reserve_provisional_growth(
                execution,
                records.slots.capacity(),
                needed,
                malformed,
            )?;
            (Some(growth), tier)
        } else {
            (None, 0)
        };
        records.pending_reservations = next_pending;
        records.pending_by_tier[growth_tier as usize] += 1;
        Ok(MemoryOriginalSlot {
            registry: Arc::downgrade(&self.originals.0),
            growth,
            growth_tier,
            filled: false,
        })
    }
    pub(crate) fn observe_memory_original(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: crate::identity::RuntimeWorldOwnerIdentity,
        effect: worth_relational::facade::durability::ObservedRelationalMemoryCommit,
    ) -> Result<WorldObservedMemoryPublication, Denial> {
        validate_owner(&lock_state(&self.state), owner).map_err(Denial::History)?;
        let records = self
            .originals
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut answer = None;
        let scope = execution.run_native_with_checkpoint(|context| {
            answer = Some((|| {
                let mut found = None;
                for slot in &records.slots {
                    (if context.has_accounting() {
                        context.checkpoint(1)
                    } else {
                        context.poll()
                    })
                    .map_err(|cause| {
                        Denial::Request(worth_execution::WorkCeilingDenial::Stopped(cause))
                    })?;
                    let Slot::Original(record) = slot else {
                        continue;
                    };
                    if !record.committed.load(Ordering::Acquire)
                        || !record.after_component.matches_original(&effect)
                    {
                        continue;
                    }
                    // The opaque Native packet proves the exact immutable canonical
                    // owner allocation, including its semantic Arc, in constant time.
                    if found.is_some() {
                        return Err(Denial::NativeMismatch);
                    }
                    found = Some(Arc::clone(record));
                }
                found.ok_or(Denial::NotReadmitted)
            })());
        });
        let record = match (scope, answer) {
            (Ok(_), Some(answer)) => answer?,
            (
                Err(worth_execution::WorkCeilingDenial::Stopped(
                    worth_execution::MapKernelStop::NestedStopped,
                )),
                Some(Err(cause)),
            ) => return Err(cause),
            (Err(cause), _) => return Err(Denial::Request(cause)),
            (Ok(_), None) => unreachable!("original owner observation completed"),
        };
        Ok(WorldObservedMemoryPublication {
            record,
            native: effect,
        })
    }

    #[cfg(test)]
    pub(crate) fn memory_original_capacity_for_test(&self) -> usize {
        self.originals
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .slots
            .capacity()
    }

    #[cfg(test)]
    pub(crate) const fn memory_original_slot_size_for_test() -> usize {
        std::mem::size_of::<Slot>()
    }
}
