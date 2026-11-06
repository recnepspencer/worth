//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::primary_graph::snapshot_lease::WorthQueryApplicationSnapshotLeaseDenial;

impl WorthQueryApplicationSnapshotLease {
    #[doc(hidden)]
    pub fn handle_for_test(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        Self::handle(self)
    }
}

impl WorthQueryApplicationSnapshotLease {
    #[doc(hidden)]
    pub fn snapshot_for_test(&self) -> &SnapshotHandle {
        Self::snapshot(self)
    }
}

impl WorthQueryApplicationSnapshotLease {
    #[doc(hidden)]
    pub fn layout_for_test(
        &self,
    ) -> &std::sync::Arc<crate::domain_computation::graph_layout::WorthQueryPrimaryGraphLayout>
    {
        &self.layout
    }
}

impl WorthQueryApplicationSnapshotLease {
    #[doc(hidden)]
    pub fn acquire_for_test(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        layout: std::sync::Arc<
            crate::domain_computation::graph_layout::WorthQueryPrimaryGraphLayout,
        >,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Self, WorthQueryApplicationSnapshotLeaseDenial> {
        Self::acquire(execution, handle, layout, product)
    }
}

impl WorthQueryApplicationSnapshotLease {
    #[doc(hidden)]
    pub fn release_for_test(self) -> bool {
        Self::release(self)
    }
}
