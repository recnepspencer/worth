use std::collections::BTreeSet;

use crate::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, DurableInlineRecordPlacement,
    ExtentArenaId, ExtentArenaRange, ExtentChunkCoordinate, PersistedPhysicalDataFrameSubject,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageId, PhysicalRecordSlot, PhysicalSegmentId, RecordArtifactFile,
    RecordFrameCoordinate, RecordSegmentPageManifestEntry,
};

mod codec;
mod root_state;
mod source_copy;
pub use root_state::{PersistedInlineSegmentAllocation, PersistedPhysicalRecoveryRootState};
pub use source_copy::PersistedExtentCopyRecipe;

const DOMAIN: &[u8] = b"store.physical.recovery-projection.v5";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedPhysicalRecoveryProjection {
    source_root_generation: u64,
    root_state: PersistedPhysicalRecoveryRootState,
    record_identities: Box<[PersistedRecordIdentity]>,
    payload: PersistedPhysicalRecoveryPayload,
    placements: Box<[CurrentPhysicalRecordPlacement]>,
    segment_updates: Box<[RecordSegmentPageManifestEntry]>,
    manifests: Box<[PersistedPhysicalRecoveryManifest]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistedPhysicalRecoveryPayload {
    Frames(Box<[PersistedPhysicalRecoveryFrame]>),
    SourceCopy(PersistedExtentCopyRecipe),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedPhysicalRecoveryFrame {
    subject: PersistedPhysicalDataFrameSubject,
    coordinate: RecordFrameCoordinate,
    bytes: Box<[u8]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedPhysicalRecoveryManifest {
    coordinate: RecordFrameCoordinate,
    bytes: Box<[u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryProjectionDenial {
    Malformed,
    EntryLimit,
    InvalidFrame,
    InvalidPlacement,
    InvalidSegmentUpdate,
    InvalidManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecoveryProjectionDecodeLimits {
    pub frames: u64,
    pub record_identities: u64,
    pub placements: u64,
    pub segment_updates: u64,
    pub manifests: u64,
    pub total_entries: u64,
    pub inline_allocations: u64,
}

impl PersistedPhysicalRecoveryProjection {
    pub fn new(
        source_root_generation: u64,
        root_state: PersistedPhysicalRecoveryRootState,
        record_identities: Vec<PersistedRecordIdentity>,
        frames: Vec<PersistedPhysicalRecoveryFrame>,
        placements: Vec<CurrentPhysicalRecordPlacement>,
        segment_updates: Vec<RecordSegmentPageManifestEntry>,
        manifests: Vec<PersistedPhysicalRecoveryManifest>,
    ) -> Option<Self> {
        (source_root_generation != 0
            && !record_identities.is_empty()
            && !frames.is_empty()
            && unique(record_identities.iter().copied())
            && unique(frames.iter().map(|frame| (frame.subject, frame.coordinate)))
            && strictly_ordered(placements.iter().map(|placement| placement.record()))
            && strictly_ordered(
                segment_updates
                    .iter()
                    .map(|entry| (entry.page_cell().segment_id().get(), entry.page().get())),
            )
            && strictly_ordered(manifests.iter().map(|manifest| manifest.coordinate)))
        .then_some(Self {
            source_root_generation,
            root_state,
            record_identities: record_identities.into_boxed_slice(),
            payload: PersistedPhysicalRecoveryPayload::Frames(frames.into_boxed_slice()),
            placements: placements.into_boxed_slice(),
            segment_updates: segment_updates.into_boxed_slice(),
            manifests: manifests.into_boxed_slice(),
        })
    }

    pub const fn source_root_generation(&self) -> u64 {
        self.source_root_generation
    }
    pub const fn root_state(&self) -> &PersistedPhysicalRecoveryRootState {
        &self.root_state
    }
    pub fn record_identities(&self) -> &[PersistedRecordIdentity] {
        &self.record_identities
    }
    pub fn payload(&self) -> &PersistedPhysicalRecoveryPayload {
        &self.payload
    }
    pub fn frames(&self) -> Option<&[PersistedPhysicalRecoveryFrame]> {
        match &self.payload {
            PersistedPhysicalRecoveryPayload::Frames(frames) => Some(frames),
            PersistedPhysicalRecoveryPayload::SourceCopy(_) => None,
        }
    }
    pub fn placements(&self) -> &[CurrentPhysicalRecordPlacement] {
        &self.placements
    }
    pub fn segment_updates(&self) -> &[RecordSegmentPageManifestEntry] {
        &self.segment_updates
    }
    pub fn manifests(&self) -> &[PersistedPhysicalRecoveryManifest] {
        &self.manifests
    }
}

fn unique<T: Ord>(values: impl Iterator<Item = T>) -> bool {
    let mut seen = BTreeSet::new();
    values.into_iter().all(|value| seen.insert(value))
}

fn strictly_ordered<T: Ord>(values: impl Iterator<Item = T>) -> bool {
    let values = values.collect::<Vec<_>>();
    values.windows(2).all(|pair| pair[0] < pair[1])
}

impl PersistedPhysicalRecoveryFrame {
    pub fn new(
        subject: PersistedPhysicalDataFrameSubject,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
    ) -> Option<Self> {
        (bytes.len() == coordinate.length() as usize && subject_matches(subject, coordinate))
            .then_some(Self {
                subject,
                coordinate,
                bytes: bytes.into(),
            })
    }
    pub const fn subject(&self) -> PersistedPhysicalDataFrameSubject {
        self.subject
    }
    pub const fn coordinate(&self) -> RecordFrameCoordinate {
        self.coordinate
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl PersistedPhysicalRecoveryManifest {
    pub fn new(coordinate: RecordFrameCoordinate, bytes: &[u8]) -> Option<Self> {
        (matches!(
            coordinate.artifact(),
            RecordArtifactFile::ExtentArena { .. }
        ) && coordinate.length() as usize == bytes.len())
        .then_some(Self {
            coordinate,
            bytes: bytes.into(),
        })
    }
    pub const fn artifact(&self) -> RecordArtifactFile {
        self.coordinate.artifact()
    }
    pub const fn coordinate(&self) -> RecordFrameCoordinate {
        self.coordinate
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

fn subject_matches(
    subject: PersistedPhysicalDataFrameSubject,
    coordinate: RecordFrameCoordinate,
) -> bool {
    match (subject, coordinate.artifact()) {
        (
            PersistedPhysicalDataFrameSubject::InlinePage(page),
            RecordArtifactFile::Segment {
                segment,
                generation: _,
            },
        ) => page.segment_id().get() == segment,
        (
            PersistedPhysicalDataFrameSubject::ExtentChunk(_),
            RecordArtifactFile::ExtentArena { arena },
        ) => arena != 0,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame, PhysicalGeneration,
        PhysicalGenerationAuthority, PhysicalPageId, PhysicalSegmentId, RecordArtifactFile,
        RecordFrameCoordinate,
    };

    #[test]
    fn inline_recovery_frame_keeps_page_and_segment_generations_independent() {
        let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
        let page = authority
            .page_cell(
                PhysicalSegmentId::from_raw(7).unwrap(),
                PhysicalPageId::from_raw(3).unwrap(),
            )
            .with_page_generation(PhysicalGeneration::from_raw(2).unwrap());
        let coordinate = RecordFrameCoordinate::new(
            RecordArtifactFile::Segment {
                segment: 7,
                generation: 99,
            },
            0,
            4,
        )
        .unwrap();

        assert!(PersistedPhysicalRecoveryFrame::new(
            PersistedPhysicalDataFrameSubject::InlinePage(page),
            coordinate,
            b"page",
        )
        .is_some());

        let foreign_segment = RecordFrameCoordinate::new(
            RecordArtifactFile::Segment {
                segment: 8,
                generation: 99,
            },
            0,
            4,
        )
        .unwrap();
        assert!(PersistedPhysicalRecoveryFrame::new(
            PersistedPhysicalDataFrameSubject::InlinePage(page),
            foreign_segment,
            b"page",
        )
        .is_none());
    }
}
