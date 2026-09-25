//! One certified membership read reconciles retained pair entries.

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use super::{
    Denial, WorthQueryApplicationProjection, WorthQueryManagedDerivedValue,
    WorthQueryManagedDerivedView, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::retention::WorthQueryManagedDerivedMemberToken;
use crate::domain_computation::primary_graph::application_query::{
    derived_view::WorthQueryManagedDerivedViewReconciliation, WorthQueryApplicationOneShotResult,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Certify current membership, then read only added or dirty pair entries.
    /// Publication has already tracked every intervening dependency change;
    /// any unknown transition makes this route cold instead of reusing values.
    pub fn reconcile_managed_derived_collection_pair_lazy<
        MembershipQuery,
        MembershipResult,
        Member,
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
        membership_result: &WorthQueryApplicationOneShotResult<MembershipQuery, MembershipResult>,
        members: impl Fn(&MembershipResult) -> Vec<(EntityId, Member)>,
        mut first_for: impl FnMut(
            &Member,
        ) -> Result<
            WorthQueryApplicationOneShotResult<FirstQuery, FirstResult>,
            Denial,
        >,
        mut second_for: impl FnMut(
            &FirstResult,
        ) -> Result<
            WorthQueryApplicationOneShotResult<SecondQuery, SecondResult>,
            Denial,
        >,
        first_link: impl Fn(&FirstResult) -> LinkKey,
        second_link: impl Fn(&SecondResult) -> LinkKey,
        mut project: impl FnMut(&FirstResult, &SecondResult) -> Value,
    ) -> Result<WorthQueryManagedDerivedViewReconciliation, Denial>
    where
        FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery>,
        SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery>,
        Value: WorthQueryManagedDerivedValue,
        Member: WorthQueryManagedDerivedMemberToken,
        LinkKey: Eq,
    {
        self.admit_view_product(view, product)?;
        if membership_result.rows().len() != membership_result.observed_sources().len() {
            return Err(Denial::IncompleteDependencies);
        }
        let source = membership_result.result_set_observation().source();
        let mut membership = self.checked_view_dependencies(view, product, source)?;
        let mut expected = BTreeMap::new();
        for (row, source) in membership_result
            .rows()
            .iter()
            .zip(membership_result.observed_sources())
        {
            membership.extend(self.checked_view_dependencies(view, product, source)?);
            for (root, member) in members(row) {
                if expected.insert(root, member).is_some() {
                    return Err(Denial::IncompleteDependencies);
                }
                if expected.len() > view.state.limits.maximum_entries() {
                    return Err(Denial::EntryCapacityExceeded);
                }
            }
        }
        let plan = view.state.plan_membership_reconciliation(
            &source.managed_derived_view_key(),
            &expected,
            product.selected_commit(),
        )?;
        let refreshed_entries = plan.read_count();
        let retained_entries = plan.retained_count();
        let mut entries = Vec::with_capacity(refreshed_entries);
        for (root, member) in expected {
            if !plan.needs_read(root) {
                continue;
            }
            let first_result = first_for(&member)?;
            let (key, value, dependencies) = self.read_managed_entry_pair_from_result(
                view,
                product,
                root,
                first_result,
                |row| second_for(row),
                |row| first_link(row),
                |row| second_link(row),
                |first, second| project(first, second),
            )?;
            entries.push((key, value, dependencies));
        }
        self.admit_view_product(view, product)?;
        let (keys, removed_entries) = view.state.reconcile_membership(
            plan,
            membership,
            entries,
            product.selected_commit(),
        )?;
        Ok(WorthQueryManagedDerivedViewReconciliation {
            keys,
            refreshed_entries,
            retained_entries,
            removed_entries,
        })
    }
}
