//! Resolve the installed Parent and independently admitted Source on one real prefix.
use super::*;
use crate::domain_computation::primary_graph::application_query::{
    one_shot::outcome::material::retain_scope, output_source_observation::SourceReadPlan,
    WorthQueryApplicationProjection,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_relational::facade::{
    mvcc::RelationalAuthoredCoupledCandidate,
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead},
};

pub fn prepare<'a, 'request, 'authority, S, B>(
    execution: &ExecutionRequest<'request, 'authority>,
    access: &SourceQueryReaderAccess<'a, S>,
    plan: &'a SourceReadPlan<'_, S, B>,
    before: &RelationalPinnedRead,
    candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    assembly: &RelationalAuthoredCoupledCandidate<'_>,
    allocation: &Arc<SourceFactsAllocation>,
    native: &mut ApplicationQueryNativeAccess<'_>,
) -> Result<AppliedManagedSelectionLease<'a, 'request, 'authority>, Denial>
where
    S: ApplicationSchema,
    B: ApplicationQueryBinding<S>,
    super::super::receiver::ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
{
    // Reuse the binding-aware Parent admission. Its selected READ proves the
    // original principal/request before either Source acquisition or provider work.
    let association = super::super::admitted_read::admit_parent::<S, B>(
        execution, access, plan, native, allocation,
    )?;
    select_parent::<S, B>(
        execution,
        access,
        plan,
        before,
        candidate,
        assembly,
        association,
        allocation,
        native,
    )
}

pub(super) fn select_parent<'a, 'request, 'authority, S, B>(
    execution: &ExecutionRequest<'request, 'authority>,
    access: &SourceQueryReaderAccess<'a, S>,
    plan: &'a SourceReadPlan<'_, S, B>,
    before: &RelationalPinnedRead,
    candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    assembly: &RelationalAuthoredCoupledCandidate<'_>,
    association: WorthQueryInstalledManagedIndexParent,
    allocation: &Arc<SourceFactsAllocation>,
    native: &mut ApplicationQueryNativeAccess<'_>,
) -> Result<AppliedManagedSelectionLease<'a, 'request, 'authority>, Denial>
where
    S: ApplicationSchema,
    B: ApplicationQueryBinding<S>,
{
    let application = access.application;
    let selector = retain_scope(execution, plan.scope, plan.query.name()).map_err(|cause| {
        crate::domain_computation::primary_graph::application_query::output_source_observation::SourceQueryObservationStop::Read(
            crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedOneShotStop::Execution(cause),
        )
    })?;
    let adapter = application
        .managed_indexes
        .executor(execution, &association, allocation)?
        .ok_or(Denial::ForeignAssociation)?;
    let graph = application
        .runtime
        .primary_graph()
        .ok_or(Denial::ForeignAssociation)?;
    let program = graph
        .layout_ref()
        .managed_source(&association.definition().parts().name)
        .ok_or(Denial::ForeignAssociation)?;
    let loan = assembly
        .read_managed_source_at(execution, program, selector.subject(), candidate)
        .map_err(Denial::NativeSource)?;
    let probe = super::super::admitted_read::parent_probe::<S, B>(
        execution,
        plan,
        &association,
        allocation,
    )?;
    adapter.select_applied(
        execution,
        application,
        access,
        &plan.graph_work,
        plan.controls.request_scope(),
        &association,
        &selector,
        before,
        &loan,
        probe,
        allocation,
        native,
    )
}
