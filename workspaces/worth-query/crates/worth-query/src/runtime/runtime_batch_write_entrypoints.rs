use super::*;

impl WorthQueryRuntime {
    pub fn write_batch(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commands: Vec<WorthQueryWriteCommand>,
    ) -> Result<WorthQueryBatchWriteReceipt, WorthQueryRuntimeError> {
        self.write_batch_intent(commands).execute(execution)
    }

    pub(crate) fn write_graph_batch(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commands: Vec<WorthQueryWriteCommand>,
        graph_composition_breadth: WorthQueryGraphCompositionBreadth,
        graph_composition_program: WorthQueryGraphCompositionProgram,
    ) -> Result<WorthQueryBatchWriteReceipt, WorthQueryRuntimeError> {
        let review = self.review_authoritative_runtime_write_batch_with_graph_artifacts(
            commands,
            graph_composition_breadth,
            graph_composition_program,
        )?;
        let handoff = self.resolve_reviewed_admitted_authoritative_write_batch_handoff(review)?;
        let binding = self.prepare_authoritative_mutation_batch_execution_binding(handoff);
        self.execute_authoritative_mutation_batch_execution_binding(execution, binding)
    }
}
