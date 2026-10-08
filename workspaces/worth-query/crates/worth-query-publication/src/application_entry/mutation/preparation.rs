//! Prepare one installed handler candidate without entering commit progression.
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
use worth_query_execution::facade::primary_graph::{
    HandlerResult, MutationHandlerExecutionDenial, WorthQueryApplicationEffectProgram,
};
use worth_query_execution::facade::runtime::ExecutionAllocationPolicy;
use worth_query_installation::facade::ApplicationSchema;

pub(super) type CandidateProgram<Schema, Binding> = WorthQueryApplicationEffectProgram<
    Schema, <Binding as ApplicationMutationBinding<Schema>>::Operation,
    <Binding as ApplicationMutationBinding<Schema>>::Input,
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
>;
pub(super) struct PreparedCandidate<
    'request,
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
> {
    pub(super) program: CandidateProgram<Schema, Binding>,
    pub(super) result: Binding::Result,
    pub(super) identities: ApplicationMutationIdentities<'request, Schema, Binding>,
    pub(super) extension: WorthQueryCommitExtension,
    pub(super) decision_work:
        worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork,
}
pub(super) enum CandidatePreparation<
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
    Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub(super) fn prepare_candidate<'request>(
        &'request mut self,
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
        let identities = self.identities()?;
        let prepared = prepare(self, &identities, staged)?;
        let principal_identity = prepared.principal_identity;
        let mut admission = prepared.admission;
        let commit_binding = WorthQueryMutationCommitBinding::new(&identities, prepared.extension);
        if let Some(outcome) = self.resolve_idempotency(&admission, commit_binding.idempotency())? {
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
                .validate_before_handler(
                    self.request.application,
                    admission.allowed_graph_contract().decision_fact_budget(),
                )
                .map_err(
                    WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness,
                )?;
        }
        let (handler_outcome, work) = self
            .request
            .application
            .execute_mutation_handler_report::<Intent::Binding>(
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
        if let Some(authority) = workflow_authority.as_ref() {
            program = program
                .bind_workflow_operation_authority(authority, allocation_policy)
                .map_err(
                    WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness,
                )?;
        }
        Ok(CandidatePreparation::Prepared(PreparedCandidate {
            program,
            result,
            identities,
            extension: prepared.extension,
            decision_work: work,
        }))
    }
}
