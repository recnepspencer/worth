use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
    ApplicationMutationScopeResolution,
};
use worth_query_execution::facade::primary_graph::{
    HandlerResult, MutationHandlerExecutionDenial, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolution,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

impl<'application, 'principal, 'scope, 'key, Schema, Intent>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let prepared = super::authorization::prepare(&self)?;
        let admission = prepared.admission;
        let idempotency = prepared.idempotency;
        if let Some(outcome) = self.resolve_idempotency(&admission, idempotency)? {
            return Ok(outcome);
        }
        let completed = match self
            .request
            .application
            .execute_mutation_handler::<Intent::Binding>(&self.request.intent, self.key, admission)
            .map_err(WorthQueryApplicationRequestMutationDenial::Handler)?
        {
            HandlerResult::Completed(completed) => completed,
            HandlerResult::DomainDenied(denial) => {
                return Ok(WorthQueryApplicationMutationOutcome::DomainDenied(denial));
            }
            HandlerResult::ExecutionDenied(denial) => {
                return Err(WorthQueryApplicationRequestMutationDenial::Handler(
                    MutationHandlerExecutionDenial::Handler(denial),
                ));
            }
            HandlerResult::Cancelled => {
                return Ok(WorthQueryApplicationMutationOutcome::Cancelled);
            }
            HandlerResult::DeadlineExceeded => {
                return Ok(WorthQueryApplicationMutationOutcome::DeadlineExceeded);
            }
        };
        let (program, result) = completed.into_parts();
        Ok(
            match self
                .request
                .application
                .compare_and_commit_application(program, idempotency)
            {
                WorthQueryApplicationCommitOutcome::Committed(receipt) => {
                    WorthQueryApplicationMutationOutcome::Committed { receipt, result }
                }
                WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => {
                    WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt)
                }
                outcome => WorthQueryApplicationMutationOutcome::Commit(outcome),
            },
        )
    }

    fn resolve_idempotency(
        &self,
        admission: &WorthQueryAdmittedApplicationOperation<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Input,
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<
            WorthQueryApplicationMutationOutcome<
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
            >,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let resolution = self
            .request
            .application
            .resolve_admitted_application_idempotency(admission, idempotency)
            .map_err(WorthQueryApplicationRequestMutationDenial::Idempotency)?
            .into_resolution();
        Ok(match resolution {
            WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => Some(
                WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt),
            ),
            WorthQueryApplicationIdempotencyResolution::IntentDrift => {
                Some(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)
            }
            WorthQueryApplicationIdempotencyResolution::Unseen => None,
        })
    }
}
