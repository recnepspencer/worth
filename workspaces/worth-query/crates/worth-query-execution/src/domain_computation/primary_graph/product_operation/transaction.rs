use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryProductEntry;
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
}
