//! Resolve the installed Parent and independently admitted Source on one real prefix.
use super::*;
use crate::domain_computation::primary_graph::application_query::{
    one_shot::outcome::material::{retain_scope, MaterialCompletion},
    output_source_observation::SourceReadPlan,
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
    access: &SourceQueryReaderAccess<'_, S>,
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
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
    let parameter = &association
        .access_contract()
        .meaning()
        .parent_graph()
        .managed_index_selection()
        .ok_or(Denial::ForeignAssociation)?
        .parts()
        .probe_parameter;
    let units = plan.parameters.bindings().len().checked_mul(parameter.len().checked_add(1)
        .ok_or(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow)?)
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow)?;
    MaterialCompletion::new(execution, allocation).work(units)?;
    let probe = plan
        .parameters
        .bindings()
        .iter()
        .find(|(name, _)| name.as_ref() == parameter.as_str())
        .map(|(_, value)| value)
        .ok_or(Denial::ForeignAssociation)?;
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
