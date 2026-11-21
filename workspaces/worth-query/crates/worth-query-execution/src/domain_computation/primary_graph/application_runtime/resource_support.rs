//! resource support.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(in crate::domain_computation) fn graph_work_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        self.inner.runtime.primary_provider.application_resource_support()
    }

    pub(in crate::domain_computation) fn graph_work_resource_support_ref(
        &self,
    ) -> &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot
    {
        self.inner.runtime.primary_provider.application_resource_support_ref()
    }

    pub(in crate::domain_computation) fn graph_work_provider_identity(&self) -> &str {
        self.inner.graph_work_provider_identity()
    }
}
