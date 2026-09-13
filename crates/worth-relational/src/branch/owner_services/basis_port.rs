use super::owner_binding::RelationalOwnerServiceBinding;
use crate::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDenial, RelationalBranchBasisDescriptor,
    RelationalBranchIdentity,
};
use crate::history::retention::{
    RelationalBranchRetentionLease, RelationalBranchRetentionReleaseDenial,
    RelationalBranchRetentionReleaseReceipt,
};

/// Cloneable basis service weakly bound to one Relational runtime owner.
#[derive(Debug, Clone)]
pub struct RelationalBranchBasisPort {
    owner: RelationalOwnerServiceBinding,
}

impl RelationalBranchBasisPort {
    pub(crate) fn from_runtime(runtime: &crate::runtime::RelationalRuntime) -> Self {
        Self::new(RelationalOwnerServiceBinding::new(
            runtime.state_binding(),
            runtime.owner_binding(),
        ))
    }

    pub(super) fn new(owner: RelationalOwnerServiceBinding) -> Self {
        Self { owner }
    }

    pub fn observe_branch(
        &self,
        identity: &RelationalBranchIdentity,
    ) -> Result<
        (
            RelationalBranchBasisDescriptor,
            AdmittedRelationalBranchBasis,
        ),
        RelationalBranchBasisDenial,
    > {
        self.admitted_runtime()?.observe_branch(identity)
    }

    pub fn observe_branch_with_control(
        &self,
        identity: &RelationalBranchIdentity,
        control: &crate::mvcc::RelationalOperationControl,
    ) -> Result<
        (
            RelationalBranchBasisDescriptor,
            AdmittedRelationalBranchBasis,
        ),
        RelationalBranchBasisDenial,
    > {
        self.admitted_runtime()?
            .observe_branch_with_control(identity, control)
    }

    pub fn admit_branch_basis(
        &self,
        identity: &RelationalBranchIdentity,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        self.admitted_runtime()?.admit_branch_basis(identity)
    }

    pub fn readmit_branch_basis(
        &self,
        descriptor: &RelationalBranchBasisDescriptor,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        self.admitted_runtime()?.readmit_branch_basis(descriptor)
    }

    pub fn compare_current_exact(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<(), RelationalBranchBasisDenial> {
        self.admitted_runtime()?.compare_current_exact(basis)
    }

    pub fn readmit_retained_branch_basis(
        &self,
        descriptor: &RelationalBranchBasisDescriptor,
        lease: &RelationalBranchRetentionLease,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        self.admitted_runtime()?
            .readmit_retained_branch_basis(descriptor, lease)
    }

    pub fn retain_component_basis(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<RelationalBranchRetentionLease, RelationalBranchBasisDenial> {
        self.admitted_runtime()?.retain_component_basis(basis)
    }

    pub fn release_component_basis(
        &self,
        lease: RelationalBranchRetentionLease,
    ) -> Result<RelationalBranchRetentionReleaseReceipt, RelationalBranchRetentionReleaseDenial>
    {
        let owner = match self.owner.admitted_runtime() {
            Some(owner) => owner,
            None => {
                return Err(RelationalBranchRetentionReleaseDenial::new(
                    RelationalBranchBasisDenial::OwnerUnavailable,
                    lease,
                ));
            }
        };
        owner.release_component_basis(lease)
    }

    fn admitted_runtime(
        &self,
    ) -> Result<crate::runtime::RelationalRuntime, RelationalBranchBasisDenial> {
        self.owner
            .admitted_runtime()
            .ok_or(RelationalBranchBasisDenial::OwnerUnavailable)
    }
}
