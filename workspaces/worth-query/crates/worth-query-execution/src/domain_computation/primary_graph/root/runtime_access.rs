use super::WorthQueryPrimaryGraphIntegrationHandle;
#[cfg(any(test, feature = "test-primary-graph-faults"))]
use super::WorthQueryPrimaryGraphLayout;
use worth_relational::facade::runtime::RelationalRuntime;

impl WorthQueryPrimaryGraphIntegrationHandle {
    #[doc(hidden)]
    pub fn with_runtime<T>(
        &self,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> Result<T, crate::facade::primary_graph::WorthQueryHandleDenial> {
        self.source_owner.with_runtime(read)
    }

    pub(crate) fn with_runtime_mut<T>(
        &self,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> Result<T, crate::facade::primary_graph::WorthQueryHandleDenial> {
        self.source_owner.with_runtime_mut(mutate)
    }

    pub(crate) fn with_runtime_mut_unwind_isolated<T>(
        &self,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> Result<T, crate::facade::primary_graph::WorthQueryHandleDenial> {
        self.source_owner.with_runtime_mut_unwind_isolated(mutate)
    }

    #[cfg(any(test, feature = "test-primary-graph-faults"))]
    pub(in crate::domain_computation) fn with_query_runtime_mut<T>(
        &self,
        read: impl FnOnce(&mut RelationalRuntime, &WorthQueryPrimaryGraphLayout) -> T,
    ) -> Result<T, crate::facade::primary_graph::WorthQueryHandleDenial> {
        self.source_owner
            .with_runtime_mut(|runtime| read(runtime, &self.layout))
    }
}
