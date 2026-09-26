use worth_query_declaration::facade::application_operation::{
    ApplicationCapabilityMutationBinding, ApplicationMutationBinding, ApplicationMutationIntent,
    ApplicationMutationScopeBinding, ApplicationMutationScopeResolution,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramOwner;
use worth_query_execution::facade::primary_graph::{
    HandlerResult, MutationHandlerExecutionDenial, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolution,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

#[derive(Debug)]
pub enum WorthQueryApplicationProgramMigrationPreparationDenial {
    Request(WorthQueryApplicationRequestMutationDenial),
    Migration(
        worth_query_execution::facade::primary_graph::WorthQueryProgramMigrationPreparationDenial,
    ),
}

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
        let prepared = super::authorization::prepare(&mut self)
            .map_err(WorthQueryApplicationProgramMigrationPreparationDenial::Request)?;
        let completed = match self
            .request
            .application
            .execute_mutation_handler::<Intent::Binding>(
                self.request.intent.input(),
                self.key,
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
            |application, program, idempotency| {
                application.compare_and_commit_application(program, idempotency)
            },
        )
    }

    /// Executes one action through the exact installed program that owns it.
    pub fn execute_in_program<Owner>(
        self,
        application: &'application Owner,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    >
    where
        Owner: WorthQueryProgramOwner<Schema>,
    {
        if !std::ptr::eq(application.runtime(), self.request.application) {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch);
        }
        if !application.contains_action::<Intent::Binding>() {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        self.execute_with_preparation_and_commit(
            super::authorization::prepare,
            |_, program, idempotency| {
                application
                    .compare_and_commit_program_action::<Intent::Binding>(program, idempotency)
            },
        )
    }

    /// Executes one capability-owned action through its exact installed program.
    pub fn execute_capability_in_program<Owner>(
        self,
        application: &'application Owner,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    >
    where
        Owner: WorthQueryProgramOwner<Schema>,
        Intent::Binding: ApplicationCapabilityMutationBinding<Schema>,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input:
            worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
                Schema,
                <Intent::Binding as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                Scope = <<Intent::Binding as ApplicationMutationBinding<
                    Schema,
                >>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
    {
        if !std::ptr::eq(application.runtime(), self.request.application) {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch);
        }
        if !application.contains_action::<Intent::Binding>() {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        self.execute_with_preparation_and_commit(
            super::authorization::prepare_capability,
            |_, program, idempotency| {
                application
                    .compare_and_commit_program_action::<Intent::Binding>(program, idempotency)
            },
        )
    }

    pub(super) fn execute_with_preparation_and_commit(
        mut self,
        prepare: impl FnOnce(
            &mut Self,
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
            WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationCommitOutcome,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        self.require_workflow_transition()?;
        let prepared = prepare(&mut self)?;
        let principal_identity = prepared.principal_identity;
        let admission = prepared.admission;
        let idempotency = prepared.idempotency;
        if let Some(outcome) = self.resolve_idempotency(&admission, idempotency)? {
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
                self.request.intent.input(),
                self.key,
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
            match commit(self.request.application, program, idempotency) {
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

    pub(super) fn execute_with_commit(
        self,
        retain_output_demand_observation: bool,
        commit: impl FnOnce(
            &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            WorthQueryApplicationEffectProgram<
                Schema,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Input,
                <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
            WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationCommitOutcome,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        self.require_workflow_transition()?;
        if !retain_output_demand_observation
            && self
                .request
                .application
                .requires_application_program::<Intent::Binding>()
        {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        self.execute_with_preparation_and_commit(super::authorization::prepare, commit)
    }

    fn require_workflow_transition(
        &self,
    ) -> Result<(), WorthQueryApplicationRequestMutationDenial> {
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
