//! Admit a child from its settled parent output.
use super::*;


#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub(super) fn admit_child<Schema, Program, ParentDemand, Connection>(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    parent: &WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>,
    basis: &crate::application_entry::WorthQueryApplicationReadObservation,
    minimum: &crate::application_entry::WorthQueryApplicationReadObservation,
    demand: &ChildDemand<Schema, Connection>,
    controls: WorthQueryOutputDemandControls,
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
    request
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
        .map_err(WorthQueryRequiredOutputPreparationDenial::Demand)
}
