use crate::facade::{BridgeAsyncCompletionRejection, BridgeAsyncCompletionRejectionKind};

use super::super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeOwnedSignalRuntime,
};

impl BridgeOwnedSignalRuntime {
    pub fn retire_owned_async_request(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<bool, BridgeAsyncCompletionRejection> {
        self.require_owned_async_request(request)?;
        let (port, binding) = request.signal_parts();
        let report = port
            .cancel_owned_async_request(binding, request.request().request_handle())
            .map_err(|denial| {
                BridgeAsyncCompletionRejection::new(
                    BridgeAsyncCompletionRejectionKind::SignalCompletionAdmissionUnavailable,
                    format!("owned async request cancellation denied: {denial:?}"),
                )
            })?;
        Ok(report.cancelled_request().is_some())
    }

    pub fn owned_signal_active_node_count(&self) -> Result<usize, BridgeConditionalDenial> {
        self.signal_services()?
            .conditional_extension_port()
            .active_node_count()
            .map_err(|denial| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::SignalExecution,
                    format!("Signal topology inspection failed: {denial:?}"),
                )
            })
    }

    pub fn owned_async_active_request_count(
        &self,
        request: &super::super::BridgeOwnedAsyncRequestAdmission,
    ) -> Result<usize, BridgeAsyncCompletionRejection> {
        self.require_owned_async_request(request)?;
        let (port, binding) = request.signal_parts();
        port.active_owned_async_request_count(binding)
            .map_err(|denial| {
                BridgeAsyncCompletionRejection::new(
                    BridgeAsyncCompletionRejectionKind::SignalCompletionAdmissionUnavailable,
                    format!("owned async cleanup inspection denied: {denial:?}"),
                )
            })
    }
}
