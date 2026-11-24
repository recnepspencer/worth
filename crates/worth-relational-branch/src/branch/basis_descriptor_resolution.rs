use worth_foundational::FoundationalBranchTarget;

use crate::branch::basis::RelationalBranchBasisDescriptor;
use crate::branch::basis::RelationalBranchBasisPosture;
use crate::branch::basis::ResolvedRelationalBasisDescriptor;
use crate::branch::basis::RELATIONAL_BRANCH_BASIS_DESCRIPTOR_VERSION;
use crate::branch::basis_denial::RelationalBranchBasisDenial;

pub fn resolve_relational_branch_basis_descriptor(
    descriptor: RelationalBranchBasisDescriptor,
) -> Result<ResolvedRelationalBasisDescriptor, RelationalBranchBasisDenial> {
    if descriptor.descriptor_version() != RELATIONAL_BRANCH_BASIS_DESCRIPTOR_VERSION {
        return Err(RelationalBranchBasisDenial::UnsupportedDescriptorVersion {
            supported: RELATIONAL_BRANCH_BASIS_DESCRIPTOR_VERSION,
            actual: descriptor.descriptor_version(),
        });
    }
    let expected_branch_identity = format!(
        "relational/{}/{}",
        descriptor.runtime_instance_id(),
        descriptor.branch_id().0
    );
    if descriptor.reference().branch_id().as_str() != expected_branch_identity {
        return Err(RelationalBranchBasisDenial::MalformedDescriptor);
    }
    match descriptor.reference().target() {
        FoundationalBranchTarget::Empty if descriptor.root_identity() != 0 => {
            return Err(RelationalBranchBasisDenial::EmptyCommittedTargetMismatch);
        }
        FoundationalBranchTarget::Basis(target)
            if target.runtime_instance_id() != descriptor.runtime_instance_id() =>
        {
            return Err(RelationalBranchBasisDenial::MalformedDescriptor);
        }
        _ => {}
    }
    match descriptor.posture() {
        RelationalBranchBasisPosture::Live => {}
        RelationalBranchBasisPosture::Archived => {
            return Err(RelationalBranchBasisDenial::ArchivedBranch(
                descriptor.branch_id().clone(),
            ));
        }
        RelationalBranchBasisPosture::Deleting => {
            return Err(RelationalBranchBasisDenial::DeletingBranch(
                descriptor.branch_id().clone(),
            ));
        }
    }
    Ok(ResolvedRelationalBasisDescriptor::new(descriptor))
}
