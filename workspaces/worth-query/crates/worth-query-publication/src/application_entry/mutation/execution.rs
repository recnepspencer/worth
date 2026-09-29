use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationIntent,
    ApplicationMutationScopeBinding, ApplicationMutationScopeResolution,
};
use worth_query_execution::facade::primary_graph::{
    HandlerResult, MutationHandlerExecutionDenial, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolution,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    commit_binding::WorthQueryMutationCommitBinding, staged::WorthQueryStagedMutation,
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

/// Why a program migration request was refused before its handler ran.
#[derive(Debug)]
pub enum WorthQueryApplicationProgramMigrationPreparationDenial {
    Request(WorthQueryApplicationRequestMutationDenial),
    Migration(
        worth_query_execution::facade::primary_graph::WorthQueryProgramMigrationPreparationDenial,
    ),
}

/// What `prepare_program_migration` produced. The handler runs as a candidate for atomic
/// branch adoption; it commits nothing and registers no idempotency record.
pub enum WorthQueryApplicationProgramMigrationPreparationOutcome<DomainDenial> {
    Prepared(worth_query_execution::facade::primary_graph::WorthQueryPreparedProgramMigration),
    DomainDenied(DomainDenial),
    Cancelled,
    DeadlineExceeded,
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
    /// Executes this installed mutation handler as a candidate for the
    /// target program's atomic branch adoption. No operation is committed and
    /// no operation idempotency or outbox record is registered here.
    pub fn prepare_program_migration(
        mut self,
        target: &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    ) -> Result<
        WorthQueryApplicationProgramMigrationPreparationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
        >,
        WorthQueryApplicationProgramMigrationPreparationDenial,
    > {
        if <Intent::Binding as ApplicationMutationBinding<Schema>>::REQUIRES_WORKFLOW_AUTHORITY {
            return Err(
                WorthQueryApplicationProgramMigrationPreparationDenial::Request(
                    WorthQueryApplicationRequestMutationDenial::RequiresWorkflowTransition,
                ),
            );
        }
        self.require_workflow_transition()
            .map_err(WorthQueryApplicationProgramMigrationPreparationDenial::Request)?;
        let admitted = self
            .request
            .application
            .admit_program_migration::<Intent::Binding>(target)
            .map_err(WorthQueryApplicationProgramMigrationPreparationDenial::Migration)?;
        let staged = self.stage();
        let identities = self
            .identities()
            .map_err(WorthQueryApplicationProgramMigrationPreparationDenial::Request)?;
        let prepared = super::authorization::prepare(&self, &identities, staged)
            .map_err(WorthQueryApplicationProgramMigrationPreparationDenial::Request)?;
        let completed = match self
            .request
            .application
            .execute_mutation_handler::<Intent::Binding>(
                &identities,
                &prepared.principal_identity,
                prepared.admission,
            )
            .map_err(|denial| {
                WorthQueryApplicationProgramMigrationPreparationDenial::Request(
                    WorthQueryApplicationRequestMutationDenial::Handler(denial),
                )
            })? {
            HandlerResult::Completed(completed) => completed,
            HandlerResult::DomainDenied(denial) => {
                return Ok(
                    WorthQueryApplicationProgramMigrationPreparationOutcome::DomainDenied(denial),
                );
            }
            HandlerResult::ExecutionDenied(denial) => {
                return Err(
                    WorthQueryApplicationProgramMigrationPreparationDenial::Request(
                        WorthQueryApplicationRequestMutationDenial::Handler(
                            MutationHandlerExecutionDenial::Handler(denial),
                        ),
                    ),
                );
            }
            HandlerResult::Cancelled => {
                return Ok(WorthQueryApplicationProgramMigrationPreparationOutcome::Cancelled);
            }
            HandlerResult::DeadlineExceeded => {
                return Ok(
                    WorthQueryApplicationProgramMigrationPreparationOutcome::DeadlineExceeded,
                );
            }
        };
        self.request
            .application
            .seal_program_migration(admitted, completed)
            .map(WorthQueryApplicationProgramMigrationPreparationOutcome::Prepared)
            .map_err(WorthQueryApplicationProgramMigrationPreparationDenial::Migration)
    }

    pub fn execute(
        self,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        self.require_workflow_transition()?;
        if self
            .request
            .application
            .requires_application_program::<Intent::Binding>()
        {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        self.execute_with_preparation_and_commit(
            super::authorization::prepare,
            |application, program, binding| {
                application.compare_and_commit_application(program, binding.idempotency())
            },
        )
    }

    pub(super) fn execute_with_preparation_and_commit(
        mut self,
        prepare: impl FnOnce(
            &Self,
            &ApplicationMutationIdentities<'_, Schema, Intent::Binding>,
            WorthQueryStagedMutation<Schema, Intent>,
        ) -> Result<
            super::authorization::PreparedMutation<Schema, Intent::Binding>,
            WorthQueryApplicationRequestMutationDenial,
        >,
        commit: impl FnOnce(
            &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            WorthQueryApplicationEffectProgram<
                Schema,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Input,
                <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
            &WorthQueryMutationCommitBinding<'_, '_, Schema, Intent::Binding>,
        ) -> WorthQueryApplicationCommitOutcome,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        self.require_workflow_transition()?;
        let staged = self.stage();
        let identities = self.identities()?;
        let prepared = prepare(&self, &identities, staged)?;
        let principal_identity = prepared.principal_identity;
        let admission = prepared.admission;
        let commit_binding = WorthQueryMutationCommitBinding::new(&identities, prepared.extension);
        if let Some(outcome) = self.resolve_idempotency(&admission, commit_binding.idempotency())? {
            return Ok(outcome);
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
        let completed = match self
            .request
            .application
            .execute_mutation_handler::<Intent::Binding>(
                &identities,
                &principal_identity,
                admission,
            )
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
        let (mut program, result) = completed.into_parts();
        if let Some(authority) = workflow_authority.as_ref() {
            program = program
                .bind_workflow_operation_authority(authority)
                .map_err(
                    WorthQueryApplicationRequestMutationDenial::WorkflowTransitionCurrentness,
                )?;
        }
        Ok(
            match commit(self.request.application, program, &commit_binding).landed() {
                Ok((receipt, false)) => {
                    WorthQueryApplicationMutationOutcome::Committed { receipt, result }
                }
                Ok((receipt, true)) => {
                    WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt)
                }
                Err(uncommitted) => WorthQueryApplicationMutationOutcome::Commit(uncommitted),
            },
        )
    }

    fn require_workflow_transition(
        &self,
    ) -> Result<(), WorthQueryApplicationRequestMutationDenial> {
        if <Intent::Binding as ApplicationMutationBinding<Schema>>::WORKFLOW_CONTROL {
            return Err(WorthQueryApplicationRequestMutationDenial::Handler(
                MutationHandlerExecutionDenial::WorkflowControl,
            ));
        }
        if <Intent::Binding as ApplicationMutationBinding<Schema>>::REQUIRES_WORKFLOW_AUTHORITY
            && (self.workflow_transition_identity.is_none() || self.workflow_authority.is_none())
        {
            return Err(WorthQueryApplicationRequestMutationDenial::RequiresWorkflowTransition);
        }
        Ok(())
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
