//! The runtime's consumer methods over `derived_view_registration_collection_pair` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedValue;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedView;
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewKey;
use worth_query_declaration::facade::application_query::ApplicationDerivedViewDefinition;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn open_managed_derived_collection_pair<
        MembershipQuery,
        MembershipParameters,
        MembershipResult,
        MembershipScope,
        FirstQuery,
        FirstParameters,
        FirstResult,
        FirstScope,
        SecondQuery,
        SecondParameters,
        SecondResult,
        SecondScope,
        Value,
    >(
        &self,
        definition: &ApplicationDerivedViewDefinition<
            Schema,
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
        >,
        membership_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
        >,
        first_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            FirstQuery,
            FirstParameters,
            FirstResult,
            FirstScope,
        >,
        second_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            SecondQuery,
            SecondParameters,
            SecondResult,
            SecondScope,
        >,
        product: &WorthQueryProductBranchLease,
    ) -> Result<
        WorthQueryManagedDerivedView<MembershipQuery, Value>,
        WorthQueryManagedDerivedViewDenial,
    >
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::PairOps<Schema>>::open_managed_derived_collection_pair(self, definition, membership_query, first_query, second_query, product)
    }

    pub fn open_owner_sized_managed_derived_collection_pair<
        MembershipQuery,
        MembershipParameters,
        MembershipResult,
        MembershipScope,
        FirstQuery,
        FirstParameters,
        FirstResult,
        FirstScope,
        SecondQuery,
        SecondParameters,
        SecondResult,
        SecondScope,
        Value,
    >(
        &self,
        name: &'static str,
        membership_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
        >,
        first_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            FirstQuery,
            FirstParameters,
            FirstResult,
            FirstScope,
        >,
        second_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            SecondQuery,
            SecondParameters,
            SecondResult,
            SecondScope,
        >,
        product: &WorthQueryProductBranchLease,
    ) -> Result<
        WorthQueryManagedDerivedView<MembershipQuery, Value>,
        WorthQueryManagedDerivedViewDenial,
    >
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::PairOps<Schema>>::open_owner_sized_managed_derived_collection_pair(self, name, membership_query, first_query, second_query, product)
    }

    pub fn refresh_managed_derived_collection_pair_entry<
        'a,
        MembershipQuery,
        FirstQuery,
        FirstParameters,
        FirstResult,
        FirstPrincipal,
        FirstPrincipalIdentity,
        FirstScope,
        SecondQuery,
        SecondResult,
        Value,
        LinkKey,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        view: &WorthQueryManagedDerivedView<MembershipQuery, Value>,
        product: &WorthQueryProductBranchLease,
        key: &WorthQueryManagedDerivedViewKey,
        first: WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            FirstQuery,
            FirstParameters,
            FirstResult,
            FirstPrincipal,
            FirstPrincipalIdentity,
            FirstScope,
        >,
        second_for: impl FnOnce(
            &FirstResult,
        ) -> Result<
            WorthQueryApplicationOneShotResult<SecondQuery, SecondResult>,
            WorthQueryManagedDerivedViewDenial,
        >,
        first_link: impl FnOnce(&FirstResult) -> LinkKey,
        second_link: impl FnOnce(&SecondResult) -> LinkKey,
        project: impl FnOnce(&FirstResult, &SecondResult) -> Value,
    ) -> Result<std::sync::Arc<Value>, WorthQueryManagedDerivedViewDenial>
    where
        FirstQuery: 'a,
        FirstParameters: 'a,
        FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery> + 'a,
        FirstPrincipal: 'a,
        FirstPrincipalIdentity: 'a,
        FirstScope: 'a,
        SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery> + 'a,
        Value: WorthQueryManagedDerivedValue,
        LinkKey: Eq,
    {
        <Self as crate::domain_computation::primary_graph::application_query::PairOps<Schema>>::refresh_managed_derived_collection_pair_entry(self, execution, view, product, key, first, second_for, first_link, second_link, project)
    }
}
