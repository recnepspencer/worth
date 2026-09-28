use super::{PhysicalRedoExtentCoordinate, PhysicalRedoTarget, PhysicalRedoTargetIdentity};

impl PhysicalRedoTarget {
    pub const fn identity(&self) -> PhysicalRedoTargetIdentity {
        self.identity
    }
    pub const fn artifact_offset(&self) -> u64 {
        self.artifact_offset
    }
    pub const fn extent_coordinate(&self) -> Option<PhysicalRedoExtentCoordinate> {
        self.extent_coordinate
    }
    pub const fn artifact(&self) -> worth_store_physical_format::RecordArtifactFile {
        self.artifact
    }
    pub const fn artifact_length(&self) -> u32 {
        self.artifact_length
    }
    pub const fn resulting_digest(&self) -> [u8; 32] {
        self.resulting_digest
    }

    pub(super) fn canonical_order(&self) -> PhysicalRedoTargetCanonicalOrder {
        match (self.identity, self.extent_coordinate) {
            (
                PhysicalRedoTargetIdentity::InlinePage {
                    segment,
                    page,
                    generation,
                },
                None,
            ) => PhysicalRedoTargetCanonicalOrder::Inline {
                segment,
                page,
                generation,
                artifact_segment: match self.artifact {
                    worth_store_physical_format::RecordArtifactFile::Segment {
                        segment, ..
                    } => segment,
                    _ => unreachable!("decoded inline targets retain a segment artifact"),
                },
                artifact_generation: match self.artifact {
                    worth_store_physical_format::RecordArtifactFile::Segment {
                        generation, ..
                    } => generation,
                    _ => unreachable!("decoded inline targets retain a segment artifact"),
                },
                artifact_offset: self.artifact_offset,
                artifact_length: self.artifact_length,
                resulting_digest: self.resulting_digest,
            },
            (
                PhysicalRedoTargetIdentity::ExtentChunk {
                    extent,
                    generation,
                    chunk,
                },
                Some(coordinate),
            ) => PhysicalRedoTargetCanonicalOrder::Extent {
                allocation_epoch: coordinate.allocation_epoch,
                record_ordinal: coordinate.record_ordinal,
                extent,
                generation,
                logical_bytes: coordinate.logical_bytes,
                logical_offset: coordinate.logical_offset,
                chunk,
                artifact_arena: match self.artifact {
                    worth_store_physical_format::RecordArtifactFile::ExtentArena { arena } => arena,
                    _ => unreachable!("decoded extent targets retain an extent artifact"),
                },
                artifact_offset: self.artifact_offset,
                artifact_length: self.artifact_length,
                resulting_digest: self.resulting_digest,
            },
            _ => unreachable!("decoded redo targets keep exact kind coordinates"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum PhysicalRedoTargetCanonicalOrder {
    Inline {
        segment: u64,
        page: u64,
        generation: u64,
        artifact_segment: u64,
        artifact_generation: u64,
        artifact_offset: u64,
        artifact_length: u32,
        resulting_digest: [u8; 32],
    },
    Extent {
        allocation_epoch: [u8; 16],
        record_ordinal: u64,
        extent: u64,
        generation: u64,
        logical_bytes: u64,
        logical_offset: u64,
        chunk: u32,
        artifact_arena: u64,
        artifact_offset: u64,
        artifact_length: u32,
        resulting_digest: [u8; 32],
    },
}

impl PhysicalRedoExtentCoordinate {
    pub const fn allocation_epoch(self) -> [u8; 16] {
        self.allocation_epoch
    }
    pub const fn record_ordinal(self) -> u64 {
        self.record_ordinal
    }
    pub const fn logical_bytes(self) -> u64 {
        self.logical_bytes
    }
    pub const fn logical_offset(self) -> u64 {
        self.logical_offset
    }
}
