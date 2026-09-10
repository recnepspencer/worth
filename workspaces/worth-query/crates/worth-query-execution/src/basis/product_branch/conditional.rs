use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeConditionalDenial, BridgeConditionalSignalBasisBinding,
    BridgeInstalledConditionalLowering, BridgeSealedRuntimeAssembly,
};

use super::WorthQueryProductBranchLease;

impl WorthQueryProductBranchLease {
    #[doc(hidden)]
    pub fn prepare_conditional_reconstitution(
        &self,
        bridge: &BridgeSealedRuntimeAssembly,
    ) -> Result<
        worth_runtime_bridge::facade::BridgePreparedConditionalReconstitution,
        BridgeConditionalDenial,
    > {
        bridge.prepare_conditional_reconstitution(self.signal_basis())
    }

    /// Pair the selected product's private Signal basis with an exact installed
    /// Bridge definition. No raw component admission escapes this composition.
    #[doc(hidden)]
    pub fn admit_conditional_signal_basis(
        &self,
        bridge: &BridgeSealedRuntimeAssembly,
        lowering: &Arc<BridgeInstalledConditionalLowering>,
    ) -> Result<BridgeConditionalSignalBasisBinding, BridgeConditionalDenial> {
        bridge.admit_conditional_signal_basis(lowering, self.signal_basis())
    }
}
