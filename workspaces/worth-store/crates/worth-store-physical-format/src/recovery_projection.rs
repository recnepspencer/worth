use crate::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, DurableInlineRecordPlacement,
    ExtentArenaId, ExtentArenaRange, ExtentChunkCoordinate, PersistedPhysicalDataFrameSubject,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageId, PhysicalRecordSlot, PhysicalSegmentId, RecordArtifactFile,
    RecordFrameCoordinate, RecordSegmentPageManifestEntry,
};

mod codec;
pub(crate) mod decode_storage;
mod frame;
mod head_effect;
mod operation;
mod release_head_retirement;
mod release_head_tree_claim;
mod released_directory_replacement;
mod retained_storage;
mod root_state;
mod source_copy;
pub use decode_storage::{PhysicalRecoveryDecodeFailure, PhysicalRecoveryDecodeStorage};
pub use head_effect::PersistedReleaseCustodyHeadEffectV1;
pub use operation::{
    PersistedBlobSemanticRecordBinding, PersistedDerivedDirectoryRecordBinding,
    PersistedPhysicalRecoveryOperation,
};
pub use release_head_retirement::PersistedTerminalReleaseHeadRetirementV1;
pub use release_head_tree_claim::PersistedReleaseHeadTreeClaim;
pub use released_directory_replacement::PersistedReleasedDirectoryReplacementV1;
pub use root_state::{PersistedInlineSegmentAllocation, PersistedPhysicalRecoveryRootState};
pub use source_copy::PersistedExtentCopyRecipe;

const PROJECTION_DOMAIN_PREFIX: &[u8] = b"store.physical.recovery-projection.v";
const CURRENT_RECOVERY_PROJECTION_DOMAIN: &[u8] = b"store.physical.recovery-projection.v16";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedDerivedDirectoryRetirement {
    expected_previous: Option<crate::DerivedFamilyRootDirectoryBinding>,
    dropped_records: Box<[PersistedRecordIdentity]>,
}

impl PersistedDerivedDirectoryRetirement {
    pub fn new(
        expected_previous: Option<crate::DerivedFamilyRootDirectoryBinding>,
        mut dropped_records: Vec<PersistedRecordIdentity>,
    ) -> Option<Self> {
        dropped_records.sort_unstable();
        (dropped_records.windows(2).all(|pair| pair[0] != pair[1])
            && expected_previous
                .is_none_or(|previous| dropped_records.contains(&previous.directory_record())))
        .then_some(Self {
            expected_previous,
            dropped_records: dropped_records.into_boxed_slice(),
        })
    }

    pub const fn expected_previous(&self) -> Option<crate::DerivedFamilyRootDirectoryBinding> {
        self.expected_previous
    }

    pub fn dropped_records(&self) -> &[PersistedRecordIdentity] {
        &self.dropped_records
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedPhysicalRecoveryProjection {
    source_root_generation: u64,
    root_state: PersistedPhysicalRecoveryRootState,
    record_identities: Box<[PersistedRecordIdentity]>,
    payload: PersistedPhysicalRecoveryPayload,
    operation: PersistedPhysicalRecoveryOperation,
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
    UnsupportedVersion(u16),
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
        Self::new_with_operation(
            source_root_generation,
            root_state,
            record_identities,
            frames,
            placements,
            segment_updates,
            manifests,
            PersistedPhysicalRecoveryOperation::None,
        )
    }

    pub fn new_with_operation(
        source_root_generation: u64,
        root_state: PersistedPhysicalRecoveryRootState,
        record_identities: Vec<PersistedRecordIdentity>,
        frames: Vec<PersistedPhysicalRecoveryFrame>,
        placements: Vec<CurrentPhysicalRecordPlacement>,
        segment_updates: Vec<RecordSegmentPageManifestEntry>,
        manifests: Vec<PersistedPhysicalRecoveryManifest>,
        operation: PersistedPhysicalRecoveryOperation,
    ) -> Option<Self> {
        Self::new_with_operation_in_storage(
            source_root_generation,
            root_state,
            record_identities,
            frames,
            placements,
            segment_updates,
            manifests,
            operation,
            &mut decode_storage::UnrestrictedDecodeStorage,
        )
        .ok()
    }

    fn new_with_operation_in_storage<S: PhysicalRecoveryDecodeStorage>(
        source_root_generation: u64,
        root_state: PersistedPhysicalRecoveryRootState,
        record_identities: Vec<PersistedRecordIdentity>,
        frames: Vec<PersistedPhysicalRecoveryFrame>,
        placements: Vec<CurrentPhysicalRecordPlacement>,
        segment_updates: Vec<RecordSegmentPageManifestEntry>,
        manifests: Vec<PersistedPhysicalRecoveryManifest>,
        operation: PersistedPhysicalRecoveryOperation,
        storage: &mut S,
    ) -> Result<Self, PhysicalRecoveryDecodeFailure<S::Denial>> {
        // A terminal head retirement is the one record-less member: it moves no
        // data, so it carries no record, frame, placement or inline delta.
        let record_less = operation.is_terminal_release_head_retired();
        (source_root_generation != 0
            && record_identities.is_empty() == record_less
            && frames.is_empty() == record_less
            && (!record_less
                || (segment_updates.is_empty()
                    && manifests.is_empty()
                    && root_state.inline_allocations().is_empty()
                    && root_state.last_inline_record().is_none()))
            && unique_in_storage(record_identities.iter().copied(), storage)?
            && unique_in_storage(
                frames.iter().map(|frame| (frame.subject, frame.coordinate)),
                storage,
            )?
            && strictly_ordered(placements.iter().map(|placement| placement.record()))
            && strictly_ordered(
                segment_updates
                    .iter()
                    .map(|entry| (entry.page_cell().segment_id().get(), entry.page().get())),
            )
            && strictly_ordered(manifests.iter().map(|manifest| manifest.coordinate))
            && operation.admits(source_root_generation, &record_identities, &placements))
        .then_some(Self {
            source_root_generation,
            root_state,
            record_identities: record_identities.into_boxed_slice(),
            payload: PersistedPhysicalRecoveryPayload::Frames(frames.into_boxed_slice()),
            operation,
            placements: placements.into_boxed_slice(),
            segment_updates: segment_updates.into_boxed_slice(),
            manifests: manifests.into_boxed_slice(),
        })
        .ok_or_else(|| PhysicalRecoveryProjectionDenial::Malformed.into())
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
    pub const fn operation(&self) -> &PersistedPhysicalRecoveryOperation {
        &self.operation
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

fn unique_in_storage<T: Ord, S: PhysicalRecoveryDecodeStorage>(
    values: impl ExactSizeIterator<Item = T>,
    storage: &mut S,
) -> Result<bool, PhysicalRecoveryDecodeFailure<S::Denial>> {
    let mut seen = decode_storage::reserve_vec(values.len(), storage)?;
    seen.extend(values);
    seen.sort_unstable();
    Ok(seen.windows(2).all(|pair| pair[0] != pair[1]))
}

fn strictly_ordered<T: Ord>(mut values: impl Iterator<Item = T>) -> bool {
    let Some(mut prior) = values.next() else {
        return true;
    };
    for value in values {
        if prior >= value {
            return false;
        }
        prior = value;
    }
    true
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
