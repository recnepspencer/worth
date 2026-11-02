//! Original memory publication ownership outlives reclaimable live history.
use super::{lock_state, validate_owner, CompositeHistoryCatalog};
use crate::history::publication::memory_observation::MemoryPublicationRecord;
use crate::history::{WorldObservedMemoryPublication, WorldReadmittedPublicationDenial as Denial};
use std::sync::{atomic::Ordering, Arc, Mutex, Weak};

#[derive(Debug)]
enum Slot {
    Vacant(Option<usize>),
    Original(Arc<MemoryPublicationRecord>),
}
#[derive(Debug, Default)]
struct Records {
    slots: Vec<Slot>,
    vacant: Option<usize>,
    _capacity_funding: Option<worth_execution::ExecutionMemoryReservation>,
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
impl MemoryOriginalRegistration {
    pub(crate) fn mark_committed(&self, movement: &crate::branch::ProductBranchReferenceMovement) {
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
            drop(records);
            drop(removed);
        }
    }
}
impl CompositeHistoryCatalog {
    pub(crate) fn register_memory_original(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        record: Arc<MemoryPublicationRecord>,
    ) -> Result<
        MemoryOriginalRegistration,
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
        if records.vacant.is_none() && records.slots.len() == records.slots.capacity() {
            let capacity = records
                .slots
                .capacity()
                .checked_mul(2)
                .and_then(|n| n.checked_add(1))
                .ok_or_else(malformed)?;
            let bytes = capacity
                .checked_mul(std::mem::size_of::<Slot>())
                .ok_or_else(malformed)?;
            let funding = execution
                .reserve_retained_memory(bytes as u64)
                .map_err(|cause| Refusal::Request(cause.into()))?;
            let mut charged = None;
            let scope = execution.run_native_with_checkpoint(|context| {
                charged = Some(if context.has_accounting() {
                    context.checkpoint(capacity as u64)
                } else {
                    context.poll()
                });
            });
            match (scope, charged) {
                (Ok(_), Some(Ok(()))) => {}
                (_, Some(Err(cause))) => return Err(Refusal::Request(cause.into())),
                (Err(cause), _) => return Err(Refusal::Request(cause.into())),
                (Ok(_), None) => unreachable!("registry growth charged before allocation"),
            }
            let mut slots = Vec::new();
            slots.try_reserve_exact(capacity).map_err(|_| malformed())?;
            // Vec reports the requested element capacity; refuse a wider backing
            // before retaining it against this request's exact capacity quote.
            if slots.capacity() != capacity {
                return Err(malformed());
            }
            slots.append(&mut records.slots);
            records.slots = slots;
            records._capacity_funding = funding;
        }
        let slot = if let Some(slot) = records.vacant {
            let Slot::Vacant(next) = records.slots[slot] else {
                unreachable!("free slot is vacant")
            };
            records.vacant = next;
            records.slots[slot] = Slot::Original(Arc::clone(&record));
            slot
        } else {
            let slot = records.slots.len();
            records.slots.push(Slot::Original(Arc::clone(&record)));
            slot
        };
        Ok(MemoryOriginalRegistration {
            registry: Arc::downgrade(&self.originals.0),
            slot,
            record,
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
}
