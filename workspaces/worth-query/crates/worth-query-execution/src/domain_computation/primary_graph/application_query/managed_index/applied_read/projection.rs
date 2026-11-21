//! The real admitted Parent consumes its candidate Source/provider loan.
use super::*;
use crate::domain_computation::primary_graph::application_query::{
    authorized_read::execute_selected_authorized_body_with_native_access,
    one_shot::map_selected_read_stop,
    output_source_observation::{SourceQueryObservationStop as Stop, SourceReadPlan},
    read_execution::{
        applied_source::{self, AppliedSourceProjection},
        read_plan::ReadPlan,
        ReadInterruption,
    },
    WorthQueryAdmittedOneShotStop, WorthQueryApplicationProjection,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_relational::facade::{
    mvcc::RelationalAuthoredCoupledCandidate,
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead},
};

pub(in crate::domain_computation::primary_graph::application_query) fn project<S, B, Value>(
    execution: &ExecutionRequest<'_, '_>,
    access: &SourceQueryReaderAccess<'_, S>,
    plan: &SourceReadPlan<'_, S, B>,
    before: &RelationalPinnedRead,
    candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    assembly: &RelationalAuthoredCoupledCandidate<'_>,
    native: &mut ApplicationQueryNativeAccess<'_>,
) -> Result<
    (
        AppliedSourceProjection<Value>,
        Arc<AppliedManagedSourceSupport>,
        crate::domain_computation::provider_session::WorthQuerySessionGraphReadProof,
    ),
    Stop,
>
where
    S: ApplicationSchema,
    B: ApplicationQueryBinding<S>,
    super::super::receiver::ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    Value: WorthQueryApplicationProjection<S, B::Query>,
{
    let allocation = SourceFactsAllocation::open(execution).map_err(storage_stop)?;
    let retained_stop = |cause| provider_stop(execution, cause, &allocation);
    let lease = prepare::<S, B>(
        execution,
        access,
        access.application,
        plan,
        before,
        candidate,
        assembly,
        &allocation,
        native,
    )
    .map_err(retained_stop)?;
    let rows = lease
        .rows(
            &plan.graph_work,
            plan.controls.request_scope(),
            candidate.candidate_identity(),
        )
        .map_err(retained_stop)?;
    let application = access.application;
    let (projected, _, proof) = execute_selected_authorized_body_with_native_access(
        execution,
        application,
        plan,
        native,
        |_, layout, admitted| {
            let mut read = ReadPlan::of(admitted);
            read.applied_managed_rows = Some(&rows);
            read.material_source = Some(&allocation);
            Ok(applied_source::read_applied_source::<S, B::Query, Value>(
                execution,
                &crate::domain_computation::primary_graph::value_decode::ValueDecodeState::new(
                    application.installed_schema.value_bindings().clone(),
                ),
                candidate,
                layout,
                &read,
                ReadInterruption::Query(admitted.controls.request_scope()),
            ))
        },
    )
    .map_err(|cause| Stop::Read(map_selected_read_stop(cause, plan.query.name())))?;
    // Capacity/association custody must still be live after Native materialization.
    lease
        .rows(
            &plan.graph_work,
            plan.controls.request_scope(),
            candidate.candidate_identity(),
        )
        .map_err(retained_stop)?;
    let projected = projected.map_err(|cause| cause.into_source_query_stop(plan.query.name()))?;
    let support = lease.retain_support(execution).map_err(retained_stop)?;
    Ok((projected, support, proof))
}
fn storage_stop(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> Stop {
    Stop::Read(WorthQueryAdmittedOneShotStop::Admission(
        worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause),
    ))
}
fn provider_stop(
    execution: &ExecutionRequest<'_, '_>,
    cause: Denial,
    allocation: &Arc<SourceFactsAllocation>,
) -> Stop {
    match super::super::denial::managed_index_denial(
        super::super::denial::ReceiverCause::Provider(cause),
        Some(allocation.clone()),
    )
    .retain(execution)
    {
        Ok(cause) => Stop::ManagedIndex(cause),
        Err(cause) => storage_stop(cause),
    }
}
