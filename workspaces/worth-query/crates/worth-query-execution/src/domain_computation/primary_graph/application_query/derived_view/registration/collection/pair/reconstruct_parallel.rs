//! Bounded concurrent pair reads with serial projection and atomic retention.

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

const MAX_PAIR_READS_IN_FLIGHT: usize = 8;
mod worker_pool;
use worker_pool::read_bounded_pairs;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Query runs at most eight first/second read pairs concurrently. Each
    /// worker returns Query-issued observations; their identities, declared
    /// links and tracked dependencies are checked before serial projection.
    /// A denial or worker failure leaves the retained collection unchanged.
    pub fn reconstruct_managed_derived_collection_pair_parallel<
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
        read_pair: impl Fn(
                &Member,
            ) -> Result<
                (
                    WorthQueryApplicationOneShotResult<FirstQuery, FirstResult>,
                    WorthQueryApplicationOneShotResult<SecondQuery, SecondResult>,
                ),
                Denial,
            > + Sync,
        first_link: impl Fn(&FirstResult) -> LinkKey,
        second_link: impl Fn(&SecondResult) -> LinkKey,
        mut project: impl FnMut(&FirstResult, &SecondResult) -> Value,
    ) -> Result<Vec<WorthQueryManagedDerivedViewKey>, Denial>
    where
        FirstResult: WorthQueryApplicationProjection<Schema, FirstQuery>,
        SecondResult: WorthQueryApplicationProjection<Schema, SecondQuery>,
        WorthQueryApplicationOneShotResult<FirstQuery, FirstResult>: Send,
        WorthQueryApplicationOneShotResult<SecondQuery, SecondResult>: Send,
        Member: WorthQueryManagedDerivedMemberToken,
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
        let tokens = expected
            .iter()
            .map(|(root, member)| (*root, RetainedMemberToken::new(member.clone())))
            .collect();
        let expected = expected.into_iter().collect::<Vec<_>>();
        let mut entries = Vec::with_capacity(expected.len());
        let mut keys = Vec::with_capacity(expected.len());
        read_bounded_pairs(&expected, &read_pair, |index, first, second| {
            let root = expected[index].0;
            let (key, value, dependencies) = self.read_managed_entry_pair_from_result(
                view,
                product,
                root,
                first,
                |_| Ok(second),
                |row| first_link(row),
                |row| second_link(row),
                |first, second| project(first, second),
            )?;
            keys.push(key.clone());
            entries.push((key, value, dependencies));
            Ok(())
        })?;
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
