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
        crate::branch::basis_observation_inputs::BranchObservationInputs::from(self)
            .capture_current_visibility_basis(branch_id, version_id)
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
