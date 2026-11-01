//! Descriptive history selection comes from the actual retained observation.
use super::ManagedIndexSourceObservation;
use crate::history::{
    RelationalAuthoredManagedSourceSelector, RelationalAuthoredWriteDenial as Denial,
};
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;
impl ManagedIndexSourceObservation {
    /// Select metadata for this actual program/scope, not a reconstructed key.
    /// Retained observation custody grants no Source READ or correction permission.
    pub fn authored_history_selector(
        &self,
        runtime: &RelationalRuntime,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<RelationalAuthoredManagedSourceSelector, Denial> {
        execution.consult().map_err(Denial::Request)?;
        if self.selected.runtime_instance_id() != runtime.runtime_instance_id() {
            return Err(Denial::ForeignOwner);
        }
        runtime.authored_managed_source_selector(execution, &self.record.program, self.record.scope)
    }
}
