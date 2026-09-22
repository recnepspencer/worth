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
        Ok(
            application
                .compare_and_commit_application(self.change.program, self.change.idempotency),
        )
    }

    /// Commits an already admitted change only through the installed program
    /// that declared its exact operation type.
    pub fn commit_for_program<Program>(
        self,
        admitted: WorthQueryAdmittedProgramOperation<'_, Schema, Program, Operation>,
    ) -> Result<WorthQueryApplicationCommitOutcome, WorthQueryProductTransactionCommitError>
    where
        Program: worth_query_declaration::facade::application_program::ApplicationProgramDefinition<
            Schema,
        >,
    {
        let application = self.entry.application;
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
        let Some(presented) = admitted.runtime.presented_program() else {
            return Ok(WorthQueryApplicationCommitOutcome::Denied(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::application_program_required(),
            ));
        };
        Ok(
            application.compare_and_commit_application_for_program_action(
                &presented,
                self.change.program,
                self.change.idempotency,
            ),
        )
    }
}
