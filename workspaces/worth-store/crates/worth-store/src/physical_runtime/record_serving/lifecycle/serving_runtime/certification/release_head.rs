use super::ServingPhysicalRuntime;

impl ServingPhysicalRuntime {
    /// The selected release-head custody, or nothing while the release
    /// ledger is unavailable or its rosters fail their own commitment.
    pub fn certification_release_head_observation(
        &self,
    ) -> Option<crate::physical_runtime::durability::CertificationReleaseHeadObservation> {
        self.parts.publication.release_head_observation()
    }
}
