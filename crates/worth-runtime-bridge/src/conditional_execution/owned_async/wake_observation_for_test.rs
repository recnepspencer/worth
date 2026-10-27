use super::super::{BridgeOwnedAsyncRequestAdmission, BridgeOwnedSignalRuntime};
use crate::facade::BridgeAsyncCompletionRejection;
use worth_signal::facade::{TemporalFrontierSnapshot, TemporalWakeSummary};

impl BridgeOwnedSignalRuntime {
    #[doc(hidden)]
    pub fn owned_async_retention_storage_for_test(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
    ) -> Result<
        (
            worth_signal::facade::branch::SignalOwnedAsyncRetentionStorageForTest,
            TemporalWakeSummary,
        ),
        BridgeAsyncCompletionRejection,
    > {
        self.require_owned_async_request(request)?;
        let (port, binding) = request.signal_parts();
        port.owned_async_retention_storage_for_test(binding).map_err(|cause| {
            BridgeAsyncCompletionRejection::new(
                crate::facade::BridgeAsyncCompletionRejectionKind::SignalCompletionAdmissionUnavailable,
                format!("owned async retention observation denied: {cause:?}"),
            )
        })
    }

    #[doc(hidden)]
    pub fn owned_async_wake_frontier_for_test(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
    ) -> Result<(TemporalWakeSummary, TemporalFrontierSnapshot), BridgeAsyncCompletionRejection>
    {
        self.require_owned_async_request(request)?;
        let (port, binding) = request.signal_parts();
        port.owned_async_wake_frontier_for_test(binding).map_err(|cause| {
            BridgeAsyncCompletionRejection::new(
                crate::facade::BridgeAsyncCompletionRejectionKind::SignalCompletionAdmissionUnavailable,
                format!("owned async wake observation denied: {cause:?}"),
            )
        })
    }
}
