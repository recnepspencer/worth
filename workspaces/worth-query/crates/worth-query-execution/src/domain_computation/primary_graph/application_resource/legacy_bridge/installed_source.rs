use super::{WorthQueryOwnedAsyncRuntimeDenial as Denial, WorthQueryOwnedAsyncRuntimeServices};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration;

/// Installed descriptive correspondence. Entry rechecks the actual owner catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledOwnedAsyncSource {
    pub(super) runtime: WorthQueryRuntimeAuthorityIdentity,
    pub(super) graph: u64,
    pub(super) lowered: LoweredBridgeAsyncSourceDeclaration,
}

impl WorthQueryInstalledOwnedAsyncSource {
    pub fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime
    }
}

impl WorthQueryOwnedAsyncRuntimeServices<'_> {
    pub fn installed_source(
        &self,
        lowered: LoweredBridgeAsyncSourceDeclaration,
    ) -> Result<WorthQueryInstalledOwnedAsyncSource, Denial> {
        self.require_installed(&lowered)?;
        Ok(WorthQueryInstalledOwnedAsyncSource {
            runtime: self.runtime,
            graph: self.bridge.owned_signal_graph_instance_id(),
            lowered,
        })
    }

    pub(super) fn require_installed(
        &self,
        lowered: &LoweredBridgeAsyncSourceDeclaration,
    ) -> Result<(), Denial> {
        if self
            .bridge
            .installed_owned_async_declaration(lowered.declaration_identity_for_reporting())
            .as_ref()
            == Some(lowered)
        {
            Ok(())
        } else {
            Err(Denial::SuccessorRuntime)
        }
    }
}
