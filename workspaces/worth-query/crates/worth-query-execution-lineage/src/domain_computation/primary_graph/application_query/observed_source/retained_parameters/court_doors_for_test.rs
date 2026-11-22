//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryRetainedObservedParameters {
    #[doc(hidden)]
    pub fn rows_for_test(
        &self,
    ) -> &worth_execution::ExecutionArray<Arc<WorthQueryObservedSourceMeaning>> {
        &self.rows
    }
}
