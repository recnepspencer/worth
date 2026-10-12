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
        let request = program.request_scope().clone();
        let mut candidate = Some(program);
        self.with_application_advancement(&request, |active_phase| {
            let execution = active_phase
                .execution_request_for(&self.product_runtime)
                .map_err(
                    crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from,
                )?;
            Ok(self.compare_and_commit_elevation_close_for_program(
                &execution,
                &active_phase,
                candidate.take().expect("candidate admitted once"),
                idempotency,
                self.direct_operation_commit_denial::<Operation>(),
            ))
        })
        .and_then(std::convert::identity)
        .unwrap_or_else(|denial| match candidate.take() {
            Some(candidate) => self.refuse_elevation_close(candidate, denial),
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
        let mut program = program.into_inner();
        if let Some(denial) = direct_denial {
            let Some(binding) = program.read_set.admission.take_elevation_close_binding() else {
                return WorthQueryElevationCloseOutcome::Indeterminate;
            };
            return WorthQueryElevationCloseOutcome::Denied(denial, binding.into_approved());
        }
        if validate_elevation_close_program(&program).is_err() {
            let Some(binding) = program.read_set.admission.take_elevation_close_binding() else {
                return WorthQueryElevationCloseOutcome::Indeterminate;
            };
            return WorthQueryElevationCloseOutcome::Denied(
                WorthQueryApplicationCommitDenial::elevation_close_program_mismatch(),
                binding.into_approved(),
            );
        }
        let currentness = program
            .read_set
            .admission
            .elevation_close_binding()
            .map(WorthQueryElevationCommitCurrentness::close);
        let Some(binding) = program.read_set.admission.take_elevation_close_binding() else {
            return WorthQueryElevationCloseOutcome::Indeterminate;
        };
        closed_outcome(
            self.compare_and_commit_application_inner_with_currentness(
                execution,
                phase,
                program,
                idempotency,
                currentness,
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            ),
            binding,
        )
    }
}
