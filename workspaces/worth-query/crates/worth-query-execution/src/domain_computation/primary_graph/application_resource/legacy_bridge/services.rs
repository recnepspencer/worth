use super::{WorthQueryOwnedAsyncRequestCustody, WorthQueryOwnedAsyncRuntimeDenial as Denial};
use crate::domain_computation::execution_runtime::{
    product_world::WorthQueryProductRuntime, WorthQueryRuntimeAuthorityIdentity,
};
use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

/// Borrows the actual World and Bridge of the old engine's installed product.
/// This creates neither a second runtime nor a request lifecycle.
pub struct WorthQueryOwnedAsyncRuntimeServices<'runtime> {
    pub(super) runtime: WorthQueryRuntimeAuthorityIdentity,
    pub(super) world: &'runtime WorthQueryProductRuntime,
    pub(super) bridge: &'runtime BridgeSealedRuntimeAssembly,
}

impl<'runtime> WorthQueryOwnedAsyncRuntimeServices<'runtime> {
    pub fn new(
        runtime: WorthQueryRuntimeAuthorityIdentity,
        world: &'runtime WorthQueryProductRuntime,
        bridge: &'runtime BridgeSealedRuntimeAssembly,
    ) -> Self {
        Self {
            runtime,
            world,
            bridge,
        }
    }

    pub(super) fn require_custody(
        &self,
        custody: &WorthQueryOwnedAsyncRequestCustody,
    ) -> Result<(), Denial> {
        if custody.runtime != self.runtime {
            return Err(Denial::ForeignRuntime);
        }
        if custody.graph != self.bridge.owned_signal_graph_instance_id() {
            return Err(Denial::SuccessorRuntime);
        }
        Ok(())
    }

    pub(super) fn require_selected(
        &self,
        selected: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<(), Denial> {
        if self
            .world
            .integration_owns_product_branch(selected.product_branch())
        {
            Ok(())
        } else {
            Err(Denial::ProductSelectionMismatch)
        }
    }
}
