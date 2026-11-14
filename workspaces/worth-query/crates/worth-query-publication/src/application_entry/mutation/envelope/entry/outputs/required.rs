//! A real required graph retains its typed demand and selected source owner.
use super::*;

impl<S, B, C, E> EditStageOutputs<S, B> for ApplicationOutputGraph<C, E>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S> + WorthQueryApplicationRequiredOutputSource<S, Connection<S, Self>>,
    B::Input: Clone + Send + Sync,
    B::ScopeBinding: ApplicationMutationScopeResolution<S, B::PrincipalIdentity>,
    Self: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, Self>: WorthQueryApplicationRequiredOutputConnection<S>,
    Demand<S, Self>: Clone + Send + Sync,
    <Self as ApplicationOutputGraphShape<S>>::Dependents: worth_query_execution::publication_boundary::synchronous_outputs::SynchronousOutputEdges<S, Demand<S, Self>>,
    <Source<S, Self> as ApplicationQueryBinding<S>>::Input: ApplicationQueryIntent<S, Binding = Source<S, Self>>,
    <Source<S, Self> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<S, <Source<S, Self> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
    <<Source<S, Self> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <Source<S, Self> as ApplicationQueryBinding<S>>::Query> + Clone,
{
    fn require<P: ApplicationProgramDefinition<S>>(program: &WorthQueryProgramApplicationRuntime<S, P>, _owner: &WorthQuerySelectedProgramOwner<'_, S>) -> Result<(), Failure> {
        // The existing commit owner checks the selected root/source. Capture
        // retains the existing initial-program typed-root requirement.
        crate::application_entry::mutation::performed_source::require_program_output_root::<S, P, Self>(program.runtime(), program)
    }
    fn dependencies(stage: CapturedAuthoredEditStage<S>, application: &WorthQueryPrimaryGraphApplicationRuntime<S>, input: &ApplicationEncodedInput<B::InputBinding>, owner: &WorthQuerySelectedProgramOwner<'_, S>) -> Result<CapturedAuthoredEditStage<S>, Failure> {
        let demand = <B as WorthQueryApplicationRequiredOutputSource<S, Connection<S, Self>>>::demand_from_source(input.input()).map_err(Failure::Connection)?;
        stage.with_required_output_dependencies::<B, Demand<S, Self>, <Self as ApplicationOutputGraphShape<S>>::Dependents>(application, demand, <Self as ApplicationOutputGraphShape<S>>::RootConnection::declaration(), owner.connection_order().clone()).map_err(|cause| Failure::Mutation(WorthQueryApplicationRequestMutationDenial::Handler(MutationHandlerExecutionDenial::Handler(HandlerExecutionDenial::new(cause)))))
    }
    fn capture(program: CandidateProgram<S, B>, execution: &worth_execution::ExecutionRequest<'_, '_>, input: &ApplicationEncodedInput<B::InputBinding>, owner: &WorthQuerySelectedProgramOwner<'_, S>) -> Result<CandidateProgram<S, B>, Failure> {
        let demand = <B as WorthQueryApplicationRequiredOutputSource<S, Connection<S, Self>>>::demand_from_source(input.input()).map_err(Failure::Connection)?;
        Ok(program.with_synchronous_required_outputs::<Connection<S, Self>, <Self as ApplicationOutputGraphShape<S>>::Dependents>(&worth_query_execution::publication_boundary::program_publication_access(), execution, demand, <Self as ApplicationOutputGraphShape<S>>::RootConnection::declaration(), owner.connection_order()))
    }
    fn commit<P: ApplicationProgramDefinition<S>>(application: &WorthQueryProgramApplicationRuntime<S, P>, phase: &WorthQueryAdvancementPhase<'_>, owner: &WorthQuerySelectedProgramOwner<'_, S>, program: CandidateProgram<S, B>, identities: &ApplicationMutationIdentities<'_, S, B>, source: Option<Arc<WorthQueryPreparedCorrectionSource>>, binding: WorthQueryApplicationIdempotencyBinding, unseen: &UnseenAuthoredEnvelope<'_>, allocation: ExecutionAllocationPolicy<'_, '_>) -> WorthQueryApplicationCommitOutcome {
        let access = worth_query_execution::publication_boundary::program_publication_access();
        match source {
            Some(source) => application.compare_and_commit_selected_corrective_envelope::<Self, B>(phase, &access, owner, program, identities, source, binding, unseen, allocation),
            None => application.compare_and_commit_selected_edit_envelope::<Self, B>(phase, &access, owner, program, binding, unseen, allocation),
        }
    }
    fn inverse<K>(target: &WorthQuerySelectedProductOperation<'_, S>, phase: &WorthQueryAdvancementPhase<'_>, principal: &WorthQueryAuthenticatedExternalPrincipal<S>, scope: &WorthQueryRequestScope, source: &WorthQuerySelectedCorrectionSource<'_, S>, key: &WorthQueryCorrectionCommandKey, maximum_history: Option<NonZeroUsize>, maximum_results: NonZeroUsize) -> SelectedSource<S, B>
    where K: ApplicationCorrectionBinding<S>, K::InverseIntent: ApplicationMutationIntent<S, Binding = B>,
    {
        target.derive_required_output_correction::<K, Self>(&worth_query_execution::publication_boundary::program_publication_access(), phase, principal, scope, source, key, maximum_history, maximum_results)
    }
    fn reapplication<K>(target: &WorthQuerySelectedProductOperation<'_, S>, phase: &WorthQueryAdvancementPhase<'_>, principal: &WorthQueryAuthenticatedExternalPrincipal<S>, scope: &WorthQueryRequestScope, source: &WorthQuerySelectedCorrectionAlternative<'_, S>, key: &WorthQueryCorrectionCommandKey, maximum_history: Option<NonZeroUsize>, maximum_results: NonZeroUsize) -> SelectedSource<S, B>
    where K: ApplicationCorrectionBinding<S, Source = B>, B::Input: ApplicationMutationInputScope<S, B>,
    {
        target.derive_required_output_reapplication::<K, Self>(&worth_query_execution::publication_boundary::program_publication_access(), phase, principal, scope, source, key, maximum_history, maximum_results)
    }
}
