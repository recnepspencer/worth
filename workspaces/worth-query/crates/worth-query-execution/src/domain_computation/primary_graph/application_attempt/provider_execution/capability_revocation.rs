use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
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
        let program = match self.validate_commit_program(program) {
            Ok(program) => program,
            Err(outcome) => return outcome,
        };
        let request = program.request_scope().clone();
        let mut candidate = Some(program);
        self.with_application_advancement(&request, |active_phase| {
            let execution = active_phase
                .execution_request_for(&self.runtime.product_runtime)
                .map_err(
                    crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from,
                )?;
            if let Some(denial) = self.direct_operation_commit_denial::<Operation>() {
                return Ok(WorthQueryApplicationCommitOutcome::Denied(denial));
            }
            Ok(
                match candidate
                    .take()
                    .expect("candidate admitted once")
                    .in_advancement(&active_phase)
                {
                    Ok(entry) => entry.compare_and_commit_capability_revocation_for_program(
                        &execution,
                        idempotency,
                    ),
                    Err(outcome) => outcome,
                },
            )
        })
        .and_then(std::convert::identity)
        .unwrap_or_else(|denial| denial.into_commit_outcome())
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_capability_revocation_for_program<
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryCapabilityRevocationProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => return outcome,
        };
        entry.compare_and_commit_capability_revocation_for_program(execution, idempotency)
    }
}
