use super::*;
use crate::domain_computation::execution_runtime::ApplicationAdvancementOps;
use crate::domain_computation::primary_graph::application_attempt::EntryCustodyScopedOps;
use crate::domain_computation::primary_graph::application_attempt::EntryScopedOps;

pub trait ElevationApprovalOps<Schema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn compare_and_commit_elevation_approval<Operation, Input, Scope>(
        &self,
        program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryElevationApprovalOutcome
    where
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let program = match self.readmit_elevation_approval_installation(program) {
            Ok(program) => program,
            Err(outcome) => return outcome,
        };
        let request = program.request_scope().clone();
        let mut candidate = Some(program);
        self.with_application_advancement(&request, |active_phase| {
            let execution = active_phase
                .execution_request_for(self.product_runtime())
                .map_err(
                    crate::domain_computation::request_execution::WorthQueryAdvancementDenial::from,
                )?;
            Ok(
                match candidate
                    .take()
                    .expect("candidate admitted once")
                    .in_advancement(&active_phase)
                {
                    Ok(entry) => entry.compare_and_commit_elevation_approval_for_program(
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
            None => WorthQueryElevationApprovalOutcome::Indeterminate,
        })
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ElevationApprovalOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

pub(in crate::domain_computation::primary_graph) trait ElevationApprovalScopedOps<Schema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn compare_and_commit_elevation_approval_for_program<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        program: WorthQueryElevationApprovalProgram<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        direct_denial: Option<WorthQueryApplicationCommitDenial>,
    ) -> WorthQueryElevationApprovalOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        let entry = match self.validate_commit_program_in_advancement(phase, program) {
            Ok(entry) => entry,
            Err(outcome) => return outcome,
        };
        entry.compare_and_commit_elevation_approval_for_program(
            execution,
            idempotency,
            direct_denial,
        )
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> ElevationApprovalScopedOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}
