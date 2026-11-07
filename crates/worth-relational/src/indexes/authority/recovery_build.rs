//! Explicit recovery operations retain the lane inside their fixed implementation.
use super::{DerivedIndexBuildOutcome, DerivedIndexBuildRequest, IndexAuthority};
use crate::branch::AdmittedRelationalBranchBasis;
use crate::storage::substrate::maintenance::ReconstructionLane;

impl IndexAuthority<'_> {
    /// Explicit recovery: build against the exact immutable root already carried by an admitted
    /// owner basis. This path performs no historical retention reacquisition.
    /// The caller's request supplies allocation authority and its lease budget.
    pub fn build_for_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
    ) -> DerivedIndexBuildOutcome {
        ReconstructionLane::enter_reconstruction(|lane| {
            self.reconstruct_basis_image(execution, request, basis, lane)
        })
    }

    /// Explicit recovery: build using the allocation authority and lease budget of the caller's request.
    pub fn build_for_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
    ) -> DerivedIndexBuildOutcome {
        ReconstructionLane::enter_reconstruction(|lane| {
            self.reconstruct_commit_image(execution, request, lane)
        })
    }
}
