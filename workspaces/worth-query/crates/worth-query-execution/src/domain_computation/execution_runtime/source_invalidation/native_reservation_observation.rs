//! Installed observer provenance of the owner ledger's live retained reservations.
use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex, OnceLock,
};
#[derive(Clone, Copy)]
pub(in crate::domain_computation) enum NativeRetainedKind {
    Hint,
    Branch,
    Completion,
}
struct Reservation {
    ledger: usize,
    kind: Option<NativeRetainedKind>,
    site: &'static std::panic::Location<'static>,
    bytes: u64,
}
static LIVE: OnceLock<Mutex<BTreeMap<usize, Reservation>>> = OnceLock::new();
fn live() -> &'static Mutex<BTreeMap<usize, Reservation>> {
    LIVE.get_or_init(Default::default)
}
pub(super) fn observe(
    address: usize,
    ledger: usize,
    bytes: u64,
    site: &'static std::panic::Location<'static>,
) {
    if !native_observation_is_installed() {
        return;
    }
    let mut records = live()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !native_observation_is_installed() {
        return;
    }
    let prior = records.insert(
        address,
        Reservation {
            ledger,
            kind: None,
            site,
            bytes,
        },
    );
    assert!(
        prior.is_none(),
        "one provenance record per live reservation"
    );
}
pub(super) fn retain(address: usize, ledger: usize, kind: NativeRetainedKind, bytes: u64) {
    if !native_observation_is_installed() {
        return;
    }
    let mut live = live()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(reservation) = live.get_mut(&address) else {
        return; // The ticket was issued outside the currently installed capture.
    };
    assert_eq!((reservation.ledger, reservation.bytes), (ledger, bytes));
    assert!(
        reservation.kind.replace(kind).is_none(),
        "one Native class per reservation"
    );
}
pub(super) fn breakdown(ledger: usize) -> Vec<(usize, &'static str, u32, u64)> {
    live()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .filter(|(_, reservation)| reservation.ledger == ledger)
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
pub(super) fn release(address: usize) {
    if !native_observation_is_installed() {
        return;
    }
    live()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&address);
}
pub(super) fn snapshot(ledger: usize) -> [(usize, u64); 3] {
    let mut totals = [(0, 0); 3];
    for reservation in live()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .values()
        .filter(|reservation| reservation.ledger == ledger)
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

static LISTENERS: AtomicUsize = AtomicUsize::new(0);
pub(in crate::domain_computation) struct NativeReservationObservation {
    _installed: (),
}
impl NativeReservationObservation {
    pub(in crate::domain_computation) fn install() -> Self {
        let _records = live()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let count = LISTENERS
            .load(Ordering::Relaxed)
            .checked_add(1)
            .expect("Native listener count overflow");
        LISTENERS.store(count, Ordering::Release);
        Self { _installed: () }
    }
}
impl Drop for NativeReservationObservation {
    fn drop(&mut self) {
        let mut records = live()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remaining = LISTENERS.load(Ordering::Relaxed) - 1;
        LISTENERS.store(remaining, Ordering::Release);
        if remaining == 0 {
            records.clear();
        }
    }
}
pub(in crate::domain_computation) fn native_observation_is_installed() -> bool {
    LISTENERS.load(Ordering::Acquire) != 0
}

#[cfg(test)]
mod observation_tests {
    use super::*;
    #[test]
    fn capture_classifies_only_reservations_it_observed() {
        let addresses = Box::new([0_u8, 0]);
        let address = &addresses[0] as *const u8 as usize;
        let outside_capture = &addresses[1] as *const u8 as usize;
        let ledger = address;
        let _capture = NativeReservationObservation::install();
        observe(address, ledger, 23, std::panic::Location::caller());
        retain(address, ledger, NativeRetainedKind::Hint, 23);
        retain(outside_capture, ledger, NativeRetainedKind::Completion, 11);
        assert_eq!(snapshot(ledger), [(1, 23), (0, 0), (0, 0)]);
        release(address);
        assert_eq!(snapshot(ledger), [(0, 0); 3]);
    }
}
