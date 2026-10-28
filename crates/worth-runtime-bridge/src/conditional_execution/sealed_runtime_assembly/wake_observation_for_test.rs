use super::BridgeSealedRuntimeAssembly;
use crate::facade::{BridgeAsyncCompletionRejection, BridgeOwnedAsyncRequestAdmission};
use worth_signal::facade::{TemporalFrontierSnapshot, TemporalWakeSummary};

impl BridgeSealedRuntimeAssembly {
    #[doc(hidden)]
    pub fn owned_async_retention_storage_for_test(
        &self,
        request: &crate::facade::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<
        (
            worth_signal::facade::branch::SignalOwnedAsyncRetentionStorageForTest,
            worth_signal::facade::TemporalWakeSummary,
        ),
        crate::facade::BridgeAsyncCompletionRejection,
    > {
        self.runtime.owned_async_retention_storage_for_test(request)
    }

    #[doc(hidden)]
    pub fn owned_async_wake_frontier_for_test(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
    ) -> Result<(TemporalWakeSummary, TemporalFrontierSnapshot), BridgeAsyncCompletionRejection>
    {
        self.runtime.owned_async_wake_frontier_for_test(request)
    }
}
