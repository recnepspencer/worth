use super::*;
use crate::facade::{
    BridgeOwnedAsyncCommittedCompletion, BridgeOwnedAsyncCompletionAdmission,
    BridgeOwnedAsyncManagedDenial, BridgeOwnedAsyncManagedRequestAdmission,
    BridgeOwnedAsyncReleasedResource, BridgeOwnedAsyncResourceReleaseRefusal,
};

impl BridgeSealedRuntimeAssembly {
    pub fn abort_process_local_owned_async_retry(
        &self,
        owner: &mut BridgeOwnedAsyncManagedRequestAdmission,
    ) -> Result<(), BridgeOwnedAsyncManagedDenial> {
        self.runtime.abort_process_local_owned_async_retry(owner)
    }

    pub fn release_process_local_managed_request(
        &self,
        owner: BridgeOwnedAsyncManagedRequestAdmission,
        completion: Option<BridgeOwnedAsyncCommittedCompletion>,
    ) -> Result<BridgeOwnedAsyncReleasedResource, BridgeOwnedAsyncResourceReleaseRefusal> {
        self.runtime
            .release_process_local_managed_request(owner, completion)
    }

    pub fn release_process_local_completion_denial(
        &self,
        owner: &BridgeOwnedAsyncManagedRequestAdmission,
        completion: &BridgeOwnedAsyncCompletionAdmission,
    ) -> Result<(), BridgeOwnedAsyncManagedDenial> {
        self.runtime
            .release_process_local_completion_denial(owner, completion)
    }
}
