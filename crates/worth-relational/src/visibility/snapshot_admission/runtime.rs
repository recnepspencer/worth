use super::*;

impl super::super::authority::VisibilityAuthority<'_> {
    pub(in crate::visibility) fn open_active_snapshot_for_basis(
        &self,
        basis: crate::history::retention::VisibilitySnapshotBasis,
        read_policy: crate::snapshots::data::SnapshotReadPolicy,
    ) -> Result<crate::snapshots::data::SnapshotHandle, RelationalSnapshotAdmissionDenial> {
        crate::visibility::snapshot_observation::SnapshotObservationInputs::from(self.runtime)
            .open_active_snapshot_for_basis(basis, read_policy)
    }

    pub fn snapshot_for_observation(
        &self,
        observation: &crate::mvcc::observation::RelationalBranchObservation,
    ) -> Result<crate::snapshots::data::SnapshotHandle, RelationalSnapshotAdmissionDenial> {
        crate::visibility::snapshot_observation::SnapshotObservationInputs::from(self.runtime)
            .snapshot_for_observation(observation)
    }

    pub(crate) fn snapshot(
        &self,
    ) -> Result<crate::snapshots::data::SnapshotHandle, CurrentBranchSnapshotAdmissionDenial> {
        use CurrentBranchSnapshotAdmissionDenial as Denial;
        let version = self.runtime.current_version_id();
        let branch = super::super::branch_scope::branch_for_version(self.runtime, version)
            .ok_or(Denial::CurrentVersionUnavailable)?;
        let basis = self
            .runtime
            .capture_current_visibility_basis(&branch, version)
            .map_err(Denial::Basis)?
            .ok_or(Denial::CurrentVersionUnavailable)?;
        self.open_active_snapshot_for_basis(
            basis,
            crate::snapshots::data::SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        )
        .map_err(Denial::Snapshot)
    }
}
