use super::BridgeSealedRuntimeAssembly;
use crate::facade::{
    BridgeAsyncRequestIdentityRejection, BridgeAsyncRequestTruthViewBasis,
    BridgeOwnedAsyncManagedDenial, BridgeOwnedAsyncManagedRequestAdmission,
    LoweredBridgeAsyncSourceDeclaration,
};

impl BridgeSealedRuntimeAssembly {
    /// World supplies the genuine selected source; queue mode is installed meaning.
    #[doc(hidden)]
    pub fn admit_unqueued_owned_async_request_identity(
        &self,
        declaration: &LoweredBridgeAsyncSourceDeclaration,
        signal_basis: &worth_signal::facade::branch::AdmittedSignalBranchBasis,
        truth_basis: BridgeAsyncRequestTruthViewBasis,
    ) -> Result<BridgeOwnedAsyncManagedRequestAdmission, BridgeAsyncRequestIdentityRejection> {
        self.runtime.admit_unqueued_owned_async_request_identity(
            declaration,
            signal_basis,
            truth_basis,
        )
    }

    pub fn finish_unqueued_owned_async_entry_cleanup(
        &self,
        owner: &mut BridgeOwnedAsyncManagedRequestAdmission,
    ) -> Result<(), BridgeOwnedAsyncManagedDenial> {
        self.runtime
            .finish_unqueued_owned_async_entry_cleanup(owner)
    }
}
