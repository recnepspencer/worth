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

thread_local! {
    static EXHAUST_RETAINED_DELIVERY: Cell<bool> = const { Cell::new(false) };
}
pub(super) fn exhaust_delivery_capacity(
    resources: &crate::domain_computation::execution_runtime::WorthQueryInvalidationResources,
) -> Option<
    crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity,
> {
    EXHAUST_RETAINED_DELIVERY
        .with(|armed| armed.replace(false))
        .then(|| {
            let remaining = resources.installation().maximum_retained_bytes
                - resources.retained_capacity_bytes();
            resources
                .reserve_retained_capacity(remaining)
                .expect("exact remaining ledger capacity")
        })
}
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Hold the ledger's remaining real capacity during one source delivery.
    #[doc(hidden)]
    pub fn exhaust_next_source_delivery_capacity_for_test(&self) {
        EXHAUST_RETAINED_DELIVERY.with(|armed| armed.set(true));
    }
}
