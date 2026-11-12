//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn with_runtime_for_test<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutate: impl FnOnce(&RelationalRuntime) -> T,
    ) -> T {
        Self::with_runtime(self, execution, mutate)
    }
}

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn with_query_runtime_for_test<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(&RelationalRuntime, &super::super::WorthQueryPrimaryGraphLayout) -> T,
    ) -> T {
        Self::with_query_runtime(self, execution, read)
    }
}
