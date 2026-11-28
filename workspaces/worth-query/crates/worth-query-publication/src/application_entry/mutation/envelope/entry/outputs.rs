//! Declaration-owned output choices share one nominal stage executor.
use super::*;
use crate::application_entry::mutation::preparation::CandidateProgram;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationIdentities,
};
use worth_query_declaration::facade::application_program::{
    ApplicationDiscoveredOutputGraph, ApplicationNoOutputGraph, ApplicationOutputGraph,
    ApplicationProgramOutputShape,
};
use worth_query_execution::facade::application_installation::WorthQuerySelectedProgramOwner;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitOutcome,
};

mod sealed {
    pub trait Outputs {}
    impl<C, E> Outputs for super::ApplicationOutputGraph<C, E> {}
    impl<C, E> Outputs for super::ApplicationDiscoveredOutputGraph<C, E> {}
    impl Outputs for super::ApplicationNoOutputGraph {}
}
mod authored;
mod discovered;
mod required;
type Failure = crate::application_entry::WorthQueryPerformedMutationExecutionDenial;
pub trait EditStageOutputs<S, B>: sealed::Outputs + ApplicationProgramOutputShape<S>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: Clone + Send + Sync,
    B::ScopeBinding: ApplicationMutationScopeResolution<S, B::PrincipalIdentity>,
{
    fn require<P: ApplicationProgramDefinition<S>>(
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
    ) -> Result<(), Failure>;
    fn dependencies(
        stage: CapturedAuthoredEditStage<S>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        input: &ApplicationEncodedInput<B::InputBinding>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
    ) -> Result<CapturedAuthoredEditStage<S>, Failure>;
    fn capture(
        program: CandidateProgram<S, B>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        input: &ApplicationEncodedInput<B::InputBinding>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
    ) -> Result<CandidateProgram<S, B>, Failure>;
    fn commit<P: ApplicationProgramDefinition<S>>(
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        phase: &WorthQueryAdvancementPhase<'_>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
        program: CandidateProgram<S, B>,
        identities: &ApplicationMutationIdentities<'_, S, B>,
        binding: WorthQueryApplicationIdempotencyBinding,
        unseen: &UnseenAuthoredEnvelope<'_>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome;
}
