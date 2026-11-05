//! The decision reader's actual Parent and Source admissions share its Native borrow.
use super::{
    execution::ProviderExecutionDenial as Denial,
    receiver::{BoundPlan, ResultValue},
};
use crate::domain_computation::primary_graph::{
    application_query::{
        authorized_read::execute_selected_authorized_body_with_native_access,
        one_shot::{
            self,
            outcome::{
                complete_one_shot,
                material::{retain_scope, MaterialCompletion},
                CompletedOneShotRead,
            },
        },
        output_source_observation::{SourceQueryObservationStop, SourceQueryReaderAccess},
        read_execution::{self, read_plan::ReadPlan, OneShotReadWorkObservation, ReadInterruption},
        ApplicationQueryNativeAccess, WorthQueryApplicationProjection,
    },
    source_facts::SourceFactsAllocation,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::{
    application_query::ApplicationQueryBinding, application_schema::ApplicationSchema,
};

fn read<S, B>(
    execution: &ExecutionRequest<'_, '_>,
    access: &SourceQueryReaderAccess<'_, S>,
    plan: BoundPlan<'_, S, B>,
    native: &mut ApplicationQueryNativeAccess<'_>,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<CompletedOneShotRead<B::Query, ResultValue<S, B>>, Denial>
where
    S: ApplicationSchema + 'static,
    B: ApplicationQueryBinding<S>,
    ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
{
    let application = access.application;
    let storage = MaterialCompletion::new(execution, allocation);
    let association = admit_parent::<S, B>(execution, access, &plan, native, allocation)?;
    let selector = retain_scope(execution, plan.scope, plan.query.name()).map_err(|cause| {
        SourceQueryObservationStop::Read(super::super::WorthQueryAdmittedOneShotStop::Execution(
            cause,
        ))
    })?;
    let adapter = application
        .application_runtime()
        .managed_indexes
        .executor(execution, &association, allocation)?
        .ok_or(Denial::ForeignAssociation)?;
    let parameter = &association
        .access_contract()
        .meaning()
        .parent_graph()
        .managed_index_selection()
        .ok_or(Denial::ForeignAssociation)?
        .parts()
        .probe_parameter;
    storage.work(u64::try_from(plan.parameters.bindings().len())
        .ok().and_then(|count| count.checked_mul(parameter.len() as u64 + 1))
        .ok_or(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow)?)?;
    let probe = plan
        .parameters
        .bindings()
        .iter()
        .find(|(name, _)| *name == parameter.as_str())
        .map(|(_, value)| value)
        .ok_or(Denial::ForeignAssociation)?;
    let lease = adapter.select_admitted(
        execution,
        application,
        access,
        &plan.graph_work,
        plan.controls.request_scope(),
        &association,
        &selector,
        probe,
        &plan.parameters,
        &plan.governance,
        allocation,
        native,
    )?;
    let rows = lease
        .rows(&plan.graph_work, plan.controls.request_scope())
        .map_err(Denial::Selection)?;
    let spent = OneShotReadWorkObservation::new();
    let (raw, authorization, proof) = execute_selected_authorized_body_with_native_access(
        execution,
        application,
        &plan,
        native,
        |runtime, layout, plan| {
            let mut read = ReadPlan::of(plan);
            read.managed_rows = Some(rows);
            read.material_source = Some(allocation);
            read_execution::read_bounded_plan_rows(
                runtime,
                execution,
                layout,
                &read,
                Some(&spent),
                ReadInterruption::Query(plan.controls.request_scope()),
            )
        },
    )
    .map_err(|cause| {
        SourceQueryObservationStop::Read(one_shot::map_selected_read_stop(cause, plan.query.name()))
    })?;
    lease
        .rows(&plan.graph_work, plan.controls.request_scope())
        .map_err(Denial::Selection)?;
    drop(lease);
    one_shot::settle_admitted_read(
        &spent,
        complete_one_shot(execution, application, plan, raw, authorization, proof),
    )
    .map_err(|cause| Denial::SourceRead(SourceQueryObservationStop::Read(cause)))
}

fn admit_parent<S, B>(
    execution: &ExecutionRequest<'_, '_>,
    access: &SourceQueryReaderAccess<'_, S>,
    plan: &BoundPlan<'_, S, B>,
    native: &mut ApplicationQueryNativeAccess<'_>,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<worth_query_installation::facade::WorthQueryInstalledManagedIndexParent, Denial>
where
    S: ApplicationSchema + 'static,
    B: ApplicationQueryBinding<S>,
    ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
{
    let application = access.application;
    let storage = MaterialCompletion::new(execution, allocation);
    let binding = application
        .installed_schema()
        .installed_query_binding::<B>()
        .map_err(SourceQueryObservationStop::Installation)?;
    let association = application.installed_schema().installed_managed_index_parent(&binding,
        &mut |units, bytes| { storage.work(units)?; allocation.claim(execution, bytes) })
        .map_err(|cause| match cause {
            worth_query_installation::facade::WorthQueryInstalledQueryBindingAdmissionStop::Admission(cause) => Denial::Execution(cause),
            _ => Denial::ForeignAssociation,
        })?.ok_or(Denial::ForeignAssociation)?;
    one_shot::plan_admission::validate_one_shot_plan_with_native_access(
        execution,
        application,
        plan,
        native,
    )
    .map_err(|cause| {
        SourceQueryObservationStop::Read(super::super::WorthQueryAdmittedOneShotStop::Execution(
            cause,
        ))
    })?;
    // Parent READ must be current before any provider call. This is the same
    // selected security receiver that surrounds the subsequent projection.
    execute_selected_authorized_body_with_native_access(
        execution,
        application,
        plan,
        native,
        |_, _, _| Ok(()),
    )
    .map_err(|cause| {
        SourceQueryObservationStop::Read(one_shot::map_selected_read_stop(cause, plan.query.name()))
    })?;
    Ok(association)
}

pub(in crate::domain_computation::primary_graph) fn read_decision<S, B>(
    execution: &ExecutionRequest<'_, '_>,
    access: &SourceQueryReaderAccess<'_, S>,
    intent: B::Input,
    context: &std::cell::Cell<
        crate::domain_computation::primary_graph::handler::DecisionContextUse,
    >,
    operation_scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
    native: &mut ApplicationQueryNativeAccess<'_>,
) -> Result<
    (
        Vec<ResultValue<S, B>>,
        Arc<crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead>,
    ),
    super::WorthQueryManagedIndexDenial,
>
where
    S: ApplicationSchema,
    B: ApplicationQueryBinding<S>,
    B::Input:
        worth_query_declaration::facade::application_query::ApplicationQueryIntent<S, Binding = B>,
    B::ScopeBinding:
        worth_query_declaration::facade::application_query::ApplicationQueryScopeResolution<
            S,
            B::PrincipalIdentity,
        >,
    ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
{
    use super::denial::{ReceiverCause, WorthQueryManagedIndexDenial};
    let allocation = SourceFactsAllocation::open(execution).map_err(|cause| {
        WorthQueryManagedIndexDenial::new(ReceiverCause::Execution(cause), None)
    })?;
    let result =
        access.with_plan::<B, _, Denial>(execution, intent, context, native, |plan, native| {
            if crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(plan.scope.entity_id()) != operation_scope.scope()
                || crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(plan.graph_work.principal()) != operation_scope.principal() {
                return Err(Denial::ForeignAssociation);
            }
            let completed = read::<S, B>(execution, access, plan, native, &allocation)?;
            let dependency = completed
                .read
                .result_set_observation()
                .source()
                .invocation_managed_source()
                .map_err(|_| Denial::ForeignAssociation)?
                .ok_or(Denial::ForeignAssociation)?;
            let dependency = Arc::clone(dependency);
            Ok((completed.read.into_rows(), dependency))
        });
    result.map_err(|cause| {
        WorthQueryManagedIndexDenial::new(ReceiverCause::Provider(cause), Some(allocation))
    })
}

/// The already resolved selected Source consumes the same Native transport.
pub(in crate::domain_computation::primary_graph::application_query) fn read_source_admitted<S, B>(
    execution: &ExecutionRequest<'_, '_>,
    access: &SourceQueryReaderAccess<'_, S>,
    plan: BoundPlan<'_, S, B>,
    native: &mut ApplicationQueryNativeAccess<'_>,
) -> Result<CompletedOneShotRead<B::Query, ResultValue<S, B>>, SourceQueryObservationStop>
where
    S: ApplicationSchema,
    B: ApplicationQueryBinding<S>,
    ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
{
    use super::denial::{ReceiverCause, WorthQueryManagedIndexDenial};
    let storage_stop =
        |cause: worth_relational::facade::transactions::RelationalExecutionDenialCause| {
            SourceQueryObservationStop::Read(crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedOneShotStop::Admission(worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause)))
        };
    let allocation = SourceFactsAllocation::open(execution).map_err(storage_stop)?;
    read::<S, B>(execution, access, plan, native, &allocation).map_err(|cause| {
        match WorthQueryManagedIndexDenial::new(ReceiverCause::Provider(cause), Some(allocation))
            .retain(execution)
        {
            Ok(cause) => SourceQueryObservationStop::ManagedIndex(cause),
            Err(cause) => storage_stop(cause),
        }
    })
}
