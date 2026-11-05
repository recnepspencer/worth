//! Moved whole from `domain_computation::primary_graph::application_attempt::external_dispatch_attempt`: the items below and every impl they own.

/// Opaque, move-only ordinal minted only while the runtime admits an attempt.
pub(in crate::domain_computation) struct WorthQueryExternalDispatchAttemptOrdinal(u64);

impl WorthQueryExternalDispatchAttemptOrdinal {
    fn mint(value: u64) -> Self {
        Self(value)
    }

    pub(in crate::domain_computation) fn into_value(self) -> u64 {
        self.0
    }

    pub(in crate::domain_computation) const fn value_for_test(&self) -> u64 {
        self.0
    }
}
