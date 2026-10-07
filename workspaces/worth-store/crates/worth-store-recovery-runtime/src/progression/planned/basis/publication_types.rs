use worth_store_physical_format::{DurableFreeSpaceManifestHeader, RecordArtifactFile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPublicationCandidateArtifact {
    pub(super) artifact: RecordArtifactFile,
    pub(super) bytes: Box<[u8]>,
    pub(super) payload_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecoveryReleaseTopologyProof {
    pub(crate) source_free: DurableFreeSpaceManifestHeader,
    pub(crate) published_free: DurableFreeSpaceManifestHeader,
    pub(crate) transition: worth_store_recovery_physics::VerifiedReleasedV3InventoryTransition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryPublicationAction {
    MaterializeRootCandidate { artifact: RecordArtifactFile },
    SynchronizeRootCandidate { artifact: RecordArtifactFile },
    ReplaceRootProtocol,
    SynchronizeStoreNamespace,
}
