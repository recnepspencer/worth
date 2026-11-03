//! Fresh public Source READ precedes the existing managed-provider purpose.
use super::{
    adapter::InstalledIndexProvider,
    execution::{purpose, ProviderExecutionDenial as Denial},
    selection::ManagedSelectionLease,
    source::service::ManagedSourceRoot,
    source_image::SourceBinding,
};
use crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance;
use crate::domain_computation::{
    primary_graph::{
        application_attempt::ManagedSourceRead,
        application_query::{
            authorized_read::execute_selected_authorized_body_with_native_access,
            one_shot::{map_selected_read_stop, outcome::material::MaterialCompletion},
            output_source_observation::{SourceQueryObservationStop, SourceQueryReaderAccess},
            ApplicationQueryNativeAccess, WorthQueryObservedScopeSelector,
        },
        source_facts::SourceFactsAllocation,
    },
    provider_session::{
        WorthQueryCompletedSourceRead, WorthQueryManagedGraphWorkSession,
        WorthQueryObservedGraphReadWork,
    },
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters as Parameters;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndexParent;
use worth_relational::facade::runtime::{
    ManagedIndexSourceObservation, ManagedIndexSourcePreparationDenial,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn select<'a, 'request, 'authority, S, I>(
    execution: &ExecutionRequest<'request, 'authority>,
    application: &'a crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
    access: &SourceQueryReaderAccess<'_, S>,
    parent: &'a WorthQueryManagedGraphWorkSession,
    request: &'a WorthQueryRequestScope,
    association: &WorthQueryInstalledManagedIndexParent,
    selector: &WorthQueryObservedScopeSelector,
    adapter: &Arc<dyn InstalledIndexProvider<S, I>>,
    probe: &AspectValue,
    parameters: &Parameters,
    governance: &WorthQueryApplicationQueryGovernance,
    allocation: &Arc<SourceFactsAllocation>,
    native: &mut ApplicationQueryNativeAccess<'_>,
) -> Result<ManagedSelectionLease<'a, 'request, 'authority>, Denial>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
{
    let (source, observation, authorization) = read_source::<S, I>(
        execution,
        access,
        parent,
        association,
        selector,
        adapter,
        allocation,
        native,
    )?;
    let root = observation
        .payload()
        .downcast_ref::<ManagedSourceRoot<S, I>>()
        .ok_or(Denial::ForeignAssociation)?;
    if !root.installed.same_installation(adapter.installed())
        || !Arc::ptr_eq(adapter, &root.adapter)
    {
        return Err(Denial::ForeignAssociation);
    }
    let purpose = purpose::start(
        execution,
        application,
        parent,
        request,
        association,
        Arc::clone(&source),
        allocation,
    )?;
    let frame =
        crate::domain_computation::primary_graph::access_strategy::coverage::CoverageFrame::public(
            root.rows.view(&root.bindings, None),
            &root.nodes,
            root,
            probe,
            parent,
            association.access_contract().meaning().match_posture(),
        );
    let (selection, callback) = purpose
        .call(|actual| adapter.candidates(actual, frame))
        .map_err(Denial::Session)?
        .map_err(Denial::Work)?;
    let slots = super::invocation::controlled(execution, |control| selection.receive(
        frame, adapter.coverage_authority(),
        association.definition().parts().membership != worth_query_declaration::facade::application_query::ApplicationManagedIndexMembership::Candidates,
        control,
    )).map_err(Denial::Work)?;
    let proof = purpose.complete(callback).map_err(Denial::Session)?;
    let influence = super::realized_footprint::capture(
        execution,
        &proof,
        super::realized_footprint::CaptureInput {
            parent,
            request,
            association,
            source: &source,
            root,
            observation: &observation,
            parameters,
            governance,
            probe,
            selected: &slots,
            allocation,
        },
    )
    .map_err(Denial::Selection)?;
    let storage = MaterialCompletion::new(execution, allocation);
    storage.arc::<ManagedSourceRead>()?;
    let expectation = Arc::new(ManagedSourceRead::new(
        *association
            .access_contract()
            .graph_obligations()
            .identity()
            .bytes(),
        selector.subject(),
        Arc::clone(&observation),
        Arc::clone(allocation),
        influence,
        *parameters.identity().bytes(),
        authorization,
    ));
    let dependencies =
        super::source_dependencies::capture_maintained(execution, expectation, allocation)
            .map_err(Denial::SourceCompletion)?;
    ManagedSelectionLease::issue_maintained(
        proof,
        parent,
        request,
        association,
        source,
        Arc::clone(&observation),
        &root.rows,
        slots,
        dependencies,
        allocation,
        adapter.coverage_authority().clone(),
    )
    .map_err(Denial::Selection)
}

#[allow(clippy::too_many_arguments)]
fn read_source<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>(
    execution: &ExecutionRequest<'_, '_>,
    access: &SourceQueryReaderAccess<'_, S>,
    parent: &WorthQueryManagedGraphWorkSession,
    association: &WorthQueryInstalledManagedIndexParent,
    selector: &WorthQueryObservedScopeSelector,
    adapter: &Arc<dyn InstalledIndexProvider<S, I>>,
    allocation: &Arc<SourceFactsAllocation>,
    native: &mut ApplicationQueryNativeAccess<'_>,
) -> Result<
    (
        Arc<WorthQueryCompletedSourceRead>,
        Arc<ManagedIndexSourceObservation>,
        Arc<crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization>,
    ),
    Denial,
> {
    let application = access.application;
    let graph = application
        .runtime
        .primary_graph()
        .ok_or(Denial::ForeignAssociation)?;
    let program = graph
        .layout
        .managed_source(&adapter.installed().definition().parts().name)
        .ok_or(Denial::ForeignAssociation)?;
    let parent_context = parent
        .retain_query_source_context()
        .map_err(Denial::GraphRead)?;
    access.with_same_scope_plan::<SourceBinding<S, I>, _, Denial>(
        execution,
        selector,
        parent_context.principal(),
        allocation,
        native,
        |mut plan, native| {
            if !adapter.installed().admits_source(plan.query)
                || !matches!(
                    plan.governance,
                    super::super::disclosure::WorthQueryApplicationQueryGovernance::Public
                )
                || plan.retain_expression_context
                || plan.query.graph_obligations().identity()
                    != association.access_contract().source_obligation()
            {
                return Err(Denial::ForeignAssociation);
            }
            let binding = plan.query.binding_observation().cloned().ok_or(Denial::ForeignAssociation)?;
            let source_identity = *plan.query.identity().as_bytes();
            let context = plan
                .graph_work
                .retain_query_source_context()
                .map_err(Denial::GraphRead)?;
            let (observation, _, proof) = execute_selected_authorized_body_with_native_access(
                execution,
                application,
                &plan,
                native,
                |runtime, _, source_plan| {
                    Ok(runtime.read_truth().observe_managed_index_source(
                        execution,
                        source_plan.basis.snapshot_handle(),
                        program,
                        selector.subject(),
                    ))
                },
            )
            .map_err(|stop| {
                SourceQueryObservationStop::Read(map_selected_read_stop(stop, plan.query.name()))
            })?;
            let observation = observation.map_err(Denial::NativeSource)?;
            let root = observation
                .payload()
                .downcast_ref::<ManagedSourceRoot<S, I>>()
                .ok_or(Denial::ForeignAssociation)?;
            if root.rows.len() > plan.controls.maximum_result_count().get() {
                return Err(Denial::NativeSource(ManagedIndexSourcePreparationDenial::Source(
                    worth_relational::facade::runtime::DerivedRelationReadDenial::Definition(
                        worth_relational::facade::schema::DerivedRelationDenial::WorkBudgetExceeded,
                    ),
                )));
            }
            // This read looked up one exact complete association. It did not
            // enumerate Source candidates or project/recount cached rows.
            let storage = MaterialCompletion::new(execution, allocation);
            storage.work(1)?;
            storage.arc::<ManagedIndexSourceObservation>()?;
            let observation = Arc::new(observation);
            let release = plan.basis.release();
            let completion = plan
                .graph_work
                .complete_query_read(
                    proof,
                    WorthQueryObservedGraphReadWork {
                        examined_candidates: 0,
                        projected_records: 0,
                        projected_fields: 0,
                        relation_records_examined: 0,
                        ordering_comparisons: 0,
                    },
                    release,
                )
                .map_err(Denial::GraphRead)?;
            let terminal = completion
                .into_managed_source(context, execution, allocation, |units| storage.work(units))?
                .ok_or(Denial::ForeignAssociation)?;
            let authorization = crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization::complete(
                execution, &terminal, binding, source_identity, plan.authorization, &application.authorization, allocation,
            ).map_err(Denial::NativeAdmission)?.ok_or(Denial::ForeignAssociation)?;
            storage.arc::<WorthQueryCompletedSourceRead>()?;
            Ok((Arc::new(terminal), observation, authorization))
        },
    )
}
