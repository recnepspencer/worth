use super::*;
use crate::domain_computation::primary_graph::application_attempt::EntryCustodyScopedOps;
use crate::domain_computation::primary_graph::application_attempt::EntryScopedOps;

pub trait ElevationRequestOps<Schema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn compare_and_commit_elevation_request<Operation, Input, Scope>(
        &self,
        program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationRequestOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let program = match self.readmit_elevation_request_installation(program) {
            Ok(program) => program,
            Err(outcome) => return outcome,
        };
        let request = program.request_scope().clone();
        let mut candidate = Some(program);
        self.application_runtime()
            .with_application_advancement(&request, |active_phase| {
                let execution = active_phase
                    .execution_request_for(self.product_runtime())
                    .map_err(
                    crate::domain_computation::request_execution::WorthQueryAdvancementDenial::from,
                )?;
                if let Some(denial) = self.direct_operation_commit_denial::<Operation>() {
                    return Ok(WorthQueryElevationRequestOutcome::Denied(denial));
                }
                Ok(
                    match candidate
                        .take()
                        .expect("candidate admitted once")
                        .in_advancement(&active_phase)
                    {
                        Ok(entry) => entry.compare_and_commit_elevation_request_for_program(
                            &execution,
                            idempotency,
                        ),
                        Err(outcome) => outcome,
                    },
                )
            })
            .and_then(std::convert::identity)
            .unwrap_or_else(|denial| match candidate.take() {
                Some(candidate) => candidate.refuse_advancement(denial),
                None => WorthQueryElevationRequestOutcome::Indeterminate,
            })
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ElevationRequestOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

pub(in crate::domain_computation::primary_graph) trait ElevationRequestScopedOps<Schema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn compare_and_commit_elevation_request_for_program<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationRequestOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => return outcome,
        };
        entry.compare_and_commit_elevation_request_for_program(execution, idempotency)
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ElevationRequestScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}
