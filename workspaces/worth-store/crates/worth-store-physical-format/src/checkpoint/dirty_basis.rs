use crate::{RecordArtifactFile, RecordFrameCoordinate};

use super::record::{read_u64, CheckpointStreamDecodeDenial};

pub(super) const DIRTY_BASIS_PAYLOAD_BYTES: usize = 48;
pub const CHECKPOINT_DIRTY_FRAME_RECORD_BYTES: usize = 68;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointDirtyFrameBasis {
    coordinate: RecordFrameCoordinate,
    dirty_generation: u64,
}

pub(super) fn encode_dirty_basis(
    basis: CheckpointDirtyFrameBasis,
) -> [u8; DIRTY_BASIS_PAYLOAD_BYTES] {
    let mut payload = [0; DIRTY_BASIS_PAYLOAD_BYTES];
    let (kind, first, second) = encode_artifact(basis.coordinate().artifact());
    payload[0] = kind;
    payload[8..16].copy_from_slice(&first.to_le_bytes());
    payload[16..24].copy_from_slice(&second.to_le_bytes());
    payload[24..32].copy_from_slice(&basis.coordinate().offset().to_le_bytes());
    payload[32..36].copy_from_slice(&basis.coordinate().length().to_le_bytes());
    payload[40..48].copy_from_slice(&basis.dirty_generation().to_le_bytes());
    payload
}

pub(super) fn decode_dirty_basis(
    payload: &[u8],
) -> Result<CheckpointDirtyFrameBasis, CheckpointStreamDecodeDenial> {
    if payload[1..8] != [0; 7] || payload[36..40] != [0; 4] {
        return Err(CheckpointStreamDecodeDenial::ReservedFieldNonZero);
    }
    let artifact = decode_artifact(payload[0], read_u64(payload, 8), read_u64(payload, 16))?;
    let coordinate = RecordFrameCoordinate::new(
        artifact,
        read_u64(payload, 24),
        u32::from_le_bytes(payload[32..36].try_into().unwrap()),
    )
    .ok_or(CheckpointStreamDecodeDenial::InvalidCoordinate)?;
    Ok(CheckpointDirtyFrameBasis::new(
        coordinate,
        read_u64(payload, 40),
    ))
}

fn encode_artifact(artifact: RecordArtifactFile) -> (u8, u64, u64) {
    match artifact {
        RecordArtifactFile::BootstrapCatalog => (1, 0, 0),
        RecordArtifactFile::CurrentRootSelector => (12, 0, 0),
        RecordArtifactFile::PreviousRootSelector => (13, 0, 0),
        RecordArtifactFile::RootSelectorCandidate { role, publication } => match role {
            crate::RootSelectorRole::Current => (14, publication, 0),
            crate::RootSelectorRole::Previous => (15, publication, 0),
        },
        RecordArtifactFile::CatalogCandidate { publication } => (2, publication, 0),
        RecordArtifactFile::RootManifest { generation } => (3, generation, 0),
        RecordArtifactFile::RootRoutingBlock { generation, block } => (4, generation, block),
        RecordArtifactFile::Segment {
            segment,
            generation,
        } => (5, segment, generation),
        RecordArtifactFile::SegmentManifest {
            segment,
            generation,
        } => (6, segment, generation),
        RecordArtifactFile::SegmentMembershipBlock { generation, block } => (7, generation, block),
        RecordArtifactFile::Extent { extent, generation } => (8, extent, generation),
        RecordArtifactFile::ExtentManifest { extent, generation } => (9, extent, generation),
        RecordArtifactFile::FreeSpaceManifest { generation } => (10, generation, 0),
        RecordArtifactFile::FreeSpaceMembershipBlock { generation, block } => {
            (11, generation, block)
        }
    }
}

fn decode_artifact(
    kind: u8,
    first: u64,
    second: u64,
) -> Result<RecordArtifactFile, CheckpointStreamDecodeDenial> {
    let artifact = match kind {
        1 if first == 0 && second == 0 => RecordArtifactFile::BootstrapCatalog,
        2 if second == 0 => RecordArtifactFile::CatalogCandidate { publication: first },
        3 if second == 0 => RecordArtifactFile::RootManifest { generation: first },
        4 => RecordArtifactFile::RootRoutingBlock {
            generation: first,
            block: second,
        },
        5 => RecordArtifactFile::Segment {
            segment: first,
            generation: second,
        },
        6 => RecordArtifactFile::SegmentManifest {
            segment: first,
            generation: second,
        },
        7 => RecordArtifactFile::SegmentMembershipBlock {
            generation: first,
            block: second,
        },
        8 => RecordArtifactFile::Extent {
            extent: first,
            generation: second,
        },
        9 => RecordArtifactFile::ExtentManifest {
            extent: first,
            generation: second,
        },
        10 if second == 0 => RecordArtifactFile::FreeSpaceManifest { generation: first },
        11 => RecordArtifactFile::FreeSpaceMembershipBlock {
            generation: first,
            block: second,
        },
        12 if first == 0 && second == 0 => RecordArtifactFile::CurrentRootSelector,
        13 if first == 0 && second == 0 => RecordArtifactFile::PreviousRootSelector,
        14 if second == 0 => RecordArtifactFile::RootSelectorCandidate {
            role: crate::RootSelectorRole::Current,
            publication: first,
        },
        15 if second == 0 => RecordArtifactFile::RootSelectorCandidate {
            role: crate::RootSelectorRole::Previous,
            publication: first,
        },
        _ => return Err(CheckpointStreamDecodeDenial::InvalidArtifactKind(kind)),
    };
    Ok(artifact)
}

impl CheckpointDirtyFrameBasis {
    pub fn decode_record(record: &[u8]) -> Result<Self, CheckpointStreamDecodeDenial> {
        let payload = super::record::decode_record(
            record,
            super::record::DIRTY_BASIS_KIND,
            DIRTY_BASIS_PAYLOAD_BYTES,
        )?;
        decode_dirty_basis(payload)
    }

    pub const fn new(coordinate: RecordFrameCoordinate, dirty_generation: u64) -> Self {
        Self {
            coordinate,
            dirty_generation,
        }
    }

    pub const fn coordinate(self) -> RecordFrameCoordinate {
        self.coordinate
    }

    pub const fn dirty_generation(self) -> u64 {
        self.dirty_generation
    }
}
