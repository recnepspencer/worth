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
            let execution = active_phase
                .execution_request_for(&self.product_runtime)
                .map_err(
                    crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from,
                )?;
            if let Some(denial) = self.direct_operation_commit_denial::<Operation>() {
                return Ok(WorthQueryApplicationCommitOutcome::Denied(denial));
            }
            Ok(self.compare_and_commit_capability_delegation_for_program(
                &execution,
                &active_phase,
                candidate.take().expect("candidate admitted once"),
                idempotency,
            ))
        })
        .and_then(std::convert::identity)
        .unwrap_or_else(|denial| denial.into_commit_outcome())
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_capability_delegation_for_program<
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryDelegationActivationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.compare_and_commit_application_inner(
            execution,
            phase,
            program.into_inner(),
            idempotency,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    }
}
