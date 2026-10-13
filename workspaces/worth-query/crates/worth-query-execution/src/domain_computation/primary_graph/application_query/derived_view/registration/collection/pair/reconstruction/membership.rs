//! Authenticate the census and charge canonical reconstruction membership staging.
use super::super::reconstruction_denial as denial;
use super::super::{
    Denial, WorthQueryApplicationOneShotResult, WorthQueryManagedDerivedView,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use super::owner_stage;
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::derived_view::{
    dependency::ViewDependency,
    retention::{WorthQueryManagedDerivedMemberToken, WorthQueryManagedDerivedValue},
};
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

pub(in super::super) fn prepare<Schema: ApplicationSchema, Query, Row, Member, Value>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    lease: Option<&ExecutionResourceLease<'_>>,
    hold: &mut ExecutionMemoryReservation,
    view: &WorthQueryManagedDerivedView<Query, Value>,
    product: &WorthQueryProductBranchLease,
    result: &WorthQueryApplicationOneShotResult<Query, Row>,
    members: impl Fn(&Row) -> Vec<(EntityId, Member)>,
) -> Result<(BTreeMap<EntityId, Member>, BTreeSet<ViewDependency>), Denial>
where
    Member: WorthQueryManagedDerivedMemberToken,
    Value: WorthQueryManagedDerivedValue,
{
    owner_stage::run(lease, |context| {
        context
            .checkpoint(1)
            .map_err(|stop| denial::kernel(stop, None))?;
        runtime.admit_view_product(view, product)?;
        if result.rows().len() != result.observed_sources().len() {
            return Err(Denial::IncompleteDependencies);
        }
        let mut membership = runtime.checked_view_dependencies(
            view,
            product,
            result.result_set_observation().source(),
        )?;
        owner_stage::retain_dependencies(hold, &membership)?;
        let mut expected = BTreeMap::new();
        for (row, source) in result.rows().iter().zip(result.observed_sources()) {
            context
                .checkpoint(1)
                .map_err(|stop| denial::kernel(stop, None))?;
            let dependencies = runtime.checked_view_dependencies(view, product, source)?;
            owner_stage::retain_dependencies(hold, &dependencies)?;
            membership.extend(dependencies);
            for (root, member) in members(row) {
                owner_stage::retain_member(hold, &member)?;
                if expected.insert(root, member).is_some() {
                    return Err(Denial::DuplicateRoot { root });
                }
                if view.state.limits.rejects_entries(expected.len()) {
                    return Err(Denial::EntryCapacityExceeded);
                }
            }
        }
        Ok((expected, membership))
    })
}
