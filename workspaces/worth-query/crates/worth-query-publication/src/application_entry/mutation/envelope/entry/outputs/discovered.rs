//! The existing candidate-time recipe may yield zero or real required roots.
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationDiscoveredOutputRoot, ApplicationOutputGraphShape,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
use worth_query_execution::publication_boundary::synchronous_outputs::SynchronousDiscoveredOutputRoot;

impl<S, B, C, E> EditStageOutputs<S, B> for ApplicationDiscoveredOutputGraph<C, E>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: Clone + Send + Sync,
    B::ScopeBinding: ApplicationMutationScopeResolution<S, B::PrincipalIdentity>,
    C: ApplicationConnectionShape<S>,
    C::Binding: WorthQueryApplicationDiscoveredOutputConnection<S, Source = B>,
    Self: ApplicationOutputGraphShape<S, RootConnection = C, Dependents = E>
        + ApplicationDiscoveredOutputRoot
        + SynchronousDiscoveredOutputRoot<S>,
{
    fn require<P: ApplicationProgramDefinition<S>>(
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
    ) -> Result<(), Failure> {
        crate::application_entry::mutation::performed_source::require_program_output_root::<
            S,
            P,
            Self,
        >(program.runtime(), program)?;
        program
            .require_selected_discovered_envelope_stage::<Self, B>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                owner,
            )
            .map_err(|_| {
                Failure::Mutation(
                    WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired,
                )
            })
    }
    fn dependencies(
        stage: CapturedAuthoredEditStage<S>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        input: &ApplicationEncodedInput<B::InputBinding>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
    ) -> Result<CapturedAuthoredEditStage<S>, Failure> {
        let recipe =
            C::Binding::discovery_from_source(input.input()).map_err(Failure::Connection)?;
        <Self as SynchronousDiscoveredOutputRoot<S>>::dependencies(
            stage,
            application,
            Arc::new(recipe),
            owner.connection_order(),
        )
        .map_err(|cause| {
            Failure::Mutation(WorthQueryApplicationRequestMutationDenial::Handler(
                MutationHandlerExecutionDenial::Handler(HandlerExecutionDenial::new(cause)),
            ))
        })
    }
    fn capture(
        program: CandidateProgram<S, B>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        input: &ApplicationEncodedInput<B::InputBinding>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
    ) -> Result<CandidateProgram<S, B>, Failure> {
        let recipe =
            C::Binding::discovery_from_source(input.input()).map_err(Failure::Connection)?;
        Ok(<Self as SynchronousDiscoveredOutputRoot<S>>::attach(
            program,
            &worth_query_execution::publication_boundary::program_publication_access(),
            execution,
            Arc::new(recipe),
            owner.connection_order(),
        ))
    }
    fn commit<P: ApplicationProgramDefinition<S>>(
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        phase: &WorthQueryAdvancementPhase<'_>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
        program: CandidateProgram<S, B>,
        identities: &ApplicationMutationIdentities<'_, S, B>,
        source: Option<Arc<WorthQueryPreparedCorrectionSource>>,
        binding: WorthQueryApplicationIdempotencyBinding,
        unseen: &UnseenAuthoredEnvelope<'_>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome {
        application.compare_and_commit_selected_discovered_envelope::<Self, B>(
            phase,
            &worth_query_execution::publication_boundary::program_publication_access(),
            owner,
            program,
            identities,
            source,
            binding,
            unseen,
            allocation,
        )
    }
    fn inverse<K>(
        target: &WorthQuerySelectedProductOperation<'_, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        scope: &WorthQueryRequestScope,
        source: &WorthQuerySelectedCorrectionSource<'_, S>,
        key: &WorthQueryCorrectionCommandKey,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> SelectedSource<S, B>
    where
        K: ApplicationCorrectionBinding<S>,
        K::InverseIntent: ApplicationMutationIntent<S, Binding = B>,
    {
        target.derive_discovered_output_correction::<K, Self>(
            &worth_query_execution::publication_boundary::program_publication_access(),
            phase,
            principal,
            scope,
            source,
            key,
            maximum_history,
            maximum_results,
        )
    }
    fn reapplication<K>(
        target: &WorthQuerySelectedProductOperation<'_, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        scope: &WorthQueryRequestScope,
        source: &WorthQuerySelectedCorrectionAlternative<'_, S>,
        key: &WorthQueryCorrectionCommandKey,
        maximum_history: Option<NonZeroUsize>,
        maximum_results: NonZeroUsize,
    ) -> SelectedSource<S, B>
    where
        K: ApplicationCorrectionBinding<S, Source = B>,
        B::Input: ApplicationMutationInputScope<S, B>,
    {
        target.derive_discovered_output_reapplication::<K, Self>(
            &worth_query_execution::publication_boundary::program_publication_access(),
            phase,
            principal,
            scope,
            source,
            key,
            maximum_history,
            maximum_results,
        )
    }
}
