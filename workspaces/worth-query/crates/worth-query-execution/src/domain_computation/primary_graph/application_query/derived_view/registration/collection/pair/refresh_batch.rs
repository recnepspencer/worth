//! Entry refresh with both independent reads charged to one caller-owned loan.

use crate::domain_computation::primary_graph::application_query::BatchScopedOps;
use crate::domain_computation::primary_graph::application_query::derived_view::registration::RegistrationOwnOps;
use crate::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::read::ReadParentOps;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationProjection,
    WorthQueryManagedDerivedValue, WorthQueryManagedDerivedView, WorthQueryManagedDerivedViewKey,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationBatchResult, WorthQueryApplicationQueryBatchAdmission,
    WorthQueryManagedDerivedCollectionBatchRefreshDenial as Denial,
};

pub trait RefreshBatchOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    fn refresh_managed_derived_collection_pair_entry_in_batch<
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
            Denial,
        >,
        first_link: impl FnOnce(&FirstResult) -> LinkKey,
        second_link: impl FnOnce(&SecondResult) -> LinkKey,
        project: impl FnOnce(&FirstResult, &SecondResult) -> Value,
    ) -> Result<std::sync::Arc<Value>, Denial>
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
        view.state
            .requires_entry_refresh(key, product.selected_commit())?;
        self.admit_view_product(view, product)?;
        let first_identity = view
            .state
            .entry_query
            .as_ref()
            .ok_or(super::Denial::ForeignQuery)?;
        if first.query_identity() != first_identity || first.scope.entity_id() != key.root() {
            return Err(super::Denial::ForeignQuery.into());
        }
        let first_result = self
            .execute_application_query_one_shot_in_batch_with_request(execution, first, batch)
            .map_err(Denial::Read)?;
        self.refresh_managed_derived_collection_pair_entry_from_batch_result(
            view,
            product,
            key,
            first_result,
            batch,
            second_for,
            first_link,
            second_link,
            project,
        )
    }

    fn refresh_managed_derived_collection_pair_entry_from_batch_result<
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
            Denial,
        >,
        first_link: impl FnOnce(&FirstResult) -> LinkKey,
        second_link: impl FnOnce(&SecondResult) -> LinkKey,
        project: impl FnOnce(&FirstResult, &SecondResult) -> Value,
    ) -> Result<std::sync::Arc<Value>, Denial>
    where
        FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery>,
        SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery>,
        Value: WorthQueryManagedDerivedValue,
        LinkKey: Eq,
    {
        view.state
            .requires_entry_refresh(key, product.selected_commit())?;
        self.admit_view_product(view, product)?;
        let first_result = first.into_result_for(batch).ok_or(Denial::ForeignBatch)?;
        let mut second_denial = None;
        let read = self.read_managed_entry_pair_from_result(
            view,
            product,
            key.root(),
            first_result,
            |row| match second_for(row, batch) {
                Ok(result) => match result.into_result_for(batch) {
                    Some(result) => Ok(result),
                    None => {
                        second_denial = Some(Denial::ForeignBatch);
                        Err(super::Denial::QueryExecutionDenied)
                    }
                },
                Err(denial) => {
                    second_denial = Some(denial);
                    Err(super::Denial::QueryExecutionDenied)
                }
            },
            first_link,
            second_link,
            project,
        );
        if let Some(denial) = second_denial {
            return Err(denial);
        }
        let (issued_key, value, dependencies) = read?;
        if &issued_key != key {
            return Err(super::Denial::IncompleteDependencies.into());
        }
        view.state
            .refresh_entry(key, value, dependencies, product.selected_commit())
            .map_err(Denial::View)
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > RefreshBatchOps<Schema> for HostRuntime
{
}
