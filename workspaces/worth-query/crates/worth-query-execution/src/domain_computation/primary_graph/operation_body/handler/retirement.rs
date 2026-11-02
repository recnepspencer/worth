//! Compiled bodies retain descriptions and static functions, not live owners.
use super::OperationBodyHandler;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    WorthQueryOperationHomeRetirementDenial as Denial,
    WorthQueryOperationHomeRetirementProgress as Progress,
};
use worth_execution::ExecutionRequest;

impl<Schema, Binding> OperationBodyHandler<Schema, Binding> {
    pub(super) fn preflight_inert_body_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let Self {
            body: _,
            values: _,
            external_emission: _,
            marker: _,
            input_bindings: _,
            outputs: _,
            selection_reads: _,
            retirement_inputs: _,
            child_parents: _,
        } = self;
        // Every field stays with this original handler: body/value/output/read
        // programs, installed scalar recipes, emission recipe and index vectors.
        // Recipes hold static function pointers, never captured runtime owners.
        request.consult().map_err(Denial::Execution)
    }

    pub(super) fn complete_inert_body_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Progress, Denial> {
        self.preflight_inert_body_retirement(request)?;
        Ok(Progress::Complete)
    }
}
