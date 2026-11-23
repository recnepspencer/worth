//! The installed Runtime binds the branch directory used for readmission.
use super::basis::{AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor};
use super::basis_denial::RelationalBranchBasisDenial;
use super::basis_observation_inputs::BranchObservationInputs;
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    /// Resolve a transported descriptor and readmit it through live owner
    /// state or an already-retained exact immutable basis.
    pub fn readmit_branch_basis(
        &self,
        descriptor: &RelationalBranchBasisDescriptor,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        BranchObservationInputs::from(self).readmit_branch_basis(descriptor)
    }
}
