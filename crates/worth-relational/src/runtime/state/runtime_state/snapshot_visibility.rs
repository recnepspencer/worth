//! The installed owner selects the current branch observation for visibility.
use super::RelationalRuntime;
use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use crate::visibility::snapshot_states::VisibilitySnapshotBasis;

impl RelationalRuntime {
    pub(crate) fn capture_current_visibility_basis(
        &self,
        branch_id: &BranchId,
        version_id: VersionId,
    ) -> Result<
        Option<VisibilitySnapshotBasis>,
        crate::branch::basis_denial::RelationalBranchBasisDenial,
    > {
        let identity = self
            .branch_identity(branch_id)
            .map_err(branch_identity_denial)?;
        let (_, basis) = self.observe_branch(&identity)?;
        let observation = basis.observation();
        if observation.version_id() != version_id {
            return Ok(None);
        }
        Ok(Some(VisibilitySnapshotBasis::from_observation(
            &observation,
        )))
    }

    /// Best-effort selection for optional derived-state maintenance. The
    /// retention owner records any admission denial; authoritative truth and
    /// branch-head movement never depend on this cache-only promotion.
    pub(crate) fn capture_optional_current_visibility_basis(
        &self,
        branch_id: &BranchId,
        version_id: VersionId,
    ) -> Option<VisibilitySnapshotBasis> {
        self.capture_current_visibility_basis(branch_id, version_id)
            .ok()
            .flatten()
    }
}

fn branch_identity_denial(
    denial: crate::branch::identity::RelationalBranchIdentityDenial,
) -> crate::branch::basis_denial::RelationalBranchBasisDenial {
    match denial {
        crate::branch::identity::RelationalBranchIdentityDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        } => crate::branch::basis_denial::RelationalBranchBasisDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        },
        crate::branch::identity::RelationalBranchIdentityDenial::UnknownBranch(branch_id) => {
            crate::branch::basis_denial::RelationalBranchBasisDenial::UnknownBranch(branch_id)
        }
        crate::branch::identity::RelationalBranchIdentityDenial::IdentityMismatch => {
            crate::branch::basis_denial::RelationalBranchBasisDenial::MixedAxis(
                crate::branch::basis_denial::RelationalBranchBasisMismatchAxis::Branch,
            )
        }
    }
}
