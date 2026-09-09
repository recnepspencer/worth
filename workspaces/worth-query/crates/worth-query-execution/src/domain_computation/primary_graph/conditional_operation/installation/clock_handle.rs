//! Installed conditional clock handle and its retained lease.

use std::{marker::PhantomData, sync::Arc};

pub struct WorthQueryConditionalClockHandle<Schema, Node, Clock> {
    pub(super) binding_identity: Arc<str>,
    pub(super) node_authority: Arc<str>,
    pub(super) binding_canonical_work:
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    pub(super) lease: Arc<ConditionalClockLease>,
    pub(super) marker: PhantomData<fn() -> (Schema, Node, Clock)>,
}

pub(in crate::domain_computation::primary_graph) struct ConditionalClockLease;

impl<Schema, Node, Clock> WorthQueryConditionalClockHandle<Schema, Node, Clock> {
    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
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
