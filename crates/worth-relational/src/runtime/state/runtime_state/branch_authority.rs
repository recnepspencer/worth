use super::RelationalRuntime;
use crate::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDenial, RelationalBranchIdentity,
};
use crate::history::data::BranchId;
use crate::mvcc::validation::RelationalTransactionValidationInput;

impl RelationalRuntime {
    /// Observe every exact axis of one owner branch-reference cell. This is a
    /// read-only evidence surface; the returned state cannot mint authority or
    /// select a transaction target.
    pub fn branch_reference_state(
        &self,
        branch_id: &BranchId,
    ) -> Option<crate::branch::RelationalBranchReferenceState> {
        self.history
            .branch_cell(branch_id)
            .map(|cell| cell.evidence_state())
    }

    /// Return the owner-issued identity for one registered branch.
    ///
    /// The identity is descriptive; it carries runtime affinity so a value
    /// copied from another runtime cannot silently route a transaction here.
    pub fn branch_identity(
        &self,
        branch_id: &BranchId,
    ) -> Result<RelationalBranchIdentity, crate::branch::RelationalBranchIdentityDenial> {
        let cell = self.history.branch_cell(branch_id).ok_or_else(|| {
            crate::branch::RelationalBranchIdentityDenial::UnknownBranch(branch_id.clone())
        })?;
        if cell.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(
                crate::branch::RelationalBranchIdentityDenial::ForeignRuntime {
                    expected_runtime_instance_id: self.runtime_instance_id(),
                    actual_runtime_instance_id: cell.identity().runtime_instance_id(),
                },
            );
        }
        Ok(cell.identity().clone())
    }

    /// Return the configured main-branch identity through the same owner door
    /// used by named branches.
    pub fn main_branch_identity(&self) -> RelationalBranchIdentity {
        self.branch_identity(&self.config.history.main_branch)
            .expect("configured main branch must remain registered")
    }

    /// Assemble owner-private validation input for one exact admitted basis.
    pub(crate) fn transaction_validation_input_for(
        &self,
        identity: &RelationalBranchIdentity,
    ) -> Result<RelationalTransactionValidationInput, RelationalBranchBasisDenial> {
        let basis = self.admitted_branch_basis_for_identity(identity)?;
        Ok(RelationalTransactionValidationInput::for_owner_basis(
            &basis,
        ))
    }

    pub(crate) fn admitted_branch_basis_for_identity(
        &self,
        identity: &RelationalBranchIdentity,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        self.observe_branch(identity).map(|(_, basis)| basis)
    }

    /// Admit one exact transaction basis for an explicit owner-issued branch
    /// identity. Mutation intent is bound only at the MVCC begin port.
    pub fn admit_branch_basis(
        &self,
        identity: &RelationalBranchIdentity,
    ) -> Result<AdmittedRelationalBranchBasis, RelationalBranchBasisDenial> {
        self.admitted_branch_basis_for_identity(identity)
    }

    pub(crate) fn admitted_branch_basis_for_merge_branch(
        &self,
        branch_id: &BranchId,
    ) -> Result<
        AdmittedRelationalBranchBasis,
        crate::merge::data::RelationalMergeRequestBindingDenial,
    > {
        let identity = self
            .branch_identity(branch_id)
            .map_err(|denial| map_basis_denial(identity_to_basis_denial(denial)))?;
        self.admitted_branch_basis_for_identity(&identity)
            .map_err(map_basis_denial)
    }

    /// The id of this runtime instance, unique within the process. Every
    /// runtime-affine value this runtime issues carries it.
    pub fn runtime_instance_id(&self) -> u64 {
        self.services.runtime_instance_id()
    }
}

fn identity_to_basis_denial(
    denial: crate::branch::RelationalBranchIdentityDenial,
) -> RelationalBranchBasisDenial {
    match denial {
        crate::branch::RelationalBranchIdentityDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        } => RelationalBranchBasisDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        },
        crate::branch::RelationalBranchIdentityDenial::UnknownBranch(branch) => {
            RelationalBranchBasisDenial::UnknownBranch(branch)
        }
        crate::branch::RelationalBranchIdentityDenial::IdentityMismatch => {
            RelationalBranchBasisDenial::MixedAxis(
                crate::branch::RelationalBranchBasisMismatchAxis::Branch,
            )
        }
    }
}

fn map_basis_denial(
    denial: RelationalBranchBasisDenial,
) -> crate::merge::data::RelationalMergeRequestBindingDenial {
    match denial {
        RelationalBranchBasisDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        } => crate::merge::data::RelationalMergeRequestBindingDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        },
        RelationalBranchBasisDenial::UnknownBranch(branch) => {
            crate::merge::data::RelationalMergeRequestBindingDenial::UnknownBranch(branch)
        }
        RelationalBranchBasisDenial::ArchivedBranch(branch) => {
            crate::merge::data::RelationalMergeRequestBindingDenial::ArchivedBranch(branch)
        }
        RelationalBranchBasisDenial::DeletingBranch(branch) => {
            crate::merge::data::RelationalMergeRequestBindingDenial::DeletingBranch(branch)
        }
        RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            crate::merge::data::RelationalMergeRequestBindingDenial::RetentionCapacityExhausted
        }
        RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            crate::merge::data::RelationalMergeRequestBindingDenial::RetentionIdentityExhausted
        }
        RelationalBranchBasisDenial::SnapshotIdentityExhausted => {
            crate::merge::data::RelationalMergeRequestBindingDenial::SnapshotIdentityExhausted
        }
        _ => crate::merge::data::RelationalMergeRequestBindingDenial::IdentityMismatch,
    }
}
