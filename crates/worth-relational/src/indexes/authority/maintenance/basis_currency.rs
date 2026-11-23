//! Bind installed branch and index custody for exact-basis maintenance.
use super::super::live_access::IndexAuthority;
use super::currency_selection;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::indexes::data::*;
use worth_execution::ExecutionRequest;

impl IndexAuthority<'_> {
    /// A superseded selection may already have retired. Each read admits its own generation.
    pub fn verify_index_basis_for_commit(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        commit: &crate::history::data::RelationalCommitReceipt,
    ) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
        currency_selection::verify_index_basis_for_commit(
            &self.runtime.indexes,
            &self.runtime.history,
            execution,
            commit,
        )
    }

    /// Carry the exact selected generation across a genuine fresh fork.
    pub fn carry_index_basis_for_basis(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<DerivedIndexMaintenanceWork, DerivedIndexMaintenanceDenial> {
        currency_selection::carry_index_basis_for_basis(
            &self.runtime.indexes,
            execution,
            basis,
            |observation| self.runtime.read_truth().project_observation(observation),
        )
    }

    /// Explicit recovery of the installed indexes' cold projections.
    pub fn recover_installed_indexes_for_basis(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<DerivedIndexBuildOutcome, DerivedIndexMaintenanceDenial> {
        let request = currency_selection::installed_build_request(&self.runtime.indexes, basis)?;
        Ok(self.build_for_basis(execution, request, basis))
    }
}
