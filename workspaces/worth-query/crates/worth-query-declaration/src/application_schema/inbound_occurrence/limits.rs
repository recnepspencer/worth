use std::num::NonZeroU64;

/// Finite installed ceilings for receiver work and retained custody.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationInboundOccurrenceLimits {
    pub maximum_envelope_bytes: NonZeroU64,
    /// Maximum trusted verifier work units consumed while authenticating one envelope.
    pub maximum_verifier_work: NonZeroU64,
    pub maximum_payload_bytes: NonZeroU64,
    /// Co-committed outbound effects retained before any callback is accepted.
    pub maximum_outstanding_dispatch_provenance: NonZeroU64,
    pub maximum_accepted_occurrences: NonZeroU64,
    pub maximum_accepted_bytes: NonZeroU64,
    pub maximum_concurrent_publications: NonZeroU64,
    pub maximum_discovery_work: NonZeroU64,
    pub replay_window_milliseconds: NonZeroU64,
    pub maximum_cleanup_work: NonZeroU64,
}

impl ApplicationInboundOccurrenceLimits {
    pub const fn accommodates_payload(&self) -> bool {
        self.maximum_payload_bytes.get() <= self.maximum_envelope_bytes.get()
            && self.maximum_payload_bytes.get() <= self.maximum_accepted_bytes.get()
    }
}
