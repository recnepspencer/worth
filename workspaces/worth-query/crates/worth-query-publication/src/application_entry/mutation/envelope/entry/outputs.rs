//! The two declaration-owned output choices share one nominal stage executor.
use super::*;
use crate::application_entry::mutation::preparation::CandidateProgram;
use std::num::NonZeroUsize;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_aftermath::ApplicationCorrectionBinding;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationIdentities, ApplicationMutationInputScope,
};
use worth_query_declaration::facade::application_program::{
    ApplicationNoOutputGraph, ApplicationOutputGraph, ApplicationProgramOutputShape,
};
use worth_query_execution::facade::application_installation::WorthQuerySelectedProgramOwner;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryCorrectionDerivationDenial,
    WorthQuerySelectedCorrectionAlternative, WorthQuerySelectedCorrectionSource,
    WorthQuerySelectedProductOperation,
};
use worth_query_execution::publication_boundary::recorded_correction::{
    WorthQueryCorrectionCommandKey, WorthQueryPreparedCorrectionSource,
};

mod sealed {
    pub trait Outputs {}
    impl<C, E> Outputs for super::ApplicationOutputGraph<C, E> {}
    impl Outputs for super::ApplicationNoOutputGraph {}
}
mod authored;
mod required;
type Failure = crate::application_entry::WorthQueryPerformedMutationExecutionDenial;
type SelectedSource<S, B> = Result<
    (
        ApplicationEncodedInput<<B as ApplicationMutationBinding<S>>::InputBinding>,
        Arc<WorthQueryPreparedCorrectionSource>,
    ),
    WorthQueryCorrectionDerivationDenial,
>;

/// Execution mechanics for the existing required-graph and no-graph meanings.
/// Consumers cannot add an output policy or supply a program presentation.
#[doc(hidden)]
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
        input: &ApplicationEncodedInput<B::InputBinding>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
    ) -> Result<CandidateProgram<S, B>, Failure>;
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
    ) -> WorthQueryApplicationCommitOutcome;
    fn inverse<C>(
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
        C: ApplicationCorrectionBinding<S>,
        C::InverseIntent: ApplicationMutationIntent<S, Binding = B>;
    fn reapplication<C>(
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
        C: ApplicationCorrectionBinding<S, Source = B>,
        B::Input: ApplicationMutationInputScope<S, B>;
}
