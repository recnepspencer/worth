use std::collections::BTreeSet;

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use super::{
    Denial, WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationProjection,
    WorthQueryManagedDerivedValue, WorthQueryManagedDerivedView, WorthQueryManagedDerivedViewKey,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::dependency::ViewDependency;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn read_managed_entry_pair<
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
        view: &WorthQueryManagedDerivedView<MembershipQuery, Value>,
        product: &WorthQueryProductBranchLease,
        expected_root: EntityId,
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
            Denial,
        >,
        first_link: impl FnOnce(&FirstResult) -> LinkKey,
        second_link: impl FnOnce(&SecondResult) -> LinkKey,
        project: impl FnOnce(&FirstResult, &SecondResult) -> Value,
    ) -> Result<
        (
            WorthQueryManagedDerivedViewKey,
            Value,
            BTreeSet<ViewDependency>,
        ),
        Denial,
    >
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
        self.admit_view_product(view, product)?;
        let first_identity = view
            .state
            .entry_query
            .as_ref()
            .ok_or(Denial::ForeignQuery)?;
        if first.query_identity() != first_identity || first.scope.entity_id() != expected_root {
            return Err(Denial::ForeignQuery);
        }
        let first_result = self
            .execute_application_query_one_shot(first)
            .map_err(|_| Denial::QueryExecutionDenied)?;
        self.read_managed_entry_pair_from_result(
            view,
            product,
            expected_root,
            first_result,
            second_for,
            first_link,
            second_link,
            project,
        )
    }

    pub(super) fn read_managed_entry_pair_from_result<
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
        expected_root: EntityId,
        first_result: WorthQueryApplicationOneShotResult<FirstQuery, FirstResult>,
        second_for: impl FnOnce(
            &FirstResult,
        ) -> Result<
            WorthQueryApplicationOneShotResult<SecondQuery, SecondResult>,
            Denial,
        >,
        first_link: impl FnOnce(&FirstResult) -> LinkKey,
        second_link: impl FnOnce(&SecondResult) -> LinkKey,
        project: impl FnOnce(&FirstResult, &SecondResult) -> Value,
    ) -> Result<
        (
            WorthQueryManagedDerivedViewKey,
            Value,
            BTreeSet<ViewDependency>,
        ),
        Denial,
    >
    where
        FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery>,
        SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery>,
        Value: WorthQueryManagedDerivedValue,
        LinkKey: Eq,
    {
        self.admit_view_product(view, product)?;
        let first_identity = view
            .state
            .entry_query
            .as_ref()
            .ok_or(Denial::ForeignQuery)?;
        let second_identity = view
            .state
            .secondary_entry_query
            .as_ref()
            .ok_or(Denial::ForeignQuery)?;
        let ([first_row], [first_source]) = (first_result.rows(), first_result.observed_sources())
        else {
            return Err(Denial::IncompleteDependencies);
        };
        let first_key = first_link(first_row);
        if first_source.source_root() != expected_root {
            return Err(Denial::IncompleteDependencies);
        }
        let key = first_source.managed_derived_view_key();
        let second_result = second_for(first_row)?;
        self.admit_view_product(view, product)?;
        let ([second_row], [second_source]) =
            (second_result.rows(), second_result.observed_sources())
        else {
            return Err(Denial::IncompleteDependencies);
        };
        if first_key != second_link(second_row) {
            return Err(Denial::IncompleteDependencies);
        }
        let mut dependencies =
            self.checked_view_dependencies_for_query(view, product, first_source, first_identity)?;
        dependencies.extend(self.checked_view_dependencies_for_query(
            view,
            product,
            first_result.result_set_observation().source(),
            first_identity,
        )?);
        dependencies.extend(self.checked_view_dependencies_for_query(
            view,
            product,
            second_source,
            second_identity,
        )?);
        dependencies.extend(self.checked_view_dependencies_for_query(
            view,
            product,
            second_result.result_set_observation().source(),
            second_identity,
        )?);
        Ok((key, project(first_row, second_row), dependencies))
    }
}
