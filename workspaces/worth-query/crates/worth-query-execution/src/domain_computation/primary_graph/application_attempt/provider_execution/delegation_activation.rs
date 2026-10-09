use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
    WorthQueryDelegationActivationProgram,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn compare_and_commit_capability_delegation<Operation, Input, Scope>(
        &self,
        program: WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let request = program.request_scope().clone();
        let mut candidate = Some(program);
        self.with_application_advancement(&request, |active_phase| {
            if let Some(denial) = self.direct_operation_commit_denial::<Operation>() {
                return WorthQueryApplicationCommitOutcome::Denied(denial);
            }
            self.compare_and_commit_capability_delegation_for_program(
                &active_phase,
                candidate.take().expect("candidate admitted once"),
                idempotency,
            )
        })
        .unwrap_or_else(|denial| denial.into_commit_outcome())
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_capability_delegation_for_program<
        Operation,
        Input,
        Scope,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_inner(phase, program.into_inner(), idempotency)
    }
}
