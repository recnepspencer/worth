//! Bind Runtime's selected observation and predecessor snapshot to field refresh.
use super::super::live_access::IndexAuthority;
use super::field_refresh;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::indexes::data::*;
use crate::snapshots::data::SnapshotHandle;
use worth_execution::ExecutionRequest;

impl IndexAuthority<'_> {
    /// Maintain selected field indexes on the original request's work and memory payer.
    pub fn refresh_field_indexes_for_basis(
        &self,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        before: Option<&SnapshotHandle>,
        budget: DerivedIndexMaintenanceBudget,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        field_refresh::refresh_field_indexes_for_basis(
            &self.runtime.indexes,
            request,
            budget,
            execution,
            |request, work| self.prepare_basis_refresh(request, basis, before, work, execution),
        )
    }

    /// Maintain selected fields on the caller's original preparation payer.
    pub fn refresh_field_indexes_for_basis_admitted<Stop>(
        &self,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        budget: DerivedIndexMaintenanceBudget,
        execution: &ExecutionRequest<'_, '_>,
        prepare: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceAdmissionStop<Stop>> {
        field_refresh::refresh_field_indexes_for_basis_admitted(
            &self.runtime.indexes,
            request,
            budget,
            |request, work| self.prepare_basis_refresh(request, basis, None, work, execution),
            prepare,
        )
    }
}
