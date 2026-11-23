//! Weak allocation identity for live metadata bound to a frozen envelope.
use super::CanonicalCommitEnvelope;
use std::sync::{Arc, Weak};

pub(super) struct LiveEnvelopeOwner(Weak<dyn Send + Sync>);

impl LiveEnvelopeOwner {
    pub(super) fn bind(owner: &Weak<CanonicalCommitEnvelope>) -> Self {
        let owner = owner.clone();
        Self(owner)
    }

    pub(super) fn matches(&self, envelope: &Arc<CanonicalCommitEnvelope>) -> bool {
        // The weak share prevents address reuse. The supplied Arc keeps the
        // selected envelope live, so identity needs no owner upgrade.
        std::ptr::addr_eq(self.0.as_ptr(), Arc::as_ptr(envelope))
    }
}
