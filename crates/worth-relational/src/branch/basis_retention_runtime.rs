use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::basis_denial::RelationalBranchBasisDenial;
use crate::history::retention::{
    RelationalBranchRetentionLease, RelationalBranchRetentionReleaseDenial,
    RelationalBranchRetentionReleaseReceipt,
};
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    pub fn readmit_retained_branch_basis(
        &self,
        descriptor: &crate::branch::basis::RelationalBranchBasisDescriptor,
        lease: &RelationalBranchRetentionLease,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        super::basis_retention::readmit_retained_branch_basis(
            &self.history,
            self.runtime_instance_id(),
            &self.services.instrumentation,
            descriptor,
            lease,
        )
    }

    /// Retain the same owner-issued immutable basis, including after its branch retires.
    pub fn retain_component_basis(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<RelationalBranchRetentionLease, RelationalBranchBasisDenial> {
        super::basis_retention::retain_component_basis(
            &self.history,
            self.runtime_instance_id(),
            &self.services.instrumentation,
            basis,
        )
    }

    pub fn release_component_basis(
        &self,
        lease: RelationalBranchRetentionLease,
    ) -> Result<RelationalBranchRetentionReleaseReceipt, RelationalBranchRetentionReleaseDenial>
    {
        super::basis_retention::release_component_basis(
            &self.history,
            self.runtime_instance_id(),
            lease,
        )
    }
}
