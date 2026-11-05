//! The runtime's consumer methods over `derived_view_registration_collection` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedValue;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedView;
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewDenial;
use worth_query_declaration::facade::application_query::ApplicationDerivedViewDefinition;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn open_managed_derived_collection<
        MembershipQuery,
        MembershipParameters,
        MembershipResult,
        MembershipScope,
        EntryQuery,
        EntryParameters,
        EntryResult,
        EntryScope,
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
        entry_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            EntryQuery,
            EntryParameters,
            EntryResult,
            EntryScope,
        >,
        product: &WorthQueryProductBranchLease,
    ) -> Result<
        WorthQueryManagedDerivedView<MembershipQuery, Value>,
        WorthQueryManagedDerivedViewDenial,
    >
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::CollectionOps<
            Schema,
        >>::open_managed_derived_collection(
            self,
            definition,
            membership_query,
            entry_query,
            product,
        )
    }

    pub fn refresh_managed_derived_collection_entry<
        'a,
        MembershipQuery,
        EntryQuery,
        EntryParameters,
        EntryResult,
        Principal,
        PrincipalIdentity,
        EntryScope,
        Value,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        view: &WorthQueryManagedDerivedView<MembershipQuery, Value>,
        product: &WorthQueryProductBranchLease,
        key: &crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedViewKey,
        plan: WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            EntryQuery,
            EntryParameters,
            EntryResult,
            Principal,
            PrincipalIdentity,
            EntryScope,
        >,
        project: impl FnOnce(&EntryResult) -> Value,
    ) -> Result<std::sync::Arc<Value>, WorthQueryManagedDerivedViewDenial>
    where
        EntryResult: WorthQueryApplicationProjection<Schema, EntryQuery>,
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::CollectionOps<
            Schema,
        >>::refresh_managed_derived_collection_entry(
            self, execution, view, product, key, plan, project,
        )
    }
}
