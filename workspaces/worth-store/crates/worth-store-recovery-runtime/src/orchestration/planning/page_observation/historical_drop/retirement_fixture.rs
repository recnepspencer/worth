//! Members that wrote and retired the records of one inline page, built from
//! real projections and real decoded WAL targets.

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, ExtentArenaId, ExtentArenaRange,
    ExtentChunkCoordinate, PersistedBlobSemanticRecordBinding,
    PersistedDerivedDirectoryRecordBinding, PersistedDerivedDirectoryRetirement,
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryOperation, PersistedPhysicalRecoveryProjection,
    PersistedPhysicalRecoveryRootState, PersistedRecordIdentity, PhysicalExtentId,
    PhysicalGeneration, PhysicalGenerationAuthority, RecordArtifactFile, RecordFrameCoordinate,
};
use worth_store_recovery_physics::{
    PhysicalRedoGroupBinding, PhysicalRedoTarget, RecoveryOperationFate,
};

use super::super::inline_image_fixture::{inline_image, inline_image_beside_stray};
use super::ordered_retirements::StepIdentity;

/// The page every fixture image belongs to.
pub(super) const PAGE: u64 = 2;
const SEGMENT: u64 = 1;

/// A record of the fixture page.
pub(super) fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

/// The identity of the member seeded by `seed`, and of an edge that binds it.
pub(super) fn step(seed: u8) -> StepIdentity {
    StepIdentity {
        operation: [seed; 32],
        group: PhysicalRedoGroupBinding::new([seed; 32], [seed; 32], 1, 1, [seed; 32]).unwrap(),
        fate: RecoveryOperationFate::Indeterminate,
        redo_sha256: [seed; 32],
    }
}

/// A member that wrote the image of the page at `generation` holding records
/// `1..=total`, and the WAL target of that image.
pub(super) fn image(
    generation: u64,
    total: u64,
) -> (PersistedPhysicalRecoveryProjection, PhysicalRedoTarget) {
    let records = (1..=total).map(record).collect::<Vec<_>>();
    inline_image(
        (SEGMENT, PAGE, generation),
        (SEGMENT, generation),
        &records,
        generation as u8,
    )
}

/// As [`image`], from a member that also placed record `stray` on generation
/// `stray_generation` of the page, which is not the image it framed.
pub(super) fn image_beside_stray(
    generation: u64,
    total: u64,
    (stray_generation, stray): (u64, u64),
) -> (PersistedPhysicalRecoveryProjection, PhysicalRedoTarget) {
    let records = (1..=total).map(record).collect::<Vec<_>>();
    inline_image_beside_stray(
        (SEGMENT, PAGE, generation),
        (SEGMENT, generation),
        &records,
        (record(stray), stray_generation),
        generation as u8,
    )
}

/// A member whose derived-directory append retires these page records.
pub(super) fn retirement(dropped: &[u64]) -> PersistedPhysicalRecoveryProjection {
    let directory = PersistedRecordIdentity::new([7; 16], 10).unwrap();
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(5).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    let range = ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 0, 53_248).unwrap();
    let placement =
        DurableExtentRecordPlacement::legacy_unknown(directory, cell, 1, range).unwrap();
    let coordinate =
        RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 2 }, 0, 1).unwrap();
    let subject = PersistedPhysicalDataFrameSubject::ExtentChunk(
        ExtentChunkCoordinate::new(directory, cell, 1, 0, 1).unwrap(),
    );
    let root = PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap();
    PersistedPhysicalRecoveryProjection::new_with_operation(
        11,
        root,
        vec![directory],
        vec![PersistedPhysicalRecoveryFrame::new(subject, coordinate, &[3]).unwrap()],
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        vec![],
        vec![],
        PersistedPhysicalRecoveryOperation::DerivedDirectory {
            binding: PersistedDerivedDirectoryRecordBinding::new(
                PersistedBlobSemanticRecordBinding::new(directory, [5; 32], 12).unwrap(),
                None,
            ),
            retirement: Some(
                PersistedDerivedDirectoryRetirement::new(
                    None,
                    dropped.iter().copied().map(record).collect(),
                )
                .unwrap(),
            ),
        },
    )
    .unwrap()
}
