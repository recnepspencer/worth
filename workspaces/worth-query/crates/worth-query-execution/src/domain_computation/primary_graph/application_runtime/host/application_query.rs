//! The runtime composes `application_query`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost;

impl<Schema: ApplicationSchema> ApplicationQueryHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn application_query_host(&self) -> &dyn ApplicationQueryHost<Schema> {
        self
    }
    fn managed_indexes(&self) -> &crate::domain_computation::primary_graph::application_query::managed_index::ManagedIndexProviderCatalog<Schema>{
        &self.managed_indexes
    }
    fn relational_branch_identity(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchIdentity {
        &self.runtime.relational_branch_identity
    }
    fn source_meanings(&self) -> &crate::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry{
        self.runtime.source_meanings()
    }
}
