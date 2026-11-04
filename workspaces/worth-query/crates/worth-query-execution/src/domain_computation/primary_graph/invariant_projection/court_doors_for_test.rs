//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryApplicationInvariantProjectionSnapshot<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn into_lease_and_realized_scope_for_test(
        self,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> (
        super::super::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease,
        WorthQueryRealizedProjectionScope,
        Option<super::super::application_attempt::retained_decision_facts::RetainedSourceFacts>,
        Vec<ConsumedOutputEvidence>,
    ) {
        Self::into_lease_and_realized_scope(self, product)
    }
}

impl<Schema, Entity> WorthQueryInvariantEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn authority_identity_for_test(&self) -> &u64 {
        &self.authority_identity
    }
}

impl<Schema, Entity> WorthQueryInvariantEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn entity_for_test(&self) -> &Arc<str> {
        &self.entity
    }
}

impl<Schema, Entity> WorthQueryInvariantEntityIdentity<Schema, Entity> {
    #[doc(hidden)]
    pub fn entity_id_for_test(&self) -> &EntityId {
        &self.entity_id
    }
}

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    #[doc(hidden)]
    pub fn graph_for_test(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        &self.graph
    }
}

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    #[doc(hidden)]
    pub fn layout_for_test(&self) -> &Arc<WorthQueryPrimaryGraphLayout> {
        &self.layout
    }
}

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn active_snapshot_count_for_test(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> usize {
        Self::active_snapshot_count(self, request_execution)
    }
}
