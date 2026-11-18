//! Independently admit Source READ before borrowing an exact Native applied image.
use super::super::{source::occurrences::PersistentSourceRows, source_image::SourceBinding};
use super::*;
use crate::domain_computation::primary_graph::application_query::{
    authorized_read::execute_selected_authorized_body_with_native_access,
    one_shot::{map_selected_read_stop, outcome::material::MaterialCompletion},
    output_source_observation::SourceQueryObservationStop,
};

pub(super) struct AdmittedSource {
    pub(super) read: Arc<WorthQueryAppliedSourceRead>,
    pub(super) material: Payload,
    pub(super) support: super::super::source::projection_support::AppliedSourceAcquisitionSupport,
    pub(super) authorization: Arc<SourceReadAuthorization>,
    pub(super) parameters: [u8; 32],
}
pub(super) fn acquire<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>(
    execution: &ExecutionRequest<'_, '_>,
    access: &SourceQueryReaderAccess<'_, S>,
    parent: &WorthQueryManagedGraphWorkSession,
    association: &WorthQueryInstalledManagedIndexParent,
    selector: &WorthQueryObservedScopeSelector,
    adapter: &Arc<dyn InstalledIndexProvider<S, I>>,
    before: &worth_relational::facade::runtime::RelationalPinnedRead,
    loan: &ManagedIndexAppliedSourceRead<'_>,
    allocation: &Arc<SourceFactsAllocation>,
    native: &mut ApplicationQueryNativeAccess<'_>,
) -> Result<AdmittedSource, Denial> {
    let application = access.application;
    let graph = application
        .runtime
        .primary_graph()
        .ok_or(Denial::ForeignAssociation)?;
    let program = graph
        .layout
        .managed_source(&adapter.installed().definition().parts().name)
        .ok_or(Denial::ForeignAssociation)?;
    if program != loan.program() || selector.subject() != loan.scope() {
        return Err(Denial::ForeignAssociation);
    }
    let parent_context = parent
        .retain_query_source_context()
        .map_err(Denial::GraphRead)?;
    access.with_same_scope_plan_on::<SourceBinding<S, I>, _, Denial>(
        execution,
        selector,
        parent_context.principal(),
        allocation,
        Some((before, loan.candidate_pin())),
        native,
        |plan, native| {
            if !adapter.installed().admits_source(plan.query)
                || !matches!(
                    plan.governance,
                    super::super::super::disclosure::WorthQueryApplicationQueryGovernance::Public
                )
                || plan.retain_expression_context
                || plan.query.graph_obligations().identity()
                    != association.access_contract().source_obligation()
            {
                return Err(Denial::ForeignAssociation);
            }
            let binding = plan
                .query
                .binding_observation()
                .cloned()
                .ok_or(Denial::ForeignAssociation)?;
            let source_identity = *plan.query.identity().as_bytes();
            let context = plan
                .graph_work
                .retain_query_source_context()
                .map_err(Denial::GraphRead)?;
            let (physical, _, proof) = execute_selected_authorized_body_with_native_access(
                execution,
                application,
                &plan,
                native,
                |_, _, source_plan| {
                    // READ admits this SourceBinding/principal/scope. Physical
                    // truth comes exclusively from the compiled applied loan.
                    Ok(loan
                        .rows(*execution)
                        .map_err(Denial::NativeSource)
                        .and_then(|rows| {
                            PersistentSourceRows::acquire_observed(
                                execution,
                                &rows,
                                source_plan.graph_work.identity(),
                                source_plan.query.read_graph(),
                                source_plan.controls.maximum_result_count().get(),
                            )
                            .map_err(|cause| Denial::NativeSource(cause.into_native()))
                        }))
                },
            )
            .map_err(|stop| {
                SourceQueryObservationStop::Read(map_selected_read_stop(stop, plan.query.name()))
            })?;
            let (rows, work, support) = physical?;
            let support = support
                .seal(execution, loan.candidate_pin())
                .map_err(|cause| Denial::NativeSource(cause.into_native()))?;
            let nodes = adapter
                .build(
                    execution,
                    rows.owner(),
                    rows.view(application.installed_schema.value_bindings(), None),
                )
                .map_err(Denial::Work)?;
            let material = super::super::source::service::retain_root(
                execution,
                adapter.clone(),
                application.installed_schema.value_bindings().clone(),
                rows,
                nodes,
            )
            .map_err(Denial::NativeSource)?;
            let completion = plan
                .graph_work
                .complete_query_read(proof, work, plan.basis.release())
                .map_err(Denial::GraphRead)?;
            let terminal = completion
                .into_applied_source(context, loan, execution, allocation)?
                .ok_or(Denial::ForeignAssociation)?;
            let authorization = SourceReadAuthorization::complete_applied(
                execution,
                &terminal,
                binding,
                source_identity,
                plan.authorization,
                &application.runtime.authorization,
                allocation,
            )
            .map_err(Denial::NativeAdmission)?
            .ok_or(Denial::ForeignAssociation)?;
            MaterialCompletion::new(execution, allocation).arc::<WorthQueryAppliedSourceRead>()?;
            Ok(AdmittedSource {
                read: Arc::new(terminal),
                material,
                support,
                authorization,
                parameters: *plan.parameters.identity().bytes(),
            })
        },
    )
}
