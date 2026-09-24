use std::collections::BTreeSet;

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use super::{
    Denial, WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationProjection,
    WorthQueryManagedDerivedValue, WorthQueryManagedDerivedView, WorthQueryManagedDerivedViewKey,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Cold reconstruction runs both declared entry reads for every member;
    /// the second read is admitted from the first observed row, without a
    /// caller-maintained occurrence-to-secondary-source map. The membership
    /// result alone never certifies an entry's projected data.
    pub fn reconstruct_managed_derived_collection_pair<
        'a,
        MembershipQuery,
        MembershipResult,
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
        membership_result: &WorthQueryApplicationOneShotResult<MembershipQuery, MembershipResult>,
        first_plans: impl IntoIterator<
            Item = WorthQueryAdmittedApplicationQueryPlan<
                'a,
                Schema,
                FirstQuery,
                FirstParameters,
                FirstResult,
                FirstPrincipal,
                FirstPrincipalIdentity,
                FirstScope,
            >,
        >,
        mut second_for: impl FnMut(
            &FirstResult,
        ) -> Result<
            WorthQueryApplicationOneShotResult<SecondQuery, SecondResult>,
            Denial,
        >,
        member_roots: impl Fn(&MembershipResult) -> Vec<EntityId>,
        first_link: impl Fn(&FirstResult) -> LinkKey,
        second_link: impl Fn(&SecondResult) -> LinkKey,
        mut project: impl FnMut(&FirstResult, &SecondResult) -> Value,
    ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
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
        if membership_result.rows().len() != membership_result.observed_sources().len() {
            return Err(Denial::IncompleteDependencies);
        }
        let mut membership = self.checked_view_dependencies(
            view,
            product,
            membership_result.result_set_observation().source(),
        )?;
        let mut expected = BTreeSet::new();
        for (row, source) in membership_result
            .rows()
            .iter()
            .zip(membership_result.observed_sources())
        {
            membership.extend(self.checked_view_dependencies(view, product, source)?);
            for root in member_roots(row) {
                if !expected.insert(root) {
                    return Err(Denial::IncompleteDependencies);
                }
                if expected.len() > view.state.limits.maximum_entries() {
                    return Err(Denial::EntryCapacityExceeded);
                }
            }
        }
        let mut entries = Vec::with_capacity(expected.len());
        let mut keys = Vec::with_capacity(expected.len());
        for first in first_plans {
            let root = first.scope.entity_id();
            if !expected.remove(&root) {
                return Err(Denial::IncompleteDependencies);
            }
            let (key, value, dependencies) = self.read_managed_entry_pair(
                view,
                product,
                root,
                first,
                |row| second_for(row),
                |row| first_link(row),
                |row| second_link(row),
                |first, second| project(first, second),
            )?;
            keys.push(key.clone());
            entries.push((key, value, dependencies));
        }
        if !expected.is_empty() {
            return Err(Denial::IncompleteDependencies);
        }
        self.admit_view_product(view, product)?;
        view.state
            .reconstruct(entries, membership, product.selected_commit())?;
        Ok(keys)
    }
}
