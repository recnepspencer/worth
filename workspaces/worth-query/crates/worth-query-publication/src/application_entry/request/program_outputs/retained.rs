//! Initial read trees admit at their one originally issued native basis.
use super::*;
impl<S: ApplicationSchema + 'static> WorthQueryApplicationRequest<'_, '_, '_, S> {
    pub(in crate::application_entry) fn start_program_outputs_at_in_advancement<P,R>(
        &self, phase: &AdvancementPhase<'_>, application: &WorthQueryProgramApplicationRuntime<S,P>, root_demand: RootDemand<S,R>, controls: crate::application_entry::WorthQueryOutputDemandControls, observation: &crate::application_entry::WorthQueryApplicationReadObservation,
    ) -> Result<crate::application_entry::WorthQueryApplicationProgramOutputHandle<S,P,R>, crate::application_entry::WorthQueryRequiredOutputPreparationDenial>
    where P: ApplicationProgramDefinition<S>, R: ApplicationOutputGraphShape<S> + worth_query_declaration::facade::application_program::ApplicationFixedOutputRoot,
        RootConnection<S,R>: WorthQueryApplicationRequiredOutputConnection<S>,
        R::Dependents: crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<S,P,RootDemand<S,R>>,
        RootDemand<S,R>: Clone,
        <RootSource<S,R> as ApplicationQueryBinding<S>>::Input: ApplicationQueryIntent<S,Binding=RootSource<S,R>>,
        <RootSource<S,R> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<S,<RootSource<S,R> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
        RootValue<S,R>: WorthQueryApplicationProjection<S,RootQuery<S,R>> + Clone,
    {
        if !std::ptr::eq(application.runtime(), self.application) {
            return Err(
                crate::application_entry::WorthQueryRequiredOutputPreparationDenial::ForeignProgram,
            );
        }
        application.validate_initial_program_source::<R>(phase, &worth_query_execution::publication_boundary::program_publication_access(), &observation.retained, self.branch).map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::DemandExecution)?;
        let (root, retained) = self
            .at(observation)
            .demand(root_demand.clone())
            .controls(controls)
            .start_for_program::<P, R>(phase, application)
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?;
        Ok(
            crate::application_entry::WorthQueryApplicationProgramOutputHandle::new_initial(
                crate::application_entry::WorthQueryApplicationReadObservation::new(retained),
                root,
                root_demand,
                controls,
            ),
        )
    }
}
