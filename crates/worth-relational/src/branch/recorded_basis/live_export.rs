use super::{RecordedRelationalComponentReference, RelationalRecordedComponentRefusal as Refusal};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    /// Export descriptions of one actual current subject. The artifact digest
    /// does not qualify memory backing as durable source evidence.
    pub(crate) fn export_recorded_current_component(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<RecordedRelationalComponentReference, Refusal> {
        crate::execution::request_custody::check_request(execution).map_err(Refusal::Request)?;
        self.compare_current_exact(basis).map_err(Refusal::Basis)?;
        let recorded = self.export_recorded_retained_component(execution, basis)?;
        self.compare_current_exact(basis).map_err(Refusal::Basis)?;
        Ok(recorded)
    }

    /// Record the exact subject already pinned by this actual admitted basis,
    /// even after another publication advances the branch. No descriptor can
    /// be paired with a different result or selected by current-head fallback.
    pub(crate) fn export_recorded_retained_component(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<RecordedRelationalComponentReference, Refusal> {
        super::export::export_retained_component(
            execution,
            basis,
            self.runtime_instance_id(),
            &self.history,
            self.durability.checkpoint_selection(),
        )
    }
}
