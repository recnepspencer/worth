//! Law 2, scope: live records belong to the receiver's retention ledger.
use super::RetainedInvalidationCapacity;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, Weak,
    },
};
#[derive(Clone, Copy, Debug)]
pub(in crate::domain_computation) enum NativeRetainedKind {
    Hint,
    Branch,
    Completion,
}
#[derive(Debug)]
struct Reservation {
    kind: Option<NativeRetainedKind>,
    site: &'static std::panic::Location<'static>,
    bytes: u64,
}
#[derive(Debug, Default)]
struct Records {
    reservations: BTreeMap<usize, Reservation>,
    consumed: BTreeMap<u64, Vec<Weak<RetainedInvalidationCapacity>>>,
}
#[derive(Debug, Default)]
pub(super) struct NativeReservationRecorder {
    active: AtomicBool,
    records: Mutex<Records>,
}
impl NativeReservationRecorder {
    pub(super) const fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            records: Mutex::new(Records {
                reservations: BTreeMap::new(),
                consumed: BTreeMap::new(),
            }),
        }
    }
    pub(super) fn install(&self) {
        self.active.store(true, Ordering::Release);
    }
    pub(super) fn clear(&self) {
        self.active.store(false, Ordering::Release);
        *self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Records::default();
    }
    pub(super) fn observe(
        &self,
        address: usize,
        bytes: u64,
        site: &'static std::panic::Location<'static>,
    ) {
        if !self.active.load(Ordering::Acquire) {
            return;
        }
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !self.active.load(Ordering::Acquire) {
            return;
        }
        assert!(
            records
                .reservations
                .insert(
                    address,
                    Reservation {
                        kind: None,
                        site,
                        bytes
                    }
                )
                .is_none(),
            "one provenance record per live reservation"
        );
    }
    pub(super) fn retain(&self, address: usize, kind: NativeRetainedKind, bytes: u64) {
        if !self.active.load(Ordering::Acquire) {
            return;
        }
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !self.active.load(Ordering::Acquire) {
            return;
        }
        let reservation = records
            .reservations
            .get_mut(&address)
            .expect("Native classification requires an observed live reservation");
        assert_eq!(reservation.bytes, bytes);
        assert!(
            reservation.kind.replace(kind).is_none(),
            "one Native class per reservation"
        );
    }
    pub(super) fn release(&self, address: usize) {
        if !self.active.load(Ordering::Acquire) {
            return;
        }
        self.records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .reservations
            .remove(&address);
    }
    pub(super) fn breakdown(&self) -> Vec<(usize, &'static str, u32, u64)> {
        self.records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .reservations
            .iter()
            .map(|(address, reservation)| {
                (
                    *address,
                    reservation.site.file(),
                    reservation.site.line(),
                    reservation.bytes,
                )
            })
            .collect()
    }
    pub(super) fn snapshot(&self) -> [(usize, u64); 3] {
        let mut totals = [(0, 0); 3];
        for reservation in self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .reservations
            .values()
        {
            let index = match reservation.kind {
                Some(NativeRetainedKind::Hint) => 0,
                Some(NativeRetainedKind::Branch) => 1,
                Some(NativeRetainedKind::Completion) => 2,
                None => continue,
            };
            totals[index].0 += 1;
            totals[index].1 += reservation.bytes;
        }
        totals
    }
    pub(super) fn observe_consumed(
        &self,
        runtime: u64,
        ticket: &Arc<RetainedInvalidationCapacity>,
    ) {
        if !self.active.load(Ordering::Acquire) {
            return;
        }
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !self.active.load(Ordering::Acquire) {
            return;
        }
        if let Some(tickets) = records.consumed.get_mut(&runtime) {
            tickets.push(Arc::downgrade(ticket));
        }
    }
    pub(super) fn consumed(&self, runtime: u64) -> (usize, u64) {
        if !self.active.load(Ordering::Acquire) {
            return (0, 0);
        }
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tickets = records.consumed.entry(runtime).or_default();
        let mut totals = (0, 0);
        tickets.retain(|ticket| match ticket.upgrade() {
            Some(ticket) => {
                totals.0 += 1;
                totals.1 += ticket.bytes();
                true
            }
            None => false,
        });
        totals
    }
}
#[cfg(test)]
mod observation_tests {
    use super::*;
    #[test]
    fn capture_classifies_only_reservations_it_observed() {
        let recorder = NativeReservationRecorder::new();
        recorder.install();
        recorder.observe(1, 23, std::panic::Location::caller());
        recorder.retain(1, NativeRetainedKind::Hint, 23);
        assert!(std::panic::catch_unwind(|| recorder.retain(
            2,
            NativeRetainedKind::Completion,
            11
        ))
        .is_err());
        assert_eq!(recorder.snapshot(), [(1, 23), (0, 0), (0, 0)]);
        recorder.release(1);
        assert_eq!(recorder.snapshot(), [(0, 0); 3]);
    }
    #[test]
    fn an_armed_receiver_does_not_record_another_receivers_work() {
        let armed = NativeReservationRecorder::new();
        let ordinary = NativeReservationRecorder::new();
        armed.install();
        ordinary.observe(1, 7, std::panic::Location::caller());
        ordinary.retain(1, NativeRetainedKind::Hint, 7);
        assert!(ordinary.breakdown().is_empty());
        assert!(armed.breakdown().is_empty());
    }
    #[test]
    fn dropping_the_observer_during_unwind_clears_the_receiver() {
        use crate::domain_computation::execution_observation::WorthQueryExecutionObservation;
        let ledger = std::cell::RefCell::new(None);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = WorthQueryExecutionObservation::install_on_this_thread();
            let captured = super::super::retention::InvalidationRetentionLedger::install(100);
            captured
                .native
                .observe(1, 7, std::panic::Location::caller());
            *ledger.borrow_mut() = Some(captured);
            panic!("exercise owned recorder cleanup");
        }))
        .is_err());
        let captured = ledger.into_inner().unwrap();
        assert!(captured.native.breakdown().is_empty());
        captured
            .native
            .observe(2, 7, std::panic::Location::caller());
        assert!(captured.native.breakdown().is_empty());
    }
}
