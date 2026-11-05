//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryDecisionReadSetFailure {
    #[doc(hidden)]
    pub fn detail_for_test(&self) -> &Arc<str> {
        &self.detail
    }
}

impl WorthQueryDecisionReadSetFailure {
    #[doc(hidden)]
    pub fn with_native_query_read_stop_for_test(
        self,
        stop: worth_relational::facade::runtime::QueryReadExecutionStop,
    ) -> Self {
        Self::with_native_query_read_stop(self, stop)
    }
}
