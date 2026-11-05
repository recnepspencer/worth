//! The runtime composes `application_entry`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_entry::host::ApplicationEntryHost;

impl<Schema: ApplicationSchema> ApplicationEntryHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn application_entry_host(&self) -> &dyn ApplicationEntryHost<Schema> {
        self
    }
    fn installed_operations(
        &self,
    ) -> &std::sync::Arc<worth_query_installation::facade::InstalledOperationCatalog> {
        &self.runtime.installed_operations
    }
}
