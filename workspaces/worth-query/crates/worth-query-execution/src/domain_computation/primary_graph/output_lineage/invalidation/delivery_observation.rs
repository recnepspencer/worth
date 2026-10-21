//! Certification observation of native deliveries prepared without exact marks.

use std::cell::Cell;

use super::logical_marking::NativeMarkingReport;
use super::logical_marking::WorthQueryNativeMarkingPrecision;

thread_local! {
    static INEXACT_DELIVERIES: Cell<u64> = const { Cell::new(0) };
}

pub(super) fn record_delivery(report: &NativeMarkingReport) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if matches!(report.precision, WorthQueryNativeMarkingPrecision::Exact(_)) {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
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

impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
}

// Law 2, scope: passive counts end with their owned observation guard.
thread_local! { static CLEANUP_REGISTERED: Cell<bool> = const { Cell::new(false) }; }
fn clear_observations() {
    CLEANUP_REGISTERED.set(false);
    INEXACT_DELIVERIES.set(0);
}
