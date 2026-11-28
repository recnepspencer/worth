//! Authorize and execute the handler before allocating completion-phase frames.
use super::*;
pub(super) struct ExecutedCandidate<
    'request,
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
> {
    pub(super) completed:
        worth_query_execution::facade::primary_graph::WorthQueryCompletedMutationCandidate<
            Schema,
            Binding,
        >,
    pub(super) identities: ApplicationMutationIdentities<'request, Schema, Binding>,
    pub(super) extension: WorthQueryCommitExtension,
    pub(super) work: worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork,
    pub(super) workflow_authority: Option<
        worth_query_execution::publication_boundary::workflow_advance::WorkflowOperationAuthority,
    >,
}
pub(super) enum HandlerPreparation<
    'request,
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
> {
    Executed(ExecutedCandidate<'request, Schema, Binding>),
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
    #[inline(never)]
    pub(super) fn execute_staged_handler<'request>(
        &'request self,
        phase: &AdvancementPhase<'_>,
        identities: ApplicationMutationIdentities<'request, Schema, Intent::Binding>,
        prepared: super::super::authorization::PreparedMutation<Schema, Intent::Binding>,
        decision_work: &mut worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        HandlerPreparation<'request, Schema, Intent::Binding>,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let principal_identity = prepared.principal_identity;
        let mut admission = prepared.admission;
        let commit_binding = WorthQueryMutationCommitBinding::new(&identities, prepared.extension);
        if let Some(outcome) =
            self.resolve_idempotency(phase, &admission, commit_binding.idempotency())?
        {
            return Ok(HandlerPreparation::Settled(outcome));
        }
        if let Some(pending) = prepared.pending_source {
            pending
                .consume_into(self.request.application, phase, &mut admission)
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
            .execute_mutation_handler_report_with_query_reads::<Intent::Binding>(
                phase,
                &identities,
                &principal_identity,
                self.request.principal,
                admission,
                allocation_policy,
            )
            .into_parts();
        *decision_work = work;
        let completed =
            match handler_outcome.map_err(WorthQueryApplicationRequestMutationDenial::Handler)? {
                HandlerResult::Completed(completed) => completed,
                HandlerResult::DomainDenied(denial) => {
                    return Ok(HandlerPreparation::Settled(
                        WorthQueryApplicationMutationOutcome::DomainDenied(denial),
                    ));
                }
                HandlerResult::ExecutionDenied(denial) => {
                    return Err(WorthQueryApplicationRequestMutationDenial::Handler(
                        MutationHandlerExecutionDenial::Handler(denial),
                    ));
                }
                HandlerResult::Cancelled => {
                    return Ok(HandlerPreparation::Settled(
                        WorthQueryApplicationMutationOutcome::Cancelled,
                    ));
                }
                HandlerResult::DeadlineExceeded => {
                    return Ok(HandlerPreparation::Settled(
                        WorthQueryApplicationMutationOutcome::DeadlineExceeded,
                    ));
                }
            };
        Ok(HandlerPreparation::Executed(ExecutedCandidate {
            completed,
            identities,
            extension: prepared.extension,
            work,
            workflow_authority,
        }))
    }
}
