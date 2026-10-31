//! Rebind only an owner-readmitted exact root to the recovering Native frame.
use super::super::basis::RelationalLiveBranchBasisDescriptorAxes;
use crate::branch::{
    RelationalBranchBasisDenial, RelationalBranchBasisDescriptor, RelationalBranchRoot,
};

impl RelationalBranchBasisDescriptor {
    pub(crate) fn bind_readmitted_root(
        &self,
        runtime_instance_id: u64,
        root: &RelationalBranchRoot,
    ) -> Result<Self, RelationalBranchBasisDenial> {
        let visibility = root
            .axes()
            .map(|axes| axes.visibility.digest())
            .unwrap_or([0; 32]);
        if self.schema_commitment() != root.schema_authority().authority_digest()
            || self.visibility_commitment() != visibility
        {
            return Err(RelationalBranchBasisDenial::WrongImmutableTarget);
        }
        let target = match self.reference().target() {
            worth_foundational::FoundationalBranchTarget::Empty => {
                worth_foundational::FoundationalBranchTarget::Empty
            }
            worth_foundational::FoundationalBranchTarget::Basis(target) => {
                worth_foundational::FoundationalBranchTarget::Basis(
                    target.rebind_runtime_instance_id(runtime_instance_id),
                )
            }
        };
        let reference = crate::branch::relational_branch_observation(
            runtime_instance_id,
            &self.branch_id().0,
            target,
            self.reference().generation(),
        )
        .map_err(|_| RelationalBranchBasisDenial::MalformedDescriptor)?;
        Ok(Self::with_posture(
            RelationalLiveBranchBasisDescriptorAxes {
                runtime_instance_id,
                branch_id: self.branch_id().clone(),
                reference,
                truth_version: self.truth_version(),
                root_identity: root.id(),
                schema_commitment: self.schema_commitment(),
                visibility_commitment: self.visibility_commitment(),
            },
            self.posture(),
        ))
    }
}
