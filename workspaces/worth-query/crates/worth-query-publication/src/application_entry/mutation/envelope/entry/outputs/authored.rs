//! No graph means an ordinary installed action, never an omitted required source.
use super::*;
use worth_query_execution::facade::application_installation::WorthQueryProgramOwner;

impl<S, B> EditStageOutputs<S, B> for ApplicationNoOutputGraph
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: Clone + Send + Sync,
    B::ScopeBinding: ApplicationMutationScopeResolution<S, B::PrincipalIdentity>,
{
    fn require<P: ApplicationProgramDefinition<S>>(
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
    ) -> Result<(), Failure> {
        if !program.contains_action::<B>()
            || !owner.contains_action::<B>()
            || program.owns_output_source(std::any::TypeId::of::<B>())
            || owner.owns_output_source(std::any::TypeId::of::<B>())
        {
            return Err(Failure::Mutation(
                WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired,
            ));
        }
        Ok(())
    }
    fn dependencies(
        stage: CapturedAuthoredEditStage<S>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        _: &ApplicationEncodedInput<B::InputBinding>,
        _: &WorthQuerySelectedProgramOwner<'_, S>,
    ) -> Result<CapturedAuthoredEditStage<S>, Failure> {
        stage
            .with_declared_output_dependencies::<B>(application)
            .map_err(|cause| {
                Failure::Mutation(WorthQueryApplicationRequestMutationDenial::Handler(
                    MutationHandlerExecutionDenial::Handler(HandlerExecutionDenial::new(cause)),
                ))
            })
    }
    fn capture(
        program: CandidateProgram<S, B>,
        _: &worth_execution::ExecutionRequest<'_, '_>,
        _: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        _: &ApplicationEncodedInput<B::InputBinding>,
        _: &WorthQuerySelectedProgramOwner<'_, S>,
    ) -> Result<CandidateProgram<S, B>, Failure> {
        Ok(program)
    }
    fn commit<P: ApplicationProgramDefinition<S>>(
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        phase: &WorthQueryAdvancementPhase<'_>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
        program: CandidateProgram<S, B>,
        identities: &ApplicationMutationIdentities<'_, S, B>,
        binding: WorthQueryApplicationIdempotencyBinding,
        unseen: &UnseenAuthoredEnvelope<'_>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome {
        application.compare_and_commit_selected_authored_envelope::<B>(
            phase,
            &worth_query_execution::publication_boundary::program_publication_access(),
            owner,
            program,
            identities,
            None,
            binding,
            unseen,
            allocation,
        )
    }
}
