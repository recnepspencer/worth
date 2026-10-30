//! Read-only installed views of the declared root-selection alternatives.
use super::WorthQueryInstalledGraphReadContract;
use std::sync::Arc;

impl WorthQueryInstalledGraphReadContract {
    pub fn component_selection(
        &self,
    ) -> Option<&worth_query_declaration::facade::application_query::ApplicationQueryComponentMeaning>
    {
        self.meaning.component_selection.as_deref()
    }

    pub fn orbit(
        &self,
    ) -> Option<
        &Arc<worth_query_declaration::facade::application_query::ApplicationQueryOrbitMeaning>,
    > {
        self.meaning.orbit.as_ref()
    }

    pub fn managed_index_selection(
        &self,
    ) -> Option<
        &worth_query_declaration::facade::application_query::ApplicationQueryManagedIndexMeaning,
    > {
        self.meaning.managed_index.as_deref()
    }

    pub fn root_paths(&self) -> &[crate::application_query::WorthQueryInstalledRootPath] {
        &self.meaning.root_paths
    }
}
