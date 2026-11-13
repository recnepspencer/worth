//! A synchronous envelope has no post-commit source continuation to register.
use super::*;
use crate::domain_computation::primary_graph::application_installation::WorthQuerySelectedProgramOwner;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryApplicationRequiredOutputSource,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
impl<S, P> WorthQueryProgramApplicationRuntime<S, P>
where
    S: worth_query_installation::facade::ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
{
    /// Publishes only the whole candidate: every authored stage and required
    /// root is part of this synchronous assembly. Unlike a standalone source,
    /// it leaves no deferred output preparation or carrier for a consumer.
    pub fn compare_and_commit_selected_edit_envelope<R, B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
        program: selected::SourceProgram<S, B>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        unseen: &crate::domain_computation::primary_graph::application_contribution::authored_edit::UnseenAuthoredEnvelope<'_>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        B: ApplicationMutationBinding<S>,
        B::Input: Clone + Send + Sync,
        R: ApplicationOutputGraphShape<S>,
        RootConnection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
        B: WorthQueryApplicationRequiredOutputSource<S, RootConnection<S, R>>,
    {
        let execution = match phase.checked_execution_request_for(self.runtime().product_runtime())
        {
            Ok(execution) => execution,
            Err(cause) => return cause.into_commit_outcome(),
        };
        if program.authored_stages.is_empty() || program.synchronous_outputs.is_empty() {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::required_output_preparation_missing(B::IDENTITY),
            );
        }
        let presented = match self.selected_source_owner::<B>(owner, TypeId::of::<R>()) {
            Ok(presented) => presented,
            Err(cause) => return WorthQueryApplicationCommitOutcome::Denied(cause),
        };
        self.runtime()
            .compare_and_commit_application_in_edit_envelope(
            &execution,
            phase,
            &presented,
            program,
            idempotency,
            unseen,
            crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
            allocation,
        )
    }
}
