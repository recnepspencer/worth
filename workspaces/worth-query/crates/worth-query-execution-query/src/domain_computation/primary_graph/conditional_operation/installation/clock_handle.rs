//! Installed conditional clock handle and its retained lease.

use std::{marker::PhantomData, sync::Arc};

/// Handle to one installed temporal conditional binding.
///
/// Returned by `bind_temporal_operation`; pass it to `conditional_clock` to
/// feed that binding clock readings.
pub struct WorthQueryConditionalClockHandle<Schema, Node, Clock> {
    pub(crate) binding_identity: Arc<str>,
    pub(crate) binding_identity_digest: [u8; 32],
    pub(crate) node_authority: Arc<str>,
    pub(crate) binding_canonical_work:
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    pub(crate) lease: Arc<ConditionalClockLease>,
    pub(crate) marker: PhantomData<fn() -> (Schema, Node, Clock)>,
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

/// Crate-boundary port for `WorthQueryConditionalClockHandle` construction.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait ConditionalClockHandleInternalPort<Schema, Node, Clock>: Sized {
    fn from_parts(
        binding_identity: Arc<str>,
        binding_identity_digest: [u8; 32],
        node_authority: Arc<str>,
        binding_canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
        lease: Arc<ConditionalClockLease>,
    ) -> Self;

    fn new_lease() -> Arc<ConditionalClockLease>;
}

impl<Schema, Node, Clock> ConditionalClockHandleInternalPort<Schema, Node, Clock>
    for WorthQueryConditionalClockHandle<Schema, Node, Clock>
{
    fn from_parts(
        binding_identity: Arc<str>,
        binding_identity_digest: [u8; 32],
        node_authority: Arc<str>,
        binding_canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
        lease: Arc<ConditionalClockLease>,
    ) -> Self {
        Self {
            binding_identity,
            binding_identity_digest,
            node_authority,
            binding_canonical_work,
            lease,
            marker: PhantomData,
        }
    }

    fn new_lease() -> Arc<ConditionalClockLease> {
        Arc::new(ConditionalClockLease)
    }
}
