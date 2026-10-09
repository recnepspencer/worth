use super::*;

impl<D: 'static, O, F: 'static, L: BasisOperationLane>
    WorthQueryGraphCompletedDirectExecution<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<
        D,
        F,
        Execution = super::super::super::super::WorthQueryDirectOperation,
    >,
{
    pub(super) fn invoke_executor(
        self,
        execution: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
    ) -> super::super::super::WorthQueryBoundExecutionOutcome<D, O, F, L, O::Output> {
        let Self {
            prepared,
            graph_receipts,
        } = self;
        let WorthQueryPreparedDirectExecution {
            bound,
            input,
            executor,
            phase_proof,
            running,
            resources,
            execution_snapshot,
            conditional,
            resource_evidence,
            mut counters,
        } = prepared;
        let running = running.expect("completed graph execution owns its managed run");
        let context = WorthQueryOperationExecutionContext::new(
            execution
                .execution_request_for(&workspace.advancement_owner())
                .expect("executor uses its workspace phase"),
            bound.definition(),
            bound.binding_identity(),
            bound.basis().capability_digest(),
            bound.basis().normalized(),
            executor.installed_read.as_ref(),
            &graph_receipts,
            &resources,
            running.provider_session_identity(),
        );
        counters.executor_contacts += 1;
        let (material, primary_read_contacts) =
            match executor.execute::<D, O, F>(input, &context, workspace) {
                Ok(material) => material,
                Err(failure) => {
                    let kind = classify_executor_failure(&bound, failure.class().clone());
                    let detail = failure.detail().to_owned();
                    let cleanup = running.abandon().cleanup();
                    return TransitionOutcome::Failed(
                        WorthQueryBoundExecutionDenial::new(kind, detail, counters)
                            .with_graph_receipts(graph_receipts)
                            .with_managed_cleanup(cleanup),
                    );
                }
            };
        counters.primary_read_contacts += primary_read_contacts;
        let (output, result_state, warnings, material) = material.into_parts();
        WorthQueryExecutorCompletedDirectExecution {
            bound,
            phase_proof,
            running,
            resources,
            output,
            result_state,
            warnings,
            material,
            graph_receipts,
            snapshot: execution_snapshot,
            conditional,
            resource_evidence,
            counters,
        }
        .validate_terminal()
    }
}

impl<D: 'static, O, F: 'static, L: BasisOperationLane, Output>
    WorthQueryExecutorCompletedDirectExecution<D, O, F, L, Output>
where
    O: WorthQueryExecutableDomainOperation<D, F, Output = Output>,
    Output: super::super::super::super::WorthQueryOperationOutput,
{
    pub(super) fn validate_terminal(
        mut self,
    ) -> super::super::super::WorthQueryBoundExecutionOutcome<D, O, F, L, Output> {
        self.counters.terminal_posture_checks += 1;
        if !self
            .bound
            .definition()
            .semantics()
            .terminal
            .result_states
            .contains(&self.result_state)
        {
            let cleanup = self.running.abandon().cleanup();
            return TransitionOutcome::Denied(
                WorthQueryBoundExecutionDenial::new(
                    WorthQueryBoundExecutionDenialKind::UndeclaredResultState,
                    "executor returned a result state absent from the installed terminal contract",
                    self.counters,
                )
                .with_graph_receipts(self.graph_receipts)
                .with_managed_cleanup(cleanup),
            );
        }
        WorthQueryValidatedDirectEvidenceCompletion {
            bound: self.bound,
            phase_proof: self.phase_proof,
            running: Some(self.running),
            resources: self.resources,
            output: self.output,
            result_state: self.result_state,
            warnings: self.warnings,
            material: self.material,
            graph_receipts: self.graph_receipts,
            snapshot: self.snapshot,
            conditional: self.conditional,
            resource_evidence: self.resource_evidence,
            counters: self.counters,
        }
        .finish()
    }
}
