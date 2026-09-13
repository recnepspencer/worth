use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, ThreadId};

use arc_swap::ArcSwapOption;

use super::lifecycle_state::MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS;

const ADMISSION_IDLE: u8 = 0;
const ADMISSION_HOLDS_CELL: u8 = 1;
const ADMISSION_HOLDS_METADATA: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SignalOwnerAdmissionHoldDenial {
    AdmissionAlreadyHoldsOwnerState,
    ExecutingThreadReentry,
}

#[derive(Debug)]
pub(super) struct SignalOwnerAdmissionRecord {
    thread_id: ThreadId,
    hold_posture: AtomicU8,
}

#[derive(Debug)]
pub(super) struct SignalOwnerAdmissionTable {
    records: [ArcSwapOption<SignalOwnerAdmissionRecord>; MAXIMUM_IN_FLIGHT_SIGNAL_OWNER_OPERATIONS],
    publication: Mutex<()>,
}

impl SignalOwnerAdmissionTable {
    pub(super) fn new() -> Arc<Self> {
        Arc::new(Self {
            records: std::array::from_fn(|_| ArcSwapOption::empty()),
            publication: Mutex::new(()),
        })
    }

    pub(super) fn new_record(&self) -> Arc<SignalOwnerAdmissionRecord> {
        Arc::new(SignalOwnerAdmissionRecord {
            thread_id: thread::current().id(),
            hold_posture: AtomicU8::new(ADMISSION_IDLE),
        })
    }

    pub(super) fn publish(
        self: &Arc<Self>,
        record: Arc<SignalOwnerAdmissionRecord>,
    ) -> (SignalOwnerPublishedAdmission, usize) {
        let _publication = self.lock_publication();
        for (slot_index, slot) in self.records.iter().enumerate() {
            if slot.load().is_none() {
                slot.store(Some(Arc::clone(&record)));
                return (
                    SignalOwnerPublishedAdmission {
                        table: Arc::clone(self),
                        slot_index,
                        record: Some(record),
                        _thread_affinity: PhantomData,
                    },
                    slot_index + 1,
                );
            }
        }
        unreachable!("a reserved Signal owner admission always has one of 64 slots")
    }

    pub(super) fn executing_thread_has_admission(&self) -> (bool, usize) {
        let executing_thread = thread::current().id();
        for (index, slot) in self.records.iter().enumerate() {
            let record = slot.load();
            if record
                .as_ref()
                .is_some_and(|record| record.thread_id == executing_thread)
            {
                return (true, index + 1);
            }
        }
        (false, self.records.len())
    }

    pub(super) fn executing_thread_has_owner_hold(&self) -> (bool, usize) {
        let executing_thread = thread::current().id();
        for (index, slot) in self.records.iter().enumerate() {
            let record = slot.load();
            if record.as_ref().is_some_and(|record| {
                record.thread_id == executing_thread
                    && record.hold_posture.load(Ordering::Acquire) != ADMISSION_IDLE
            }) {
                return (true, index + 1);
            }
        }
        (false, self.records.len())
    }

    fn begin_hold<'a>(
        &'a self,
        slot_index: usize,
        record: &'a SignalOwnerAdmissionRecord,
        posture: u8,
    ) -> Result<(SignalOwnerAdmissionHold<'a>, usize), (SignalOwnerAdmissionHoldDenial, usize)>
    {
        if record
            .hold_posture
            .compare_exchange(
                ADMISSION_IDLE,
                posture,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            return Err((
                SignalOwnerAdmissionHoldDenial::AdmissionAlreadyHoldsOwnerState,
                0,
            ));
        }

        for (index, slot) in self.records.iter().enumerate() {
            if index == slot_index {
                continue;
            }
            let other = slot.load();
            if other.as_ref().is_some_and(|other| {
                other.thread_id == record.thread_id
                    && other.hold_posture.load(Ordering::Acquire) != ADMISSION_IDLE
            }) {
                record.hold_posture.store(ADMISSION_IDLE, Ordering::Release);
                return Err((
                    SignalOwnerAdmissionHoldDenial::ExecutingThreadReentry,
                    index + 1,
                ));
            }
        }
        Ok((
            SignalOwnerAdmissionHold {
                record,
                posture,
                _thread_affinity: PhantomData,
            },
            self.records.len(),
        ))
    }

    fn admission_can_acquire_owner_lock(
        &self,
        slot_index: usize,
        record: &SignalOwnerAdmissionRecord,
    ) -> Result<((), usize), (SignalOwnerAdmissionHoldDenial, usize)> {
        if record.hold_posture.load(Ordering::Acquire) != ADMISSION_IDLE {
            return Err((
                SignalOwnerAdmissionHoldDenial::AdmissionAlreadyHoldsOwnerState,
                0,
            ));
        }
        for (index, slot) in self.records.iter().enumerate() {
            if index == slot_index {
                continue;
            }
            let other = slot.load();
            if other.as_ref().is_some_and(|other| {
                other.thread_id == record.thread_id
                    && other.hold_posture.load(Ordering::Acquire) != ADMISSION_IDLE
            }) {
                return Err((
                    SignalOwnerAdmissionHoldDenial::ExecutingThreadReentry,
                    index + 1,
                ));
            }
        }
        Ok(((), self.records.len()))
    }

    fn unpublish(&self, slot_index: usize, record: &SignalOwnerAdmissionRecord) -> usize {
        let _publication = self.lock_publication();
        let published = self.records[slot_index].load_full();
        debug_assert!(published
            .as_ref()
            .is_some_and(|published| std::ptr::eq(Arc::as_ptr(published), record)));
        self.records[slot_index].store(None);
        1
    }

    fn lock_publication(&self) -> MutexGuard<'_, ()> {
        self.publication
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Debug)]
pub(super) struct SignalOwnerPublishedAdmission {
    table: Arc<SignalOwnerAdmissionTable>,
    slot_index: usize,
    record: Option<Arc<SignalOwnerAdmissionRecord>>,
    _thread_affinity: PhantomData<Rc<()>>,
}

impl SignalOwnerPublishedAdmission {
    pub(super) fn hold_cell(
        &self,
    ) -> Result<(SignalOwnerAdmissionHold<'_>, usize), (SignalOwnerAdmissionHoldDenial, usize)>
    {
        self.table
            .begin_hold(self.slot_index, self.record(), ADMISSION_HOLDS_CELL)
    }

    pub(super) fn hold_metadata(
        &self,
    ) -> Result<(SignalOwnerAdmissionHold<'_>, usize), (SignalOwnerAdmissionHoldDenial, usize)>
    {
        self.table
            .begin_hold(self.slot_index, self.record(), ADMISSION_HOLDS_METADATA)
    }

    pub(super) fn can_acquire_owner_lock(
        &self,
    ) -> Result<((), usize), (SignalOwnerAdmissionHoldDenial, usize)> {
        self.table
            .admission_can_acquire_owner_lock(self.slot_index, self.record())
    }

    pub(super) fn is_idle(&self) -> bool {
        self.record().hold_posture.load(Ordering::Acquire) == ADMISSION_IDLE
    }

    pub(super) fn unpublish(&mut self) -> usize {
        let record = self
            .record
            .take()
            .expect("a Signal owner admission is unpublished exactly once");
        self.table.unpublish(self.slot_index, &record)
    }

    fn record(&self) -> &SignalOwnerAdmissionRecord {
        self.record
            .as_deref()
            .expect("a live Signal owner admission remains published")
    }
}

pub(super) struct SignalOwnerAdmissionHold<'a> {
    record: &'a SignalOwnerAdmissionRecord,
    posture: u8,
    _thread_affinity: PhantomData<Rc<()>>,
}

impl Drop for SignalOwnerAdmissionHold<'_> {
    fn drop(&mut self) {
        let released = self
            .record
            .hold_posture
            .swap(ADMISSION_IDLE, Ordering::Release);
        debug_assert_eq!(released, self.posture);
    }
}
