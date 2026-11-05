//! Moved whole from `domain_computation::primary_graph::conditional_operation::installation::clock_handle`: the items below and every impl they own.
use std::marker::PhantomData;
use std::sync::Arc;

/// Handle to one installed temporal conditional binding.
///
/// Returned by `bind_temporal_operation`; pass it to `conditional_clock` to
/// feed that binding clock readings.
pub struct WorthQueryConditionalClockHandle<Schema, Node, Clock> {
    pub(in crate::domain_computation::primary_graph) binding_identity: Arc<str>,
    pub(in crate::domain_computation::primary_graph) binding_identity_digest: [u8; 32],
    pub(in crate::domain_computation::primary_graph) node_authority: Arc<str>,
    pub(in crate::domain_computation::primary_graph) binding_canonical_work:
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    pub(in crate::domain_computation::primary_graph) lease: Arc<ConditionalClockLease>,
    pub(in crate::domain_computation::primary_graph) marker: PhantomData<fn() -> (Schema, Node, Clock)>,
}

pub struct ConditionalClockLease;

impl<Schema, Node, Clock> WorthQueryConditionalClockHandle<Schema, Node, Clock> {
    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
    }

    pub(in crate::domain_computation::primary_graph) const fn binding_identity_digest(
        &self,
    ) -> &[u8; 32] {
        &self.binding_identity_digest
    }

    pub const fn binding_canonical_work(
        &self,
    ) -> worth_query_installation::facade::WorthQueryCanonicalWorkEvidence {
        self.binding_canonical_work
    }

    pub(in crate::domain_computation::primary_graph) fn node_authority(&self) -> &Arc<str> {
        &self.node_authority
    }

    pub(in crate::domain_computation::primary_graph) fn lease(
        &self,
    ) -> &Arc<ConditionalClockLease> {
        &self.lease
    }
}
