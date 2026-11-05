//! The runtime's consumer methods over `derived_view_registration` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedValue;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedView;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedViewSnapshot;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewDenial;
use worth_query_declaration::facade::application_query::ApplicationDerivedViewDefinition;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn open_managed_derived_view<Query, Parameters, QueryResult, Scope, Value>(
        &self,
        definition: &ApplicationDerivedViewDefinition<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        product: &WorthQueryProductBranchLease,
    ) -> Result<WorthQueryManagedDerivedView<Query, Value>, WorthQueryManagedDerivedViewDenial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::RegistrationOps<
            Schema,
        >>::open_managed_derived_view(self, definition, query, product)
    }

    pub fn observe_managed_derived_view<Query, Value>(
        &self,
        view: &WorthQueryManagedDerivedView<Query, Value>,
    ) -> Result<WorthQueryManagedDerivedViewSnapshot<Value>, WorthQueryManagedDerivedViewDenial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::RegistrationOps<
            Schema,
        >>::observe_managed_derived_view(self, view)
    }

    pub fn reconstruct_managed_derived_view<Query, QueryResult, Value>(
        &self,
        view: &WorthQueryManagedDerivedView<Query, Value>,
        product: &WorthQueryProductBranchLease,
        result: &WorthQueryApplicationOneShotResult<Query, QueryResult>,
        mut project: impl FnMut(&QueryResult) -> Value,
    ) -> Result<(), WorthQueryManagedDerivedViewDenial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::RegistrationOps<
            Schema,
        >>::reconstruct_managed_derived_view(self, view, product, result, project)
    }
}
