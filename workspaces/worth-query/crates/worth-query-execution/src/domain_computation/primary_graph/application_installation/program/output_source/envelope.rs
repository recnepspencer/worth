//! A synchronous envelope has no post-commit source continuation to register.
use super::*;
use crate::domain_computation::primary_graph::application_installation::WorthQuerySelectedProgramOwner;
use crate::domain_computation::primary_graph::program_occurrence::WorthQueryPresentedProgram;
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
        self.commit_edit_envelope::<R, B>(
            phase,
            owner,
            program,
            idempotency,
            unseen,
            crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
            allocation,
        )
    }

    pub(super) fn commit_edit_envelope<R, B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
        program: selected::SourceProgram<S, B>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        unseen: &crate::domain_computation::primary_graph::application_contribution::authored_edit::UnseenAuthoredEnvelope<'_>,
        causality: crate::domain_computation::application_aftermath::ApplicationCommitCausality,
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
        self.commit_presented_edit_envelope::<B>(
            &execution,
            phase,
            &presented,
            program,
            idempotency,
            unseen,
            causality,
            allocation,
        )
    }

    /// Ordinary authored entries have no required source to prepare. They may
    /// still share this envelope with genuine required roots from other entries.
    pub fn compare_and_commit_selected_authored_envelope<B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        owner: &WorthQuerySelectedProgramOwner<'_, S>,
        program: selected::SourceProgram<S, B>,
        identities: &worth_query_declaration::facade::application_operation::ApplicationMutationIdentities<'_, S, B>,
        source: Option<
            std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryPreparedCorrectionSource,
            >,
        >,
        mut idempotency: WorthQueryApplicationIdempotencyBinding,
        unseen: &crate::domain_computation::primary_graph::application_contribution::authored_edit::UnseenAuthoredEnvelope<'_>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        B: ApplicationMutationBinding<S>,
        B::Input: Clone + Send + Sync,
    {
        use super::super::WorthQueryProgramOwner;
        if !std::ptr::eq(owner.owned_runtime(), self.runtime())
            || !self.contains_action::<B>()
            || !owner.contains_action::<B>()
            || self.owns_output_source(std::any::TypeId::of::<B>())
            || owner.owns_output_source(std::any::TypeId::of::<B>())
            || program.authored_stages.is_empty()
        {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        let causality = match source {
            Some(source) => {
                if let Err(cause) = source.admit_receiver(self.runtime().runtime.product_runtime.owner.owner_identity()) {
                    return WorthQueryApplicationCommitOutcome::Denied(cause);
                }
                if !source.is_action_binding::<B>() || !source.matches_input(identities.input_identity()) || !source.requires_root(None) {
                    return WorthQueryApplicationCommitOutcome::Denied(WorthQueryApplicationCommitDenial::correction_context_mismatch());
                }
                idempotency = source.intent().bind_idempotency(idempotency);
                crate::domain_computation::application_aftermath::ApplicationCommitCausality::RecordedCorrection(source)
            }
            None => crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
        };
        if !program.matches_workflow_authority_binding::<B>(&idempotency) {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::workflow_authority_required(),
            );
        }
        let execution = match phase.checked_execution_request_for(self.runtime().product_runtime())
        {
            Ok(execution) => execution,
            Err(cause) => return cause.into_commit_outcome(),
        };
        let presented = match self
            .runtime()
            .installed_program_support()
            .ok_or_else(WorthQueryApplicationCommitDenial::application_program_required)
            .and_then(|support| support.present_for_commit(owner.owned_revision()))
        {
            Ok(presented) => presented,
            Err(cause) => return WorthQueryApplicationCommitOutcome::Denied(cause),
        };
        self.commit_presented_edit_envelope::<B>(
            &execution,
            phase,
            &presented,
            program,
            idempotency,
            unseen,
            causality,
            allocation,
        )
    }

    fn commit_presented_edit_envelope<B>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        presented: &WorthQueryPresentedProgram<'_>,
        program: selected::SourceProgram<S, B>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        unseen: &crate::domain_computation::primary_graph::application_contribution::authored_edit::UnseenAuthoredEnvelope<'_>,
        causality: crate::domain_computation::application_aftermath::ApplicationCommitCausality,
        allocation: ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        B: ApplicationMutationBinding<S>,
        B::Input: Clone + Send + Sync,
    {
        self.runtime()
            .compare_and_commit_application_in_edit_envelope(
                execution,
                phase,
                presented,
                program,
                idempotency,
                unseen,
                causality,
                allocation,
            )
    }
}
