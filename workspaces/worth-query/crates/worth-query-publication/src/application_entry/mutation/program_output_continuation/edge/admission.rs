//! A child consumes either its ordinary or its genuine history parent read.
use super::*;

pub(super) fn admit_child<Schema, Program, ParentDemand, Connection>(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    parent: &ProgramOutputParentAuthority<Schema, Program, ParentDemand>,
    basis: &crate::application_entry::WorthQueryApplicationReadObservation,
    minimum: &crate::application_entry::WorthQueryApplicationReadObservation,
    demand: &ChildDemand<Schema, Connection>,
    controls: WorthQueryOutputDemandControls,
    history: Option<ProgramOutputHistoryFrame<'_, '_, '_>>,
) -> Result<WorthQueryApplicationProgramDemandHandle<Schema, Program, ChildDemand<Schema, Connection>>, WorthQueryRequiredOutputPreparationDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
    Connection: ApplicationConnectionShape<Schema>,
    Binding<Schema, Connection>: WorthQueryApplicationDependentOutputConnection<Schema, RootDemand = ParentDemand>,
    ChildDemand<Schema, Connection>: Clone,
    SourceValue<Schema, ChildDemand<Schema, Connection>>: WorthQueryApplicationProjection<Schema, SourceQuery<Schema, ChildDemand<Schema, Connection>>> + Clone,
    <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = Source<Schema, ChildDemand<Schema, Connection>>>,
    <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<Schema, <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
{
    match (parent.kind(), history) {
        (ParentAuthorityKind::Ordinary(parent), None) => request
            .at(basis)
            .demand(demand.clone())
            .controls(controls)
            .start_dependent::<Program, ParentDemand, Connection>(
                phase,
                application,
                parent,
                basis,
                minimum,
            )
            .map_err(WorthQueryRequiredOutputPreparationDenial::Demand),
        (ParentAuthorityKind::History(parent), Some(history)) => {
            let owned_demand = demand.clone();
            let access = worth_query_execution::publication_boundary::program_publication_access();
            let admitted = if history.authority.uses_owned_model_graph() {
                let read = application.execute_owned_history_dependent_output_query::<ParentDemand, Connection, Source<Schema, ChildDemand<Schema, Connection>>>(
                    phase, &access, parent, history.authority, &basis.retained, &minimum.retained,
                    request.principal, request.scope, demand.source_intent(),
                ).map_err(history_query_denial)?;
                application.admit_history_program_dependent_output::<ParentDemand, Connection>(
                    phase, &access, parent, read, controls.resolve(application.runtime().output_demand_resource_profile()),
                ).map_err(|cause| WorthQueryRequiredOutputPreparationDenial::Demand(
                    crate::application_entry::WorthQueryApplicationOutputDemandDenial::History(cause),
                ))?
            } else {
                request.query(demand.source_intent()).with_admitted_query(phase,
                |_, binding, external, control| {
                    let access = worth_query_execution::publication_boundary::program_publication_access();
                    let read = application.execute_history_dependent_output_query::<ParentDemand, Connection, Source<Schema, ChildDemand<Schema, Connection>>>(
                        phase, &access, parent, history.authority, &basis.retained, &minimum.retained,
                        binding, external, control,
                    ).map_err(crate::application_entry::WorthQueryApplicationRequestQueryDenial::HistoryOutput)?;
                    Ok(application.admit_history_program_dependent_output::<ParentDemand, Connection>(
                        phase, &access, parent, read,
                        controls.resolve(application.runtime().output_demand_resource_profile()),
                    ))
                },
            ).map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?
                .map_err(|cause| WorthQueryRequiredOutputPreparationDenial::Demand(
                    crate::application_entry::WorthQueryApplicationOutputDemandDenial::History(cause),
                ))?
            };
            Ok(WorthQueryApplicationProgramDemandHandle::new_history(
                admitted,
                owned_demand,
                std::sync::Arc::clone(&basis.retained),
            ))
        }
        (ParentAuthorityKind::Verified(parent), Some(history)) => {
            let owned_demand = demand.clone();
            let access = worth_query_execution::publication_boundary::program_publication_access();
            let read = application.execute_verified_history_dependent_output_query::<ParentDemand, Connection, Source<Schema, ChildDemand<Schema, Connection>>>(
                phase, &access, parent, history.authority, &basis.retained, &minimum.retained,
                request.principal, request.scope, demand.source_intent(),
            ).map_err(history_query_denial)?;
            let admitted =
                application
                    .admit_verified_history_program_dependent_output::<ParentDemand, Connection>(
                        phase,
                        &access,
                        parent,
                        read,
                        controls.resolve(application.runtime().output_demand_resource_profile()),
                    )
                    .map_err(|cause| {
                        WorthQueryRequiredOutputPreparationDenial::Demand(
                crate::application_entry::WorthQueryApplicationOutputDemandDenial::History(cause),
            )
                    })?;
            Ok(WorthQueryApplicationProgramDemandHandle::new_history(
                admitted,
                owned_demand,
                std::sync::Arc::clone(&basis.retained),
            ))
        }
        _ => Err(history_mismatch()),
    }
}

pub(super) fn verify_child<Schema, Program, ParentDemand, Connection>(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    parent: &ProgramOutputParentAuthority<Schema, Program, ParentDemand>,
    basis: &crate::application_entry::WorthQueryApplicationReadObservation,
    minimum: &crate::application_entry::WorthQueryApplicationReadObservation,
    demand: &ChildDemand<Schema, Connection>,
    history: Option<ProgramOutputHistoryFrame<'_, '_, '_>>,
) -> Result<Option<worth_query_execution::publication_boundary::document_history::WorthQueryVerifiedHistoryParent<Schema, Program, ChildDemand<Schema, Connection>>>, WorthQueryRequiredOutputPreparationDenial>
where
    Schema: ApplicationSchema + 'static, Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema>, Connection: ApplicationConnectionShape<Schema>,
    Binding<Schema, Connection>: WorthQueryApplicationDependentOutputConnection<Schema, RootDemand = ParentDemand>,
    <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = Source<Schema, ChildDemand<Schema, Connection>>>,
    <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<Schema, <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    SourceValue<Schema, ChildDemand<Schema, Connection>>: WorthQueryApplicationProjection<Schema, SourceQuery<Schema, ChildDemand<Schema, Connection>>>,
{
    let Some(history) = history.filter(|frame| frame.authority.uses_owned_model_graph()) else {
        return Ok(None);
    };
    let access = worth_query_execution::publication_boundary::program_publication_access();
    match parent.kind() {
        ParentAuthorityKind::History(parent) => application
            .verify_owned_history_dependent_output::<ParentDemand, Connection>(
                phase,
                &access,
                parent,
                history.authority,
                &basis.retained,
                &minimum.retained,
                request.principal,
                request.scope,
                demand,
            ),
        ParentAuthorityKind::Verified(parent) => application
            .verify_verified_history_dependent_output::<ParentDemand, Connection>(
                phase,
                &access,
                parent,
                history.authority,
                &basis.retained,
                &minimum.retained,
                request.principal,
                request.scope,
                demand,
            ),
        _ => return Err(history_mismatch()),
    }
    .map_err(history_query_denial)
}
