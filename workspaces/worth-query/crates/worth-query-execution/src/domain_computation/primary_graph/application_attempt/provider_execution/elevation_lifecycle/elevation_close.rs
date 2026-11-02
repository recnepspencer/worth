use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn compare_and_commit_elevation_close<Operation, Input, Scope>(
        &self,
        program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationCloseOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let program = match self.readmit_elevation_close_installation(program) {
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
            Ok(
                match candidate
                    .take()
                    .expect("candidate admitted once")
                    .in_advancement(&active_phase)
                {
                    Ok(entry) => entry.compare_and_commit_elevation_close_for_program(
                        &execution,
                        idempotency,
                        self.direct_operation_commit_denial::<Operation>(),
                    ),
                    Err(outcome) => outcome,
                },
            )
        })
        .and_then(std::convert::identity)
        .unwrap_or_else(|denial| match candidate.take() {
            Some(candidate) => candidate.refuse_advancement(denial),
            None => WorthQueryElevationCloseOutcome::Indeterminate,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_elevation_close_for_program<
        Operation,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryElevationCloseProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        direct_denial: Option<WorthQueryApplicationCommitDenial>,
    ) -> WorthQueryElevationCloseOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => return outcome,
        };
        entry.compare_and_commit_elevation_close_for_program(execution, idempotency, direct_denial)
    }
}
