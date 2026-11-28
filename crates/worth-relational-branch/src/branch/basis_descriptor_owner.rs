//! Custody handoffs for locally issued and recorded exact branch descriptors.
use crate::branch::basis_denial::RelationalBranchBasisDenial;
use crate::branch::basis_descriptor::{
    RelationalBranchBasisDescriptor, RelationalBranchBasisPosture,
    RelationalLiveBranchBasisDescriptorAxes,
};
use crate::branch::reference::observation::RelationalBranchObservationConstructionDenial;
use crate::branch::reference::observation::RelationalBranchReferenceObservation;
use crate::branch::root::RelationalBranchRoot;

#[doc(hidden)]
pub trait RelationalBranchBasisDescriptorOwnerPort: Sized {
    fn in_canonical_frame(&self) -> Result<Self, RelationalBranchObservationConstructionDenial>;
    fn bind_readmitted_root(
        &self,
        runtime_instance_id: u64,
        root: &RelationalBranchRoot,
    ) -> Result<Self, RelationalBranchBasisDenial>;
    fn with_reference(&self, reference: RelationalBranchReferenceObservation) -> Self;
    fn with_recorded_posture(
        axes: RelationalLiveBranchBasisDescriptorAxes,
        posture: RelationalBranchBasisPosture,
    ) -> Self;
}

impl RelationalBranchBasisDescriptorOwnerPort for RelationalBranchBasisDescriptor {
    fn in_canonical_frame(&self) -> Result<Self, RelationalBranchObservationConstructionDenial> {
        self.in_canonical_frame()
    }

    fn bind_readmitted_root(
        &self,
        runtime_instance_id: u64,
        root: &RelationalBranchRoot,
    ) -> Result<Self, RelationalBranchBasisDenial> {
        self.bind_readmitted_root(runtime_instance_id, root)
    }

    fn with_recorded_posture(
        axes: RelationalLiveBranchBasisDescriptorAxes,
        posture: RelationalBranchBasisPosture,
    ) -> Self {
        Self::with_posture(axes, posture)
    }
    fn with_reference(&self, reference: RelationalBranchReferenceObservation) -> Self {
        Self::with_posture(
            RelationalLiveBranchBasisDescriptorAxes {
                runtime_instance_id: self.runtime_instance_id(),
                branch_id: self.branch_id().clone(),
                reference,
                truth_version: self.truth_version(),
                root_identity: self.root_identity(),
                schema_commitment: self.schema_commitment(),
                visibility_commitment: self.visibility_commitment(),
            },
            self.posture(),
        )
    }
}
