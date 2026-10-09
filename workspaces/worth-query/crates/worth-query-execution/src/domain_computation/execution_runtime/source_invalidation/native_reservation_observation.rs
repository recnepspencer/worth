//! Test-only provenance of the owner ledger's live retained reservations.
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
#[cfg(feature = "test-query-execution-observer")]
#[derive(Clone, Copy)]
pub(in crate::domain_computation) enum NativeRetainedKind {
    Hint,
    Branch,
    Completion,
}
struct Reservation {
    ledger: usize,
    #[cfg(feature = "test-query-execution-observer")]
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
    let prior = live()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(
            address,
            Reservation {
                ledger,
                #[cfg(feature = "test-query-execution-observer")]
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
#[cfg(feature = "test-query-execution-observer")]
pub(super) fn retain(address: usize, ledger: usize, kind: NativeRetainedKind, bytes: u64) {
    let mut live = live()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let reservation = live
        .get_mut(&address)
        .expect("the ledger reservation has provenance");
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
    live()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&address);
}
#[cfg(feature = "test-query-execution-observer")]
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
