use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
    WorthQueryCapabilityRevocationProgram,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn compare_and_commit_capability_revocation<Operation, Input, Scope>(
        &self,
        program: WorthQueryCapabilityRevocationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        if let Some(denial) = self.direct_operation_commit_denial::<Operation>() {
            return WorthQueryApplicationCommitOutcome::Denied(denial);
        }
        self.compare_and_commit_capability_revocation_for_program(program, idempotency)
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_capability_revocation_for_program<
        Operation,
        Input,
        Scope,
    >(
        &self,
        program: WorthQueryCapabilityRevocationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_inner(program.into_inner(), idempotency)
    }
}
