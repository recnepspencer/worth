use std::collections::BTreeMap;

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use super::{
    Denial, WorthQueryApplicationProjection, WorthQueryManagedDerivedValue,
    WorthQueryManagedDerivedView, WorthQueryManagedDerivedViewKey,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::retention::{
    RetainedMemberToken, WorthQueryManagedDerivedMemberToken,
};
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Admit and execute each first query only when its certified membership
    /// token is visited. No collection of retained query plans is required.
    /// Both entry reads must return Query-issued one-shot results; all entries
    /// replace retained state together only after the exact member set passes.
    pub fn reconstruct_managed_derived_collection_pair_lazy<
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
    ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
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
        let mut membership = self.checked_view_dependencies(
            view,
            product,
            membership_result.result_set_observation().source(),
        )?;
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
        let mut tokens = BTreeMap::new();
        for (root, member) in &expected {
            tokens.insert(*root, RetainedMemberToken::new(member.clone()));
        }
        let mut entries = Vec::with_capacity(expected.len());
        let mut keys = Vec::with_capacity(expected.len());
        for (root, member) in expected {
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
            keys.push(key.clone());
            entries.push((key, value, dependencies));
        }
        self.admit_view_product(view, product)?;
        view.state.reconstruct(
            membership_result
                .result_set_observation()
                .source()
                .managed_derived_view_key(),
            Some(tokens),
            entries,
            membership,
            product.selected_commit(),
        )?;
        Ok(keys)
    }
}
