//! Certification observation of native deliveries prepared without exact marks.

use std::cell::Cell;

use super::logical_marking::{NativeMarkingPrecision, NativeMarkingReport};

thread_local! {
    static INEXACT_DELIVERIES: Cell<u64> = const { Cell::new(0) };
}

pub(super) fn record_delivery(report: &NativeMarkingReport) {
    if matches!(report.precision, NativeMarkingPrecision::Exact(_)) {
        return;
    }
    INEXACT_DELIVERIES.with(|deliveries| {
        deliveries.set(
            deliveries
                .get()
                .checked_add(1)
                .expect("inexact delivery counter overflow"),
        );
    });
}

/// Prepared native deliveries on this thread whose marks degraded to a
/// declared-change discontinuity. A refused or aborted publication may still
/// have been counted; other threads have independent counts.
#[doc(hidden)]
pub fn inexact_native_deliveries_on_this_thread_for_test() -> u64 {
    INEXACT_DELIVERIES.with(Cell::get)
}
