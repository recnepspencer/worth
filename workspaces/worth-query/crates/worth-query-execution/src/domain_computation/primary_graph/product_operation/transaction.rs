use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryProductEntry;
use crate::domain_computation::primary_graph::application_installation::WorthQueryAdmittedProgramOperation;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding,
};

/// An admitted effect program paired with its stable idempotency meaning.
pub struct WorthQueryAdmittedChange<Schema, Operation, Input, Scope> {
    program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
    idempotency: WorthQueryApplicationIdempotencyBinding,
}

pub struct WorthQueryProductTransaction<'application, Schema> {
    entry: WorthQueryProductEntry<'application, Schema>,
}

pub struct WorthQueryAppliedProductTransaction<'application, Schema, Operation, Input, Scope> {
    entry: WorthQueryProductEntry<'application, Schema>,
    change: WorthQueryAdmittedChange<Schema, Operation, Input, Scope>,
}

#[derive(Debug)]
pub enum WorthQueryProductTransactionCommitError {
    ApplicationMismatch,
    BranchMismatch,
}

impl<Schema, Operation, Input, Scope> WorthQueryAdmittedChange<Schema, Operation, Input, Scope> {
    pub const fn new(
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Self {
        Self {
            program,
            idempotency,
        }
    }
}

impl<'application, Schema> WorthQueryProductEntry<'application, Schema> {
    pub fn transaction(self) -> WorthQueryProductTransaction<'application, Schema> {
        WorthQueryProductTransaction { entry: self }
    }
}

impl<'application, Schema> WorthQueryProductTransaction<'application, Schema> {
    pub fn apply<Operation, Input, Scope>(
        self,
        change: WorthQueryAdmittedChange<Schema, Operation, Input, Scope>,
    ) -> WorthQueryAppliedProductTransaction<'application, Schema, Operation, Input, Scope> {
        WorthQueryAppliedProductTransaction {
            entry: self.entry,
            change,
        }
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryAppliedProductTransaction<'_, Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn commit(
        self,
    ) -> Result<WorthQueryApplicationCommitOutcome, WorthQueryProductTransactionCommitError> {
        let request = self.change.program.request_scope().clone();
        let runtime = self.entry.application;
        runtime
            .with_application_advancement(&request, |phase| {
                let application = self.entry.application;
                if !self.change.program.belongs_to_application(
                    application.runtime.authority_identity(),
                    &application.installed_schema.binding_identity(),
                ) {
                    return Err(WorthQueryProductTransactionCommitError::ApplicationMismatch);
                }
                if self.change.program.product_branch() != self.entry.branch {
                    return Err(WorthQueryProductTransactionCommitError::BranchMismatch);
                }
                Ok(application.compare_and_commit_application_in_advancement(
                    &phase,
                    self.change.program,
                    self.change.idempotency,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                ))
            })
            .unwrap_or_else(|cause| Ok(cause.into_commit_outcome()))
    }

    /// Commits an already admitted change only through the installed program
    /// that declared its exact operation type.
    pub fn commit_for_program<Program>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admitted: WorthQueryAdmittedProgramOperation<'_, Schema, Program, Operation>,
    ) -> Result<WorthQueryApplicationCommitOutcome, WorthQueryProductTransactionCommitError>
    where
        Program: worth_query_declaration::facade::application_program::ApplicationProgramDefinition<
            Schema,
        >,
    {
        if let Err(stop) = execution.consult() {
            return Ok(WorthQueryApplicationCommitOutcome::Denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::provider_execution_denied(
                    crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::ProviderPlan,
                    crate::facade::primary_graph::relational_execution_kind(stop.into(), None),
                    "program commit request stopped",
                ),
            ));
        }
        let request = self.change.program.request_scope().clone();
        let runtime = self.entry.application;
        runtime.with_application_advancement(&request, |phase| {
        let application = self.entry.application;
        let execution = match phase.execution_request_for(application.product_runtime()) {
            Ok(execution) => execution,
            Err(cause) => return Ok(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(cause).into_commit_outcome()),
        };
        if !std::ptr::eq(application, admitted.runtime.runtime())
            || !self.change.program.belongs_to_application(
                application.runtime.authority_identity(),
                &application.installed_schema.binding_identity(),
            )
        {
            return Err(WorthQueryProductTransactionCommitError::ApplicationMismatch);
        }
        if self.change.program.product_branch() != self.entry.branch {
            return Err(WorthQueryProductTransactionCommitError::BranchMismatch);
        }
        // This erased transaction carries no mutation-binding type. If any
        // binding for the operation is workflow-guarded, the typed program
        // owner door must adjudicate that binding instead.
        if application.operation_requires_workflow_authority::<Operation>() {
            return Ok(WorthQueryApplicationCommitOutcome::Denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::workflow_authority_required(),
            ));
        }
        let Some(presented) = admitted.runtime.presented_program() else {
            return Ok(WorthQueryApplicationCommitOutcome::Denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::application_program_required(),
            ));
        };
        Ok(
            application.compare_and_commit_application_for_program_action(&execution, &phase,
                &presented,
                self.change.program,
                self.change.idempotency,
                crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            ),
        )

        }).unwrap_or_else(|cause| Ok(cause.into_commit_outcome()))
    }
}
