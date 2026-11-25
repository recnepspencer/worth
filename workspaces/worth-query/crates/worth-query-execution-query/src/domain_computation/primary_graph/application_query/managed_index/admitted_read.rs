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
    association: worth_query_installation::facade::WorthQueryInstalledManagedIndexParent,
    native: &mut ApplicationQueryNativeAccess<'_>,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<CompletedOneShotRead<B::Query, ResultValue<S, B>>, Denial>
where
    S: ApplicationSchema + 'static,
    B: ApplicationQueryBinding<S>,
    ResultValue<S, B>: WorthQueryApplicationProjection<S, B::Query>,
{
    let application = access.application;
    let selector = retain_scope(execution, plan.scope, plan.query.name()).map_err(|cause| {
        SourceQueryObservationStop::Read(super::super::WorthQueryAdmittedOneShotStop::Execution(
            cause,
        ))
    })?;
    let adapter = application
        .managed_indexes
        .executor(execution, &association, allocation)?
        .ok_or(Denial::ForeignAssociation)?;
    let probe = parent_probe::<S, B>(execution, &plan, &association, allocation)?;
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
            let mut raw = read_execution::read_bounded_plan_rows(
                runtime,
                execution,
                layout,
                &read,
                Some(&spent),
                ReadInterruption::Query(plan.controls.request_scope()),
            )?;
            if ResultValue::<S, B>::PROJECTS_PERSISTENT_NAMES {
                read_execution::persistent_names::observe_persistent_names(
                    runtime, execution, plan, &mut raw,
                );
            }
            Ok(raw)
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

pub(super) fn parent_probe<'p, S, B>(
    execution: &ExecutionRequest<'_, '_>,
    plan: &'p BoundPlan<'_, S, B>,
    association: &worth_query_installation::facade::WorthQueryInstalledManagedIndexParent,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<&'p worth_foundational::facade::AspectValue, Denial>
where
    S: ApplicationSchema,
    B: ApplicationQueryBinding<S>,
{
    let parameter = &association
        .access_contract()
        .meaning()
        .parent_graph()
        .managed_index_selection()
        .ok_or(Denial::ForeignAssociation)?
        .parts()
        .probe_parameter;
    let units = parameter.len().checked_add(1)
        .and_then(|width| width.checked_mul(plan.parameters.bindings().len()))
        .and_then(|units| u64::try_from(units).ok())
        .ok_or(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow)?;
    MaterialCompletion::new(execution, allocation).work(units)?;
    plan.parameters
        .bindings()
        .iter()
        .find(|(name, _)| name.as_ref() == parameter.as_str())
        .map(|(_, value)| value)
        .ok_or(Denial::ForeignAssociation)
}

pub enum DecisionSource {
    Committed(Arc<crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead>),
    Applied {
        facts: Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact>,
        authorization: Arc<crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization>,
        allocation: Arc<SourceFactsAllocation>,
    },
}

pub fn admit_parent<S, B>(
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

pub fn read_decision<S, B>(
    execution: &ExecutionRequest<'_, '_>,
    access: &SourceQueryReaderAccess<'_, S>,
    intent: B::Input,
    context: &std::cell::Cell<
        crate::domain_computation::primary_graph::handler::DecisionContextUse,
    >,
    admit_scope: &mut dyn FnMut(
        worth_relational::facade::identity::EntityId,
        worth_relational::facade::identity::EntityId,
    ) -> bool,
    native: &mut ApplicationQueryNativeAccess<'_>,
    applied: Option<super::AppliedManagedIndexRead<'_>>,
) -> Result<(Vec<ResultValue<S, B>>, DecisionSource), super::WorthQueryManagedIndexDenial>
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
    use super::denial::{managed_index_denial, ReceiverCause};
    let allocation = SourceFactsAllocation::open(execution)
        .map_err(|cause| managed_index_denial(ReceiverCause::Execution(cause), None))?;
    let consume = |plan: BoundPlan<'_, S, B>, native: &mut ApplicationQueryNativeAccess<'_>| {
        // The resolved scope and principal are admitted by the operation's
        // declared authority before any Parent or provider work.
        if !admit_scope(plan.scope.entity_id(), plan.graph_work.principal()) {
            return Err(Denial::ForeignAssociation);
        }
        let association = admit_parent::<S, B>(execution, access, &plan, native, &allocation)?;
        // An aligned applied read keeps the committed index probe; its claim
        // is the read's candidate support. Anything else reads the candidate.
        let alignment = match applied {
            None => None,
            Some(applied) => match applied.aligned(
                execution, access.application, &association, plan.scope.entity_id(),
                plan.query.read_graph(),
                &worth_query_declaration::facade::application_query::ApplicationQueryBindingReference::of::<S, B>(),
            )? {
                Some(alignment) => Some(alignment),
                None => return super::applied_read::read_decision::<S, B>(
                    execution, access, plan, applied, association, &allocation, native,
                ),
            },
        };
        let completed = read::<S, B>(execution, access, plan, association, native, &allocation)?;
        let dependency = completed
            .read
            .result_set_observation()
            .source()
            .invocation_managed_source()
            .map_err(|_| Denial::ForeignAssociation)?
            .ok_or(Denial::ForeignAssociation)?;
        let dependency = DecisionSource::Committed(match alignment {
            Some(alignment) => {
                allocation.claim_arc::<crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead>(execution)?;
                Arc::new(dependency.as_ref().clone().with_applied_alignment(alignment))
            }
            None => Arc::clone(dependency),
        });
        Ok((completed.read.into_rows(), dependency))
    };
    let result = match applied {
        Some(applied) => {
            context.set(context.get().source_query_permission());
            access.with_applied_plan::<B, _, Denial>(
                execution,
                intent,
                applied.before,
                applied.candidate,
                applied.own,
                native,
                consume,
            )
        }
        None => access.with_plan::<B, _, Denial>(execution, intent, context, native, consume),
    };
    result.map_err(|cause| managed_index_denial(ReceiverCause::Provider(cause), Some(allocation)))
}

/// The already resolved selected Source consumes the same Native transport.
pub fn read_source_admitted<S, B>(
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
    use super::denial::{managed_index_denial, ReceiverCause};
    let storage_stop =
        |cause: worth_relational::facade::transactions::RelationalExecutionDenialCause| {
            SourceQueryObservationStop::Read(crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedOneShotStop::Admission(worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause)))
        };
    let allocation = SourceFactsAllocation::open(execution).map_err(storage_stop)?;
    let result = admit_parent::<S, B>(execution, access, &plan, native, &allocation).and_then(
        |association| read::<S, B>(execution, access, plan, association, native, &allocation),
    );
    result.map_err(|cause| {
        match managed_index_denial(ReceiverCause::Provider(cause), Some(allocation))
            .retain(execution)
        {
            Ok(cause) => SourceQueryObservationStop::ManagedIndex(cause),
            Err(cause) => storage_stop(cause),
        }
    })
}
