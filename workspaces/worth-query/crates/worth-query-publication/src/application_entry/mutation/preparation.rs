//! Prepare one installed handler candidate without entering commit progression.
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
        let principal_identity = prepared.principal_identity;
        let mut admission = prepared.admission;
        let commit_binding = WorthQueryMutationCommitBinding::new(&identities, prepared.extension);
        if let Some(outcome) =
            self.resolve_idempotency(phase, &admission, commit_binding.idempotency())?
        {
            return Ok(CandidatePreparation::Settled(outcome));
        }
        if let Some(pending) = prepared.pending_source {
            pending
                .consume_into(self.request.application, &mut admission, allocation_policy)
                .map_err(WorthQueryApplicationRequestMutationDenial::SourceExpectation)?;
        }
        let workflow_authority = self
            .workflow_authority
            .as_ref()
            .and_then(|slot| slot.take());
        if <Intent::Binding as ApplicationMutationBinding<Schema>>::REQUIRES_WORKFLOW_AUTHORITY {
            workflow_authority
                .as_ref()
                .ok_or(WorthQueryApplicationRequestMutationDenial::WorkflowAuthoritySpent)?
                .validate_before_handler(phase, self.request.application, self.request.scope)
                .map_err(
                    WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness,
                )?;
        }
        let (handler_outcome, work) = self
            .request
            .application
            .execute_mutation_handler_report::<Intent::Binding>(
                phase,
                &identities,
                &principal_identity,
                admission,
                allocation_policy,
            )
            .into_parts();
        *decision_work = work;
        let completed =
            match handler_outcome.map_err(WorthQueryApplicationRequestMutationDenial::Handler)? {
                HandlerResult::Completed(completed) => completed,
                HandlerResult::DomainDenied(denial) => {
                    return Ok(CandidatePreparation::Settled(
                        WorthQueryApplicationMutationOutcome::DomainDenied(denial),
                    ));
                }
                HandlerResult::ExecutionDenied(denial) => {
                    return Err(WorthQueryApplicationRequestMutationDenial::Handler(
                        MutationHandlerExecutionDenial::Handler(denial),
                    ));
                }
                HandlerResult::Cancelled => {
                    return Ok(CandidatePreparation::Settled(
                        WorthQueryApplicationMutationOutcome::Cancelled,
                    ));
                }
                HandlerResult::DeadlineExceeded => {
                    return Ok(CandidatePreparation::Settled(
                        WorthQueryApplicationMutationOutcome::DeadlineExceeded,
                    ));
                }
            };
        let (mut program, result) = completed.into_parts();
        let publication_access =
            worth_query_execution::publication_boundary::program_publication_access();
        if program.requires_original_source_capture(&publication_access) {
            program = program
                .retain_original_mutation_input::<Intent::Binding>(
                    &publication_access,
                    phase,
                    self.request.application,
                    identities.encoded_input(),
                )
                .map_err(|cause| {
                    WorthQueryApplicationRequestMutationDenial::Handler(
                        MutationHandlerExecutionDenial::Attempt(cause),
                    )
                })?;
        }
        if self.workflow_birth_identity.is_some() {
            let authority = self
                .workflow_birth_authority
                .as_ref()
                .and_then(|slot| slot.take())
                .ok_or(WorthQueryApplicationRequestMutationDenial::WorkflowAuthoritySpent)?;
            program = program
                .bind_workflow_birth_authority(phase, authority, allocation_policy)
                .map_err(
                    WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness,
                )?;
        }
        if self.workflow_succession_identity.is_some() {
            let authority = self
                .workflow_succession_authority
                .as_ref()
                .and_then(|slot| slot.take())
                .ok_or(WorthQueryApplicationRequestMutationDenial::WorkflowAuthoritySpent)?;
            program = program
                .bind_workflow_succession_authority(phase, authority, allocation_policy)
                .map_err(
                    WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness,
                )?;
        }
        if let Some(authority) = workflow_authority.as_ref() {
            program = program
                .bind_workflow_operation_authority(phase, authority, allocation_policy)
                .map_err(
                    WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness,
                )?;
        }
        let program = worth_query_execution::publication_boundary::signed_creation::WorthQuerySignedCreationAdapter::prepare::<Schema, Intent::Binding>(
            &worth_query_execution::publication_boundary::program_publication_access(), phase, self.request.application, program, &identities,
        ).map_err(WorthQueryApplicationRequestMutationDenial::SignedCreation)?;
        Ok(CandidatePreparation::Prepared(PreparedCandidate {
            program,
            result,
            identities,
            principal: self.request.principal,
            extension: prepared.extension,
            decision_work: work,
        }))
    }
}
