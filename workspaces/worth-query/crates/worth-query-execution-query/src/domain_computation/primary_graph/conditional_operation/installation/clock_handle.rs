//! Installed conditional clock handle and its retained lease.

use std::{marker::PhantomData, sync::Arc};

/// Handle to one installed temporal conditional binding.
///
/// Returned by `bind_temporal_operation`; pass it to `conditional_clock` to
/// feed that binding clock readings.
pub struct WorthQueryConditionalClockHandle<Schema, Node, Clock> {
    pub binding_identity: Arc<str>,
    pub binding_identity_digest: [u8; 32],
    pub node_authority: Arc<str>,
    pub binding_canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    pub lease: Arc<ConditionalClockLease>,
    pub marker: PhantomData<fn() -> (Schema, Node, Clock)>,
}

pub struct ConditionalClockLease;

impl<Schema, Node, Clock> WorthQueryConditionalClockHandle<Schema, Node, Clock> {
    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
    }

    pub const fn binding_identity_digest(&self) -> &[u8; 32] {
        &self.binding_identity_digest
    }

    pub const fn binding_canonical_work(
        &self,
    ) -> worth_query_installation::facade::WorthQueryCanonicalWorkEvidence {
        self.binding_canonical_work
    }

    pub fn node_authority(&self) -> &Arc<str> {
        &self.node_authority
    }

    pub fn lease(&self) -> &Arc<ConditionalClockLease> {
        &self.lease
    }
}
