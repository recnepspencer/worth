use super::*;

impl super::super::authority::VisibilityAuthority<'_> {
    pub fn release_snapshot(
        &self,
        handle: &crate::snapshots::data::SnapshotHandle,
    ) -> Result<RelationalSnapshotReleaseReceipt, RelationalSnapshotReleaseDenial> {
        crate::visibility::snapshot_observation::SnapshotObservationInputs::from(self.runtime)
            .release_snapshot(handle)
    }
}
