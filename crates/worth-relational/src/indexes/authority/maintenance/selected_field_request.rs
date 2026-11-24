//! Select the required field generation using the installed observation.
use super::super::live_access::IndexAuthority;
use super::field_request;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::indexes::data::*;
use worth_execution::ExecutionRequest;

impl IndexAuthority<'_> {
    /// Ensure one field generation from an exact root or authentic fork seed.
    pub fn ensure_field_index_for_basis(
        &self,
        index: DerivedIndexId,
        basis: &AdmittedRelationalBranchBasis,
        budget: DerivedIndexMaintenanceBudget,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), DerivedIndexMaintenanceDenial> {
        field_request::ensure_field_index_for_basis(
            &self.runtime.indexes,
            index,
            basis,
            budget,
            execution,
            |observation| {
                self.runtime
                    .read_truth()
                    .project_observation(observation)
                    .map(|_| ())
            },
            |request, work| self.prepare_basis_refresh(request, basis, None, work, execution),
        )
    }
}
