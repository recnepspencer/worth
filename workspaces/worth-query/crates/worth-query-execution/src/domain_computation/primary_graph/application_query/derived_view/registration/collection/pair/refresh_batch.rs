//! Entry refresh with both independent reads charged to one caller-owned loan.

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

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Refreshes one dirty entry under a shared read-work and result/source
    /// custody allowance. The second callback executes its independently
    /// admitted plan in the supplied batch and returns its genuine result and
    /// live row claim. Both claims remain held through validation and projection.
    ///
    /// All ordinary public-query, root, link, source, branch and current-product
    /// checks are unchanged. Authorization/preparation retain their ordinary
    /// per-item protections. Failure leaves the entry dirty and unavailable.
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
            .execute_application_query_one_shot_in_batch(first, batch)
            .map_err(Denial::Read)?;
        let (first_result, _first_claim) = first_result.into_parts();
        let mut second_claim = None;
        let mut second_denial = None;
        let read = self.read_managed_entry_pair_from_result(
            view,
            product,
            key.root(),
            first_result,
            |row| match second_for(row, batch) {
                Ok(result) => match result.into_parts_for(batch) {
                    Some((result, claim)) => {
                        second_claim = Some(claim);
                        Ok(result)
                    }
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
