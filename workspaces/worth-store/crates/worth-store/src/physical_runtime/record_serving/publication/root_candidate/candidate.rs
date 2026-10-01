use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, RecordArtifactFile,
};

use super::super::PublicationPlan;

pub(in crate::physical_runtime) struct PreparedPhysicalRootCandidate {
    source_root: DurablePhysicalRootManifest,
    successor_free_space: DurableFreeSpaceManifestHeader,
    plan: PublicationPlan,
    artifacts: Box<[RecordArtifactFile]>,
}

impl PreparedPhysicalRootCandidate {
    pub(in crate::physical_runtime) const fn retained_routing_metadata_bytes(&self) -> Option<u64> {
        self.plan.routing_metadata_bytes
    }

    pub(in crate::physical_runtime::record_serving) fn new(
        source_root: DurablePhysicalRootManifest,
        successor_free_space: DurableFreeSpaceManifestHeader,
        plan: PublicationPlan,
        artifacts: Box<[RecordArtifactFile]>,
    ) -> Self {
        Self {
            source_root,
            successor_free_space,
            plan,
            artifacts,
        }
    }

    pub(in crate::physical_runtime) const fn source_root(&self) -> &DurablePhysicalRootManifest {
        &self.source_root
    }

    pub(in crate::physical_runtime) const fn successor_root(&self) -> &DurablePhysicalRootManifest {
        &self.plan.manifest
    }

    pub(in crate::physical_runtime) const fn catalog_candidate(&self) -> RecordArtifactFile {
        self.plan.candidate
    }

    pub(in crate::physical_runtime) fn artifacts(&self) -> &[RecordArtifactFile] {
        &self.artifacts
    }

    pub(in crate::physical_runtime) fn into_root_parts(
        self,
    ) -> (
        DurablePhysicalRootManifest,
        DurableFreeSpaceManifestHeader,
        DurablePhysicalRootManifest,
        Box<[RecordArtifactFile]>,
        crate::physical_runtime::RecordRootPlanningObservation,
    ) {
        let observation = crate::physical_runtime::RecordRootPlanningObservation::from_publication(
            self.plan.observation,
        );
        (
            self.source_root,
            self.successor_free_space,
            self.plan.manifest,
            self.artifacts,
            observation,
        )
    }

    pub(in crate::physical_runtime) fn commit_arena_reservations(&mut self) {
        for reservation in std::mem::take(&mut self.plan.arena_reservations) {
            reservation.publish();
        }
    }
}
