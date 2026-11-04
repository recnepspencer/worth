//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryGraphProviderCall {
    #[doc(hidden)]
    pub fn streamed_for_test(
        &self,
        provider_receipt: impl Into<Arc<str>>,
        stream: WorthQueryExecutionGraphReadStreamEvidence,
        work_report: WorthQueryProviderWorkReport,
    ) -> Result<WorthQueryGraphProviderReceipt, WorthQueryGraphProviderFailure> {
        Self::streamed(self, provider_receipt, stream, work_report)
    }
}

impl WorthQueryGraphProviderCallRequest {
    #[doc(hidden)]
    pub fn with_managed_execution_snapshot_for_test(
        self,
        snapshot_identity: impl Into<Arc<str>>,
    ) -> Self {
        Self::with_managed_execution_snapshot(self, snapshot_identity)
    }
}
