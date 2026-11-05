//! The runtime's consumer methods over `derived_view_registration_collection_pair_refresh_batch` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedValue;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedView;
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBatchResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchAdmission;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedCollectionBatchRefreshDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewKey;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn refresh_managed_derived_collection_pair_entry_in_batch<
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
        batch: &WorthQueryApplicationQueryBatchAdmission,
        second_for: impl FnOnce(
            &FirstResult,
            &WorthQueryApplicationQueryBatchAdmission,
        ) -> Result<
            WorthQueryApplicationBatchResult<SecondQuery, SecondResult>,
            WorthQueryManagedDerivedCollectionBatchRefreshDenial,
        >,
        first_link: impl FnOnce(&FirstResult) -> LinkKey,
        second_link: impl FnOnce(&SecondResult) -> LinkKey,
        project: impl FnOnce(&FirstResult, &SecondResult) -> Value,
    ) -> Result<std::sync::Arc<Value>, WorthQueryManagedDerivedCollectionBatchRefreshDenial>
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
        <Self as crate::domain_computation::primary_graph::application_query::RefreshBatchOps<
            Schema,
        >>::refresh_managed_derived_collection_pair_entry_in_batch(
            self,
            execution,
            view,
            product,
            key,
            first,
            batch,
            second_for,
            first_link,
            second_link,
            project,
        )
    }

    pub fn refresh_managed_derived_collection_pair_entry_from_batch_result<
        MembershipQuery,
        FirstQuery,
        FirstResult,
        SecondQuery,
        SecondResult,
        Value,
        LinkKey,
    >(
        &self,
        view: &WorthQueryManagedDerivedView<MembershipQuery, Value>,
        product: &WorthQueryProductBranchLease,
        key: &WorthQueryManagedDerivedViewKey,
        first: WorthQueryApplicationBatchResult<FirstQuery, FirstResult>,
        batch: &WorthQueryApplicationQueryBatchAdmission,
        second_for: impl FnOnce(
            &FirstResult,
            &WorthQueryApplicationQueryBatchAdmission,
        ) -> Result<
            WorthQueryApplicationBatchResult<SecondQuery, SecondResult>,
            WorthQueryManagedDerivedCollectionBatchRefreshDenial,
        >,
        first_link: impl FnOnce(&FirstResult) -> LinkKey,
        second_link: impl FnOnce(&SecondResult) -> LinkKey,
        project: impl FnOnce(&FirstResult, &SecondResult) -> Value,
    ) -> Result<std::sync::Arc<Value>, WorthQueryManagedDerivedCollectionBatchRefreshDenial>
    where
        FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery>,
        SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery>,
        Value: WorthQueryManagedDerivedValue,
        LinkKey: Eq,
    {
        <Self as crate::domain_computation::primary_graph::application_query::RefreshBatchOps<
            Schema,
        >>::refresh_managed_derived_collection_pair_entry_from_batch_result(
            self,
            view,
            product,
            key,
            first,
            batch,
            second_for,
            first_link,
            second_link,
            project,
        )
    }
}
