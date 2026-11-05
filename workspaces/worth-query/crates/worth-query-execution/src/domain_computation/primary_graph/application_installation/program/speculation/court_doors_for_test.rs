//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationPreviewSession {
    #[doc(hidden)]
    pub fn _source_for_test(
        &self,
    ) -> &std::sync::Arc<worth_runtime_bridge::facade::RelationalBridgeObservationLease> {
        &self._source
    }
}

impl WorthQueryApplicationPreviewSession {
    #[doc(hidden)]
    pub fn handle_for_test(&self) -> &BridgeSpeculativeSessionHandle {
        &self.handle
    }
}

impl WorthQueryReadmittedApplicationPreview {
    #[doc(hidden)]
    pub fn session_for_test(&self) -> &WorthQueryApplicationPreviewSession {
        &self.session
    }
}

impl WorthQueryReadmittedApplicationPreview {
    #[doc(hidden)]
    pub fn session_into_for_test(self) -> WorthQueryApplicationPreviewSession {
        self.session
    }
}
