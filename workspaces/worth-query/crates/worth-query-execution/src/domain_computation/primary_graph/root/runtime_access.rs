//! Borrowed runtime access under installed phase custody.
use super::WorthQueryPrimaryGraphIntegrationHandle;
use worth_relational::facade::runtime::RelationalRuntime;

impl WorthQueryPrimaryGraphIntegrationHandle {
    pub(in crate::domain_computation) fn relational_runtime_instance_id(&self) -> u64 {
        self.source_owner.runtime_instance_id
    }

    pub fn with_runtime<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> T {
        self.source_owner.with_runtime(execution, read)
    }

    pub(crate) fn with_runtime_mut<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> T {
        self.source_owner.with_runtime_mut(execution, mutate)
    }

    pub(crate) fn with_runtime_mut_unwind_isolated<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> T {
        self.source_owner
            .with_runtime_mut_unwind_isolated(execution, mutate)
    }

    pub(in crate::domain_computation) fn with_query_runtime_mut<T>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(&mut RelationalRuntime, &super::WorthQueryPrimaryGraphLayout) -> T,
    ) -> T {
        self.source_owner
            .with_runtime_mut(execution, |runtime| read(runtime, &self.layout))
    }
}
