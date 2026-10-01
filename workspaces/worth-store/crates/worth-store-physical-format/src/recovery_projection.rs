use std::collections::BTreeSet;

use crate::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, DurableInlineRecordPlacement,
    ExtentArenaId, ExtentArenaRange, ExtentChunkCoordinate, PersistedPhysicalDataFrameSubject,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageId, PhysicalRecordSlot, PhysicalSegmentId, RecordArtifactFile,
    RecordFrameCoordinate, RecordSegmentPageManifestEntry, ReleaseCustodyHeadMutationV1,
};

mod blob_semantic;
mod codec;
mod frame;
mod head_effect;
mod retained_storage;
mod root_state;
mod source_copy;
pub use blob_semantic::{
    PersistedBlobSemanticRecordBinding, PersistedDerivedDirectoryRecordBinding,
    PersistedPhysicalRecoveryBlobSemantic,
};
pub use head_effect::PersistedReleaseCustodyHeadEffectV1;
pub use root_state::{PersistedInlineSegmentAllocation, PersistedPhysicalRecoveryRootState};
pub use source_copy::PersistedExtentCopyRecipe;

const V5_DOMAIN: &[u8] = b"store.physical.recovery-projection.v5";
const V6_DOMAIN: &[u8] = b"store.physical.recovery-projection.v6";
const V7_DOMAIN: &[u8] = b"store.physical.recovery-projection.v7";
const V8_DOMAIN: &[u8] = b"store.physical.recovery-projection.v8";
const V9_DOMAIN: &[u8] = b"store.physical.recovery-projection.v9";
const V10_DOMAIN: &[u8] = b"store.physical.recovery-projection.v10";
const V11_DOMAIN: &[u8] = b"store.physical.recovery-projection.v11";
const V12_DOMAIN: &[u8] = b"store.physical.recovery-projection.v12";
const V13_DOMAIN: &[u8] = b"store.physical.recovery-projection.v13";
const V14_DOMAIN: &[u8] = b"store.physical.recovery-projection.v14";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryProjectionVersion {
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedDerivedDirectoryRetirement {
    expected_previous: Option<crate::DerivedFamilyRootDirectoryBinding>,
    dropped_records: Box<[PersistedRecordIdentity]>,
}

impl PersistedDerivedDirectoryRetirement {
    pub const fn expected_previous(&self) -> Option<crate::DerivedFamilyRootDirectoryBinding> {
        self.expected_previous
    }

    pub fn dropped_records(&self) -> &[PersistedRecordIdentity] {
        &self.dropped_records
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedPhysicalRecoveryProjection {
    version: RecoveryProjectionVersion,
    source_root_generation: u64,
    root_state: PersistedPhysicalRecoveryRootState,
    record_identities: Box<[PersistedRecordIdentity]>,
    payload: PersistedPhysicalRecoveryPayload,
    blob_semantic: PersistedPhysicalRecoveryBlobSemantic,
    derived_retirement: Option<PersistedDerivedDirectoryRetirement>,
    release_head_effect: Option<PersistedReleaseCustodyHeadEffectV1>,
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
        Self::new_with_blob_semantic(
            source_root_generation,
            root_state,
            record_identities,
            frames,
            placements,
            segment_updates,
            manifests,
            PersistedPhysicalRecoveryBlobSemantic::None,
        )
    }

    pub fn new_with_blob_semantic(
        source_root_generation: u64,
        root_state: PersistedPhysicalRecoveryRootState,
        record_identities: Vec<PersistedRecordIdentity>,
        frames: Vec<PersistedPhysicalRecoveryFrame>,
        placements: Vec<CurrentPhysicalRecordPlacement>,
        segment_updates: Vec<RecordSegmentPageManifestEntry>,
        manifests: Vec<PersistedPhysicalRecoveryManifest>,
        blob_semantic: PersistedPhysicalRecoveryBlobSemantic,
    ) -> Option<Self> {
        let classified_routes = placements
            .iter()
            .any(|placement| !placement.route_metadata().is_legacy_unknown());
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
            && strictly_ordered(manifests.iter().map(|manifest| manifest.coordinate))
            && blob_semantic.admits(source_root_generation, &record_identities, &placements))
        .then_some(Self {
            version: if classified_routes {
                RecoveryProjectionVersion::V13
            } else if matches!(
                blob_semantic,
                PersistedPhysicalRecoveryBlobSemantic::DedupeQuarantined(_)
            ) {
                RecoveryProjectionVersion::V11
            } else if matches!(
                blob_semantic,
                PersistedPhysicalRecoveryBlobSemantic::ChunkReused(_)
            ) {
                RecoveryProjectionVersion::V9
            } else if let PersistedPhysicalRecoveryBlobSemantic::DerivedDirectory(directory) =
                blob_semantic
            {
                if directory.indexed_through_quarantine().is_some() {
                    RecoveryProjectionVersion::V12
                } else {
                    RecoveryProjectionVersion::V8
                }
            } else if matches!(
                blob_semantic,
                PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(_)
            ) {
                RecoveryProjectionVersion::V7
            } else {
                RecoveryProjectionVersion::V6
            },
            source_root_generation,
            root_state,
            record_identities: record_identities.into_boxed_slice(),
            payload: PersistedPhysicalRecoveryPayload::Frames(frames.into_boxed_slice()),
            blob_semantic,
            derived_retirement: None,
            release_head_effect: None,
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
    pub const fn blob_semantic(&self) -> PersistedPhysicalRecoveryBlobSemantic {
        self.blob_semantic
    }
    pub fn derived_retirement(&self) -> Option<&PersistedDerivedDirectoryRetirement> {
        self.derived_retirement.as_ref()
    }
    pub fn release_head_effect(&self) -> Option<&PersistedReleaseCustodyHeadEffectV1> {
        self.release_head_effect.as_ref()
    }
    pub fn with_release_head_upsert(
        mut self,
        effect: PersistedReleaseCustodyHeadEffectV1,
    ) -> Option<Self> {
        let ReleaseCustodyHeadMutationV1::Upsert { next, .. } = effect.mutation() else {
            return None;
        };
        if !matches!(
            self.blob_semantic,
            PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding)
                if self.record_identities.as_ref() == [binding.record()]
                    && next.descriptor_record() == binding.record()
                    && next.source_root_generation() == self.source_root_generation
                    && self.source_root_generation.checked_add(1)
                        == Some(binding.candidate_root_generation())
        ) || self.derived_retirement.is_some()
            || self.release_head_effect.is_some()
        {
            return None;
        }
        self.version = RecoveryProjectionVersion::V14;
        self.release_head_effect = Some(effect);
        Some(self)
    }
    pub fn with_derived_retirement(
        mut self,
        expected_previous: Option<crate::DerivedFamilyRootDirectoryBinding>,
        mut dropped_records: Vec<PersistedRecordIdentity>,
    ) -> Option<Self> {
        if !matches!(
            self.blob_semantic,
            PersistedPhysicalRecoveryBlobSemantic::DerivedDirectory(_)
        ) {
            return None;
        }
        dropped_records.sort_unstable();
        if dropped_records.windows(2).any(|pair| pair[0] == pair[1])
            || expected_previous
                .is_some_and(|previous| !dropped_records.contains(&previous.directory_record()))
            || dropped_records
                .iter()
                .any(|record| self.record_identities.contains(record))
        {
            return None;
        }
        self.version = if self.version == RecoveryProjectionVersion::V13 {
            RecoveryProjectionVersion::V13
        } else if matches!(self.blob_semantic, PersistedPhysicalRecoveryBlobSemantic::DerivedDirectory(directory) if directory.indexed_through_quarantine().is_some())
        {
            RecoveryProjectionVersion::V12
        } else {
            RecoveryProjectionVersion::V10
        };
        self.derived_retirement = Some(PersistedDerivedDirectoryRetirement {
            expected_previous,
            dropped_records: dropped_records.into_boxed_slice(),
        });
        Some(self)
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
