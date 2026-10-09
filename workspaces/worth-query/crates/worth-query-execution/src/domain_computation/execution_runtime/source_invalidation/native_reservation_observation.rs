//! Observer-only provenance of live Native hint reservations. Accounting is unchanged.
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
#[derive(Clone, Copy)]
pub(in crate::domain_computation) enum NativeRetainedKind {
    Hint,
    Branch,
    Completion,
}
struct Reservation {
    ledger: usize,
    kind: NativeRetainedKind,
    bytes: u64,
}
static LIVE: OnceLock<Mutex<BTreeMap<usize, Reservation>>> = OnceLock::new();
fn live() -> &'static Mutex<BTreeMap<usize, Reservation>> {
    LIVE.get_or_init(Default::default)
}
pub(super) fn retain(address: usize, ledger: usize, kind: NativeRetainedKind, bytes: u64) {
    let prior = live()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(
            address,
            Reservation {
                ledger,
                kind,
                bytes,
            },
        );
    assert!(
        prior.is_none(),
        "one provenance record per live reservation"
    );
}
pub(super) fn release(address: usize) {
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
            NativeRetainedKind::Hint => 0,
            NativeRetainedKind::Branch => 1,
            NativeRetainedKind::Completion => 2,
        };
        totals[index].0 += 1;
        totals[index].1 += reservation.bytes;
    }
    totals
}
