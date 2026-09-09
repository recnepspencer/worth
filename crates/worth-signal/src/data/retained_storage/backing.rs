use std::ops::Deref;

use super::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
    SignalConditionalRetentionReservation,
};

/// A persistent base and the resource custody attributable to its conversion.
/// Sharing the enclosing Arc shares custody; materialization creates a different
/// allocation. Empty-overlay conversion cost may remain attributed to this base
/// until its last reference drops. The reservation never retains SignalOwner.
pub(crate) struct RetainedStorageBacking<T> {
    value: T,
    _custody: Option<SignalConditionalRetentionReservation>,
}

impl<T> RetainedStorageBacking<T> {
    pub(crate) fn new(value: T, custody: Option<SignalConditionalRetentionReservation>) -> Self {
        Self {
            value,
            _custody: custody,
        }
    }
}

impl<T> Deref for RetainedStorageBacking<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: PartialEq> PartialEq for RetainedStorageBacking<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T: Eq> Eq for RetainedStorageBacking<T> {}

impl<T: std::fmt::Debug> std::fmt::Debug for RetainedStorageBacking<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(formatter)
    }
}

impl<T: RetainedStorageMeasurement> RetainedStorageMeasurement for RetainedStorageBacking<T> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        // The enclosing Arc charge includes the embedded reservation's bytes.
        // Its ledger is an external resource owner, not retained payload storage.
        self.value.retained_heap_charge(work)
    }
}
