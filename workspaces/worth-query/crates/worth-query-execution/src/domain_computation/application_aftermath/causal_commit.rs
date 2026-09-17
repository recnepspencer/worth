//! Typed entry from aftermath progression into the ordinary commit lane.

use worth_query_installation::facade::ApplicationSchema;

use super::{WorthQueryRedoProgressionHandoff, WorthQueryUndoProgressionHandoff};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Commit an admitted undo and its Query causal fact in the same ordinary
    /// Relational transaction. The handoff is the only public way to select
    /// undo causality.
    pub fn compare_and_commit_undo_application<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        handoff: &WorthQueryUndoProgressionHandoff,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if self.has_installed_application_program() {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        self.compare_and_commit_application_with_aftermath(
            program,
            idempotency,
            handoff.pending_causality(),
        )
    }

    /// Commit an admitted redo and its Query causal fact in the same ordinary
    /// Relational transaction. Relational also rechecks the bound branch head.
    pub fn compare_and_commit_redo_application<Operation, Input, Scope>(
        &self,
        program: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        handoff: &WorthQueryRedoProgressionHandoff,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if self.has_installed_application_program() {
            return WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::application_program_required(),
            );
        }
        self.compare_and_commit_application_with_aftermath(
            program,
            idempotency,
            handoff.pending_causality(),
        )
    }
}
