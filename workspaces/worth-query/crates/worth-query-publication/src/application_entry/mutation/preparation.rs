//! Prepare one installed handler candidate without entering commit progression.
mod completion;
mod handler_execution;
mod original_input;
use super::{
    commit_binding::{WorthQueryCommitExtension, WorthQueryMutationCommitBinding},
    staged::WorthQueryStagedMutation,
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationIntent,
    ApplicationMutationScopeBinding, ApplicationMutationScopeResolution,
};
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;
use worth_query_execution::facade::primary_graph::{
    HandlerResult, MutationHandlerExecutionDenial, WorthQueryApplicationEffectProgram,
};
use worth_query_execution::facade::runtime::ExecutionAllocationPolicy;
use worth_query_installation::facade::ApplicationSchema;

pub(in crate::application_entry) type CandidateProgram<Schema, Binding> = WorthQueryApplicationEffectProgram<
    Schema, <Binding as ApplicationMutationBinding<Schema>>::Operation,
    <Binding as ApplicationMutationBinding<Schema>>::Input,
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
>;
pub(in crate::application_entry) struct PreparedCandidate<
    'request,
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
> {
    pub(in crate::application_entry) program: CandidateProgram<Schema, Binding>,
    pub(in crate::application_entry) result: Binding::Result,
    pub(super) identities: ApplicationMutationIdentities<'request, Schema, Binding>,
    pub(super) principal: &'request worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) extension: WorthQueryCommitExtension,
    pub(super) decision_work:
        worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork,
}
pub(in crate::application_entry) enum CandidatePreparation<
    'request,
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
> {
    Prepared(PreparedCandidate<'request, Schema, Binding>),
    Settled(WorthQueryApplicationMutationOutcome<Binding::Denial, Binding::Result>),
}
impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub(super) fn prepare_candidate<'request>(
        &'request mut self,
        phase: &AdvancementPhase<'_>,
        prepare: impl FnOnce(
            &Self,
            &ApplicationMutationIdentities<'_, Schema, Intent::Binding>,
            WorthQueryStagedMutation<Schema, Intent>,
        ) -> Result<
            super::authorization::PreparedMutation<Schema, Intent::Binding>,
            WorthQueryApplicationRequestMutationDenial,
        >,
        decision_work: &mut worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        CandidatePreparation<'request, Schema, Intent::Binding>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        self.require_workflow_transition()?;
        let staged = self.stage()?;
        let captured_input = self.capture_original_input(phase)?;
        let identities = match captured_input.as_ref() {
            Some(input) => ApplicationMutationIdentities::with_encoded_input(self.key, input)
                .map_err(WorthQueryApplicationRequestMutationDenial::Identity)?,
            None => self.identities()?,
        };
        self.prepare_staged_candidate(
            phase,
            identities,
            staged,
            prepare,
            decision_work,
            allocation_policy,
        )
    }

    /// Uses the caller's existing owning identity carrier for the same candidate spine.
    pub(in crate::application_entry) fn prepare_owned_candidate<'request>(
        &'request mut self,
        phase: &AdvancementPhase<'_>,
        identities: ApplicationMutationIdentities<'request, Schema, Intent::Binding>,
        prepare: impl FnOnce(
            &Self,
            &ApplicationMutationIdentities<'_, Schema, Intent::Binding>,
            WorthQueryStagedMutation<Schema, Intent>,
        ) -> Result<
            super::authorization::PreparedMutation<Schema, Intent::Binding>,
            WorthQueryApplicationRequestMutationDenial,
        >,
        decision_work: &mut worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        CandidatePreparation<'request, Schema, Intent::Binding>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        self.require_workflow_transition()?;
        let staged = self.stage()?;
        let prepared = self.prepare_staged_candidate(
            phase,
            identities,
            staged,
            prepare,
            decision_work,
            allocation_policy,
        )?;
        match prepared {
            CandidatePreparation::Prepared(mut candidate) => {
                let access =
                    worth_query_execution::publication_boundary::program_publication_access();
                if !candidate.program.requires_original_input_retention(&access) {
                    candidate.program = candidate
                        .program
                        .retain_performed_correction_input::<Intent::Binding>(
                            &access,
                            phase,
                            self.request.application,
                            candidate.identities.encoded_input(),
                        )
                        .map_err(|cause| {
                            WorthQueryApplicationRequestMutationDenial::Handler(
                                MutationHandlerExecutionDenial::Attempt(cause),
                            )
                        })?;
                }
                Ok(CandidatePreparation::Prepared(candidate))
            }
            settled => Ok(settled),
        }
    }

    fn prepare_staged_candidate<'request>(
        &'request self,
        phase: &AdvancementPhase<'_>,
        identities: ApplicationMutationIdentities<'request, Schema, Intent::Binding>,
        staged: WorthQueryStagedMutation<Schema, Intent>,
        prepare: impl FnOnce(
            &Self,
            &ApplicationMutationIdentities<'_, Schema, Intent::Binding>,
            WorthQueryStagedMutation<Schema, Intent>,
        ) -> Result<
            super::authorization::PreparedMutation<Schema, Intent::Binding>,
            WorthQueryApplicationRequestMutationDenial,
        >,
        decision_work: &mut worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        CandidatePreparation<'request, Schema, Intent::Binding>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let prepared = prepare(self, &identities, staged)?;
        match self.execute_staged_handler(
            phase,
            identities,
            prepared,
            decision_work,
            allocation_policy,
        )? {
            handler_execution::HandlerPreparation::Executed(executed) => {
                self.finish_executed_candidate(phase, executed, allocation_policy)
            }
            handler_execution::HandlerPreparation::Settled(outcome) => {
                Ok(CandidatePreparation::Settled(outcome))
            }
        }
    }
}
