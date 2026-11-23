use crate::branch::basis_denial::RelationalBranchBasisDenial;
use crate::branch::identity::RelationalBranchIdentity;
use crate::runtime::state::subsystems::history::HistorySubsystem;

pub(crate) fn require_local_branch_identity(
    runtime_instance_id: u64,
    history: &HistorySubsystem,
    identity: &RelationalBranchIdentity,
) -> Result<(), RelationalBranchBasisDenial> {
    if identity.runtime_instance_id() != runtime_instance_id {
        return Err(RelationalBranchBasisDenial::ForeignRuntime {
            expected_runtime_instance_id: runtime_instance_id,
            actual_runtime_instance_id: identity.runtime_instance_id(),
        });
    }
    let current = history
        .branch_cell(identity.branch_id())
        .ok_or_else(|| RelationalBranchBasisDenial::UnknownBranch(identity.branch_id().clone()))?;
    if current.identity() != identity {
        return Err(RelationalBranchBasisDenial::MixedAxis(
            crate::branch::basis_denial::RelationalBranchBasisMismatchAxis::Branch,
        ));
    }
    Ok(())
}

pub(crate) fn identity_denial(
    denial: crate::branch::identity::RelationalBranchIdentityDenial,
) -> RelationalBranchBasisDenial {
    match denial {
        crate::branch::identity::RelationalBranchIdentityDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        } => RelationalBranchBasisDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        },
        crate::branch::identity::RelationalBranchIdentityDenial::UnknownBranch(branch) => {
            RelationalBranchBasisDenial::UnknownBranch(branch)
        }
        crate::branch::identity::RelationalBranchIdentityDenial::IdentityMismatch => {
            RelationalBranchBasisDenial::MixedAxis(
                crate::branch::basis_denial::RelationalBranchBasisMismatchAxis::Branch,
            )
        }
    }
}
