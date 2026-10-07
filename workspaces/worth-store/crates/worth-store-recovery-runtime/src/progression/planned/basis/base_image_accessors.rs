//! Read-only projection of the immutable selected-source and recovery effects.

use super::*;

impl RecoveryBaseImagePlan {
    pub(crate) const fn latest_blob_publication(&self) -> Option<IndexedThroughBlobPublication> {
        self.latest_blob_publication
    }
    pub(crate) const fn latest_blob_quarantine(&self) -> Option<PersistedRecordIdentity> {
        self.latest_blob_quarantine
    }

    pub(crate) const fn tier_epoch_anchor(&self) -> Option<[u8; 32]> {
        self.tier_epoch_anchor
    }

    pub(crate) const fn derived_family_directory(
        &self,
    ) -> Option<DerivedFamilyRootDirectoryBinding> {
        self.derived_family_directory
    }
    pub const fn selected_selector(&self) -> DurableRootSelector {
        self.selected_selector
    }
    pub const fn selected_root(&self) -> &DurablePhysicalRootManifest {
        &self.selected_root
    }
    pub(crate) fn selected_root_topology(&self) -> &[SelectedRootTopologyEntry] {
        &self.selected_root_topology
    }
    pub const fn destination_generation(&self) -> u64 {
        self.destination_generation
    }
    pub fn actions(&self) -> &[RecoveryBaseImageAction] {
        &self.actions
    }
    pub fn segment_updates(&self) -> &[RecoverySegmentRoutingAction] {
        &self.segment_updates
    }
    pub fn manifests(&self) -> &[RecoveryPayloadManifestAction] {
        &self.manifests
    }
    pub fn root_states(&self) -> &[PersistedPhysicalRecoveryRootState] {
        &self.root_states
    }
    pub(crate) fn release_head_replay(
        &self,
    ) -> Option<&worth_store_recovery_physics::VerifiedSelectedReleaseHeadReplayV14> {
        self.release_head_replay
            .as_ref()
            .map(crate::progression::PendingReleaseReplay::head)
    }
    pub(crate) fn release_directory_replacement(
        &self,
    ) -> Option<&worth_store_recovery_physics::VerifiedReleasedDirectoryReplacement> {
        self.release_head_replay
            .as_ref()
            .and_then(crate::progression::PendingReleaseReplay::directory)
    }
    pub fn source_artifacts(&self) -> &[RecordArtifactFile] {
        &self.source_artifacts
    }
}
