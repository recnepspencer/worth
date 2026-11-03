//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn with_runtime_mut_for_test<T>(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>, mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> T {
        Self::with_runtime_mut(self, execution, mutate)
    }
}
