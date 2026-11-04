//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraph {
    #[doc(hidden)]
    pub fn with_runtime_for_test<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> T {
        Self::with_runtime(self, execution, read)
    }
}
