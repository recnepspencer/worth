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
        let staged = self
            .stage()
            .map_err(WorthQueryApplicationProgramMigrationPreparationDenial::Request)?;
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
        self.execute_report().into_outcome()
    }

    /// Executes the same ordinary mutation and preserves this attempt's decision work.
    pub fn execute_report(
        self,
    ) -> super::WorthQueryApplicationMutationAttemptReport<
        Result<
            WorthQueryApplicationMutationOutcome<
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
            >,
            WorthQueryApplicationRequestMutationDenial,
        >,
    > {
        if let Err(denial) = self.require_workflow_transition() {
            return super::WorthQueryApplicationMutationAttemptReport::new(Err(denial),
                worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted);
        }
        if self
            .request
            .application
            .requires_application_program::<Intent::Binding>()
        {
            return super::WorthQueryApplicationMutationAttemptReport::new(
                Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired),
                worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted);
        }
        self.execute_with_preparation_and_commit_report(
            super::authorization::prepare,
            |application, program, binding| {
                application.compare_and_commit_application(program, binding.idempotency())
            },
        )
    }

    pub(super) fn execute_with_preparation_and_commit(
        self,
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
        self.execute_with_preparation_and_commit_report(prepare, commit)
            .into_outcome()
    }

    pub(super) fn execute_with_preparation_and_commit_report(
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
    ) -> super::WorthQueryApplicationMutationAttemptReport<
        Result<
            WorthQueryApplicationMutationOutcome<
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
            >,
            WorthQueryApplicationRequestMutationDenial,
        >,
    > {
        let mut decision_work =
            worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted;
        let outcome = (|| {
            let application = self.request.application;
            let candidate = match self.prepare_candidate(prepare, &mut decision_work)? {
                super::preparation::CandidatePreparation::Prepared(candidate) => candidate,
                super::preparation::CandidatePreparation::Settled(outcome) => return Ok(outcome),
            };
            let super::preparation::PreparedCandidate {
                program,
                result,
                identities,
                extension,
                decision_work: _,
            } = candidate;
            let commit_binding = WorthQueryMutationCommitBinding::new(&identities, extension);
            Ok(
                match commit(application, program, &commit_binding).landed() {
                    Ok((receipt, false)) => {
                        WorthQueryApplicationMutationOutcome::Committed { receipt, result }
                    }
                    Ok((receipt, true)) => {
                        WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt)
                    }
                    Err(uncommitted) => WorthQueryApplicationMutationOutcome::Commit(uncommitted),
                },
            )
        })();
        super::WorthQueryApplicationMutationAttemptReport::new(outcome, decision_work)
    }

    pub(super) fn require_workflow_transition(
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

    pub(super) fn resolve_idempotency(
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
        let resolution = match self
            .request
            .application
            .resolve_admitted_application_idempotency(admission, idempotency)
        {
            Ok(read) => read.into_resolution(),
            Err(denial) => {
                return match denial.historical_commit() {
                    Some(observation) => Ok(Some(
                        WorthQueryApplicationMutationOutcome::PreviouslyCommitted(observation),
                    )),
                    None => Err(WorthQueryApplicationRequestMutationDenial::Idempotency(
                        denial,
                    )),
                };
            }
        };
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
