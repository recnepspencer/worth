#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PhysicalWorkObligationOperationCode {
    ArtifactRangeRead = 1,
    ArtifactRangeWrite = 2,
    ArtifactPublication = 3,
    ArtifactMetadataRead = 4,
    WalAppend = 5,
    DurabilityBarrier = 6,
    CheckpointCapture = 7,
    WalReclamation = 8,
    RootPublication = 9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkArtifactCode {
    BootstrapCatalog,
    CurrentRootSelector,
    PreviousRootSelector,
    RootSelectorCandidate { role: u8, publication: u64 },
    CatalogCandidate { publication: u64 },
    RootManifest { generation: u64 },
    RootRoutingBlock { generation: u64, block: u64 },
    Segment { segment: u64, generation: u64 },
    SegmentManifest { segment: u64, generation: u64 },
    SegmentMembershipBlock { generation: u64, block: u64 },
    Extent { extent: u64, generation: u64 },
    ExtentManifest { extent: u64, generation: u64 },
    FreeSpaceManifest { generation: u64 },
    FreeSpaceMembershipBlock { generation: u64, block: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkCheckpointActionCode {
    CreateCandidate { byte_count: u64 },
    AppendCandidate { offset: u64, byte_count: u64 },
    SynchronizeCandidate,
    RemoveCandidate,
    PublishCandidate,
    SynchronizeNamespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWorkObligationTargetCode {
    Range {
        artifact: PhysicalWorkArtifactCode,
        offset: u64,
        byte_count: u64,
    },
    WalArtifactInterval {
        segment: u64,
        generation: u64,
        offset: u64,
        byte_count: u64,
    },
    Checkpoint {
        sequence: u64,
        action: PhysicalWorkCheckpointActionCode,
    },
    WalSegmentReclamation {
        segment: u64,
        generation: u64,
    },
    ArtifactFileSynchronization(PhysicalWorkArtifactCode),
    ArtifactParentSynchronization(PhysicalWorkArtifactCode),
    CatalogReplacement(PhysicalWorkArtifactCode),
    RecordNamespaceSynchronization,
}

impl PhysicalWorkObligationOperationCode {
    pub(super) const fn decode(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::ArtifactRangeRead),
            2 => Some(Self::ArtifactRangeWrite),
            3 => Some(Self::ArtifactPublication),
            4 => Some(Self::ArtifactMetadataRead),
            5 => Some(Self::WalAppend),
            6 => Some(Self::DurabilityBarrier),
            7 => Some(Self::CheckpointCapture),
            8 => Some(Self::WalReclamation),
            9 => Some(Self::RootPublication),
            _ => None,
        }
    }
}

pub(super) fn artifact_parts(artifact: PhysicalWorkArtifactCode) -> (u8, u64, u64) {
    match artifact {
        PhysicalWorkArtifactCode::BootstrapCatalog => (1, 0, 0),
        PhysicalWorkArtifactCode::CatalogCandidate { publication } => (2, publication, 0),
        PhysicalWorkArtifactCode::RootManifest { generation } => (3, generation, 0),
        PhysicalWorkArtifactCode::RootRoutingBlock { generation, block } => (4, generation, block),
        PhysicalWorkArtifactCode::Segment {
            segment,
            generation,
        } => (5, segment, generation),
        PhysicalWorkArtifactCode::SegmentManifest {
            segment,
            generation,
        } => (6, segment, generation),
        PhysicalWorkArtifactCode::SegmentMembershipBlock { generation, block } => {
            (7, generation, block)
        }
        PhysicalWorkArtifactCode::Extent { extent, generation } => (8, extent, generation),
        PhysicalWorkArtifactCode::ExtentManifest { extent, generation } => (9, extent, generation),
        PhysicalWorkArtifactCode::FreeSpaceManifest { generation } => (10, generation, 0),
        PhysicalWorkArtifactCode::FreeSpaceMembershipBlock { generation, block } => {
            (11, generation, block)
        }
        PhysicalWorkArtifactCode::CurrentRootSelector => (12, 0, 0),
        PhysicalWorkArtifactCode::PreviousRootSelector => (13, 0, 0),
        PhysicalWorkArtifactCode::RootSelectorCandidate {
            role: 1,
            publication,
        } => (14, publication, 0),
        PhysicalWorkArtifactCode::RootSelectorCandidate {
            role: 2,
            publication,
        } => (15, publication, 0),
        PhysicalWorkArtifactCode::RootSelectorCandidate { .. } => (0, 0, 0),
    }
}

pub(super) fn decode_artifact(
    tag: u8,
    first: u64,
    second: u64,
) -> Option<PhysicalWorkArtifactCode> {
    match tag {
        1 if first == 0 && second == 0 => Some(PhysicalWorkArtifactCode::BootstrapCatalog),
        2 if second == 0 => Some(PhysicalWorkArtifactCode::CatalogCandidate { publication: first }),
        3 if second == 0 => Some(PhysicalWorkArtifactCode::RootManifest { generation: first }),
        4 => Some(PhysicalWorkArtifactCode::RootRoutingBlock {
            generation: first,
            block: second,
        }),
        5 => Some(PhysicalWorkArtifactCode::Segment {
            segment: first,
            generation: second,
        }),
        6 => Some(PhysicalWorkArtifactCode::SegmentManifest {
            segment: first,
            generation: second,
        }),
        7 => Some(PhysicalWorkArtifactCode::SegmentMembershipBlock {
            generation: first,
            block: second,
        }),
        8 => Some(PhysicalWorkArtifactCode::Extent {
            extent: first,
            generation: second,
        }),
        9 => Some(PhysicalWorkArtifactCode::ExtentManifest {
            extent: first,
            generation: second,
        }),
        10 if second == 0 => {
            Some(PhysicalWorkArtifactCode::FreeSpaceManifest { generation: first })
        }
        11 => Some(PhysicalWorkArtifactCode::FreeSpaceMembershipBlock {
            generation: first,
            block: second,
        }),
        12 if first == 0 && second == 0 => Some(PhysicalWorkArtifactCode::CurrentRootSelector),
        13 if first == 0 && second == 0 => Some(PhysicalWorkArtifactCode::PreviousRootSelector),
        14 if second == 0 => Some(PhysicalWorkArtifactCode::RootSelectorCandidate {
            role: 1,
            publication: first,
        }),
        15 if second == 0 => Some(PhysicalWorkArtifactCode::RootSelectorCandidate {
            role: 2,
            publication: first,
        }),
        _ => None,
    }
}
