use worth_relational::facade::runtime::RelationalRuntime;

use super::WorthQueryPrimaryGraph;

impl WorthQueryPrimaryGraph {
    /// Internal selected-read owners borrow the existing Native runtime
    /// directly. Constructing a full integration handle here would rebuild
    /// every installed index ID for a one-field currency check.
    pub(in crate::domain_computation::primary_graph) fn with_runtime<T>(
        &self,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> Result<T, crate::facade::primary_graph::WorthQueryHandleDenial> {
        self.source_owner.with_runtime(read)
    }

    pub(in crate::domain_computation::primary_graph) fn with_runtime_mut<T>(
        &self,
        mutate: impl FnOnce(&mut RelationalRuntime) -> T,
    ) -> Result<T, crate::facade::primary_graph::WorthQueryHandleDenial> {
        self.source_owner.with_runtime_mut(mutate)
    }
}
