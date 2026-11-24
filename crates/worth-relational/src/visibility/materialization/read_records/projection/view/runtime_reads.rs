//! Select the installed read operands before handing off to the read implementation.
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

impl<'runtime> VisibilityReadContext<'runtime> {
    pub(crate) fn project_branch_head(
        &self,
        branch_id: &crate::history::data::BranchId,
        version_id: VersionId,
    ) -> Result<
        Option<VisibilityProjectionView>,
        crate::branch::basis_denial::RelationalBranchBasisDenial,
    > {
        let basis = self
            .runtime()
            .capture_current_visibility_basis(branch_id, version_id)?;
        Ok(basis.map(|basis| {
            VisibilityProjectionView::new(
                self.runtime(),
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis),
            )
        }))
    }

    /// Projects the exact state at `version_id` on the branch that committed
    /// it, so a fork's commit is read from the fork's own root.
    pub fn try_project_historical_version(
        &self,
        version_id: VersionId,
    ) -> Result<VisibilityProjectionView, crate::branch::basis_denial::RelationalBranchBasisDenial>
    {
        let basis = crate::visibility::cache_state::historical_basis_for_retained_version(
            self.runtime(),
            version_id,
        )
        .map_err(historical_projection_denial)?;
        Ok(VisibilityProjectionView::new(
            self.runtime(),
            crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis),
        ))
    }

    pub(crate) fn try_project_retained_commit(
        &self,
        commit_id: crate::history::data::CommitId,
        branch_id: crate::history::data::BranchId,
        version_id: VersionId,
    ) -> Result<VisibilityProjectionView, crate::branch::basis_denial::RelationalBranchBasisDenial>
    {
        let basis =
            crate::visibility::snapshot_states::HistoricalVisibilityBasis::resolve_retained_commit(
                &self.runtime().history,
                commit_id,
                branch_id,
                version_id,
            )
            .map_err(historical_projection_denial)?;
        Ok(VisibilityProjectionView::new(
            self.runtime(),
            crate::visibility::snapshot_states::SnapshotStateBasis::Historical(basis),
        ))
    }

    /// Retain immutable basis metadata; record reads consult their request.
    pub fn project_snapshot(&self, handle: &SnapshotHandle) -> Option<VisibilityProjectionView> {
        let owner = self.runtime();
        VisibilityProjectionView::from_snapshot(owner.into(), owner.into(), handle)
    }
}
