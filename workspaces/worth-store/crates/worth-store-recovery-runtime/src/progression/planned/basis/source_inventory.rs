use super::*;

mod retained_storage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecoverySelectedSourceInventory {
    pub(crate) free_space: DurableFreeSpaceManifestHeader,
    pub(crate) segment_pages: BTreeMap<(u64, u64), RecoverySelectedSegmentPage>,
    pub(crate) segment_topology:
        BTreeMap<(u64, u64), worth_store_physical_format::PhysicalSegmentMembershipBlock>,
    pub(crate) free_entries: Box<[RecordFreeSpaceManifestEntry]>,
    pub(crate) free_topology:
        BTreeMap<(u64, u64), worth_store_physical_format::PhysicalFreeSpaceMembershipBlock>,
    pub(crate) source_artifacts: Box<[RecordArtifactFile]>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RecoveryObservedSuccessorCandidate {
    pub(crate) root: DurablePhysicalRootManifest,
    pub(crate) free_space: DurableFreeSpaceManifestHeader,
    pub(crate) placements: Box<[CurrentPhysicalRecordPlacement]>,
    pub(crate) segment_entries: Box<[RecordSegmentPageManifestEntry]>,
    pub(crate) free_entries: Box<[RecordFreeSpaceManifestEntry]>,
    pub(crate) referenced_artifacts: Box<[RecordArtifactFile]>,
    pub(crate) artifacts: Box<[RecoveryObservedCandidateArtifact]>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RecoveryObservedCandidateArtifact {
    pub(crate) artifact: RecordArtifactFile,
    pub(crate) bytes: Box<[u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecoverySelectedSegmentPage {
    pub(crate) entry: RecordSegmentPageManifestEntry,
    pub(crate) routing_identity: [u8; 32],
    pub(crate) membership_artifact: RecordArtifactFile,
}
