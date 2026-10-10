//! Law 2, scope: weak consumed-ticket records live on the receiver's ledger.
use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use std::sync::Arc;
pub(super) fn observe(runtime: u64, ticket: &Arc<RetainedInvalidationCapacity>) {
    ticket.observe_consumed(runtime);
}
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Enables passive weak observation of this receiver's consumed-edge tickets.
    #[doc(hidden)]
    pub fn consumed_output_custody_for_test(&self) -> (usize, u64) {
        let owner = &self.primary_provider.graph.source_owner.invalidation_owner;
        owner
            .resources
            .consumed_capacity_observation(owner.runtime_instance_id)
    }
}
