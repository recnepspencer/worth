//! Opt-in, bounded, runtime-bound reservation evidence on the observing thread.
use super::*;
use crate::identity::data::{EntityId, RelationId};
use crate::transactions::data::RecordRef;
use std::{
    cell::RefCell,
    rc::{Rc, Weak as LocalWeak},
};
use worth_execution::{
    ExecutionAllocationDenial, ExecutionAllocationPolicy, ExecutionByteBuffer,
    ExecutionImmutableBytes,
};
const RECORD_BYTES: usize = 1 + 4 + 8 + 4;
thread_local! {
    static OBSERVERS: RefCell<Vec<LocalWeak<RefCell<Capture>>>> = const { RefCell::new(Vec::new()) };
}
struct Capture {
    authority: std::sync::Weak<Mutex<RecordIdentityState>>,
    bytes: ExecutionByteBuffer,
    maximum_records: usize,
    failure: Option<RecordReservationObservationDenial>,
}
#[derive(Clone, Debug)]
/// Why complete reservation evidence is unavailable. Allocation failures
/// retain the original request/physical cause instead of becoming capacity.
pub enum RecordReservationObservationDenial {
    CapacityExceeded { maximum_records: usize },
    Allocation(ExecutionAllocationDenial),
}
impl From<ExecutionAllocationDenial> for RecordReservationObservationDenial {
    fn from(value: ExecutionAllocationDenial) -> Self {
        Self::Allocation(value)
    }
}
/// An owned read-only observer. It is deliberately thread-bound. It records
/// actual successful reservations in this runtime, including proposals later
/// refused, never inferred checkpoint deltas. It changes no allocator policy.
/// Dropping it stops observation, including during unwinding. Over-capacity or
/// allocation failure makes the report unavailable; partial evidence is never
/// returned as complete. Nested observers and distinct runtimes are isolated.
pub struct RecordReservationObservation {
    capture: Rc<RefCell<Capture>>,
}
/// Sealed immutable coordinates of actual reservations in one observed prefix.
/// A complete report grants no reservation, reuse or publication authority.
pub struct RecordReservationReport {
    bytes: ExecutionImmutableBytes,
}
impl std::fmt::Debug for RecordReservationObservation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordReservationObservation")
            .finish_non_exhaustive()
    }
}
impl std::fmt::Debug for RecordReservationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordReservationReport")
            .field("records", &(self.bytes.len() / RECORD_BYTES))
            .finish()
    }
}
impl RecordIdentitySubsystem {
    pub(crate) fn observe_reservations(
        &self,
        maximum_records: usize,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<RecordReservationObservation, RecordReservationObservationDenial> {
        let total = maximum_records
            .checked_mul(RECORD_BYTES)
            .ok_or(RecordReservationObservationDenial::CapacityExceeded { maximum_records })?;
        let capture = Rc::new(RefCell::new(Capture {
            authority: Arc::downgrade(&self.state),
            bytes: ExecutionByteBuffer::allocate(total, policy)?,
            maximum_records,
            failure: None,
        }));
        OBSERVERS.with_borrow_mut(|observers| {
            observers.retain(|observer| observer.strong_count() != 0);
            observers.push(Rc::downgrade(&capture));
        });
        Ok(RecordReservationObservation { capture })
    }
}
impl RecordReservationObservation {
    /// Snapshot the complete observed prefix. O(reservations) copying into
    /// caller-funded immutable backing; the observer continues until dropped.
    pub fn report(
        &self,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<RecordReservationReport, RecordReservationObservationDenial> {
        let captured = self.capture.borrow();
        if let Some(failure) = &captured.failure {
            return Err(failure.clone());
        }
        let mut bytes = ExecutionByteBuffer::allocate(captured.bytes.len(), policy)?;
        bytes.extend_from_slice(captured.bytes.bytes())?;
        Ok(RecordReservationReport {
            bytes: bytes.seal()?,
        })
    }
}
impl RecordReservationReport {
    pub fn records(&self) -> impl ExactSizeIterator<Item = RecordRef> + '_ {
        self.bytes
            .as_chunks::<RECORD_BYTES>()
            .0
            .iter()
            .map(|record| {
                let partition = PartitionId(u32::from_be_bytes(
                    record[1..5].try_into().expect("owned partition width"),
                ));
                let slot = u64::from_be_bytes(record[5..13].try_into().expect("owned slot width"));
                let generation =
                    u32::from_be_bytes(record[13..17].try_into().expect("owned generation width"));
                match record[0] {
                    0 => RecordRef::Entity(EntityId::new(partition, slot, generation)),
                    1 => RecordRef::Relation(RelationId::new(partition, slot, generation)),
                    _ => unreachable!("private reservation report tag"),
                }
            })
    }
}
impl Drop for RecordReservationObservation {
    fn drop(&mut self) {
        let selected = Rc::downgrade(&self.capture);
        OBSERVERS
            .with_borrow_mut(|observers| observers.retain(|observer| !observer.ptr_eq(&selected)));
    }
}
pub(super) fn reserved(
    authority: &Arc<Mutex<RecordIdentityState>>,
    class: RecordAllocationClass,
    partition_id: PartitionId,
    slot: usize,
    generation: u32,
) {
    OBSERVERS.with_borrow(|observers| {
        for observer in observers {
            let Some(observer) = observer.upgrade() else {
                continue;
            };
            let mut captured = observer.borrow_mut();
            if !captured.authority.ptr_eq(&Arc::downgrade(authority)) || captured.failure.is_some()
            {
                continue;
            }
            if captured.bytes.len() / RECORD_BYTES == captured.maximum_records {
                captured.failure = Some(RecordReservationObservationDenial::CapacityExceeded {
                    maximum_records: captured.maximum_records,
                });
                continue;
            }
            let mut bytes = [0; RECORD_BYTES];
            bytes[0] = match class {
                RecordAllocationClass::Entity => 0,
                RecordAllocationClass::Relation => 1,
            };
            bytes[1..5].copy_from_slice(&partition_id.0.to_be_bytes());
            bytes[5..13].copy_from_slice(
                &u64::try_from(slot)
                    .expect("native slot fits its public identity")
                    .to_be_bytes(),
            );
            bytes[13..17].copy_from_slice(&generation.to_be_bytes());
            if let Err(error) = captured.bytes.extend_from_slice(&bytes) {
                captured.failure = Some(error.into());
            }
        }
    });
}

#[cfg(test)]
mod tests;
