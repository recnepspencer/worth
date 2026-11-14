//! Conditional roots settle in the candidate, never in a post-commit carrier.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::authored_edit::UnseenAuthoredEnvelope,
    application_output_demand::PreparedOutputRootKind, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationIdempotencyBinding, WorthQueryPreparedCorrectionSource,
};
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationIdentities},
    application_program::ApplicationDiscoveredOutputRoot,
};

impl<S, P> WorthQueryProgramApplicationRuntime<S, P>
where
    S: worth_query_installation::facade::ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
{
    pub fn compare_and_commit_selected_discovered_envelope<R, B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        owner: &super::super::WorthQuerySelectedProgramOwner<'_, S>,
        program: selected::SourceProgram<S, B>,
        identities: &ApplicationMutationIdentities<'_, S, B>,
        source: Option<std::sync::Arc<WorthQueryPreparedCorrectionSource>>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        unseen: &UnseenAuthoredEnvelope<'_>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
        RootConnection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = B>,
        B: ApplicationMutationBinding<S>,
        B::Input: Clone + Send + Sync,
    {
        match source {
            Some(source) => self.commit_corrective_envelope::<R, B>(
                phase, owner, program, identities, source, idempotency, unseen,
                PreparedOutputRootKind::Discovered(TypeId::of::<R>()), allocation,
            ),
            None => self.commit_edit_envelope::<R, B>(
                phase, owner, program, idempotency, unseen,
                crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
                allocation,
            ),
        }
    }
}
