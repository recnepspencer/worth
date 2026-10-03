//! A retained WAL image is historically retired only when the ordered history
//! holds its own publishing edge and exactly one later ordinary edge whose
//! admitted derived-directory retirement drops that exact record.

use super::*;
use crate::redo_replay::plan::historical_drop::{anchored_root_identity, RootAnchors};
use crate::redo_replay::plan::historical_retired::{unrouted_under_anchored_root, EdgeIdentity};
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, ExtentArenaId, ExtentArenaRange,
    ExtentChunkCoordinate, PersistedBlobSemanticRecordBinding,
    PersistedDerivedDirectoryRecordBinding, PersistedDerivedDirectoryRetirement,
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryOperation, PersistedPhysicalRecoveryProjection,
    PersistedPhysicalRecoveryRootState, PersistedRecordIdentity, PhysicalExtentId,
    PhysicalGeneration, PhysicalGenerationAuthority, RecordFrameCoordinate,
};

const OLD: [u8; 32] = [0x31; 32];
const RETIRING: [u8; 32] = [0x52; 32];
const EXTENT: (u64, u64) = (3, 4);

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
}

fn projection(
    placed: PersistedRecordIdentity,
    extent: (u64, u64),
    operation: PersistedPhysicalRecoveryOperation,
) -> PersistedPhysicalRecoveryProjection {
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(extent.0).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(extent.1).unwrap());
    let range = ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 0, 53_248).unwrap();
    let placement = DurableExtentRecordPlacement::legacy_unknown(placed, cell, 1, range).unwrap();
    let coordinate =
        RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 2 }, 0, 1).unwrap();
    let subject = PersistedPhysicalDataFrameSubject::ExtentChunk(
        ExtentChunkCoordinate::new(placed, cell, 1, 0, 1).unwrap(),
    );
    let root = PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap();
    PersistedPhysicalRecoveryProjection::new_with_operation(
        11,
        root,
        vec![placed],
        vec![PersistedPhysicalRecoveryFrame::new(subject, coordinate, &[3]).unwrap()],
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        vec![],
        vec![],
        operation,
    )
    .unwrap()
}

fn retirement(dropped: PersistedRecordIdentity) -> PersistedPhysicalRecoveryOperation {
    let directory = record(10);
    PersistedPhysicalRecoveryOperation::DerivedDirectory {
        binding: PersistedDerivedDirectoryRecordBinding::new(
            PersistedBlobSemanticRecordBinding::new(directory, [5; 32], 12).unwrap(),
            None,
        ),
        retirement: Some(PersistedDerivedDirectoryRetirement::new(None, vec![dropped]).unwrap()),
    }
}

fn group(seed: u8) -> PhysicalRedoGroupBinding {
    PhysicalRedoGroupBinding::new([seed; 32], [seed; 32], 1, 1, [seed; 32]).unwrap()
}

fn member(
    operation: [u8; 32],
    projection: PersistedPhysicalRecoveryProjection,
) -> AdmittedPhysicalRedoMember {
    AdmittedPhysicalRedoMember {
        lsn_range: range(),
        operation,
        group: group(operation[0]),
        fate: RecoveryOperationFate::Indeterminate,
        records: Box::new([]),
        projection,
        canonical_redo_sha256: operation,
        inline_frames: Box::new([]),
    }
}

fn members(retired: PersistedRecordIdentity) -> AdmittedPhysicalRedoMembers {
    AdmittedPhysicalRedoMembers {
        scratch_bytes: 0,
        members: vec![
            member(
                OLD,
                projection(record(9), EXTENT, PersistedPhysicalRecoveryOperation::None),
            ),
            member(
                RETIRING,
                projection(record(10), (5, 1), retirement(retired)),
            ),
        ]
        .into_boxed_slice(),
        group_allocations: BTreeMap::new(),
        rewrites: Box::new([]),
        rewrite_admissions: Box::new([]),
        source_copies: Box::new([]),
    }
}

fn edge(operation: [u8; 32], ordinary: bool) -> EdgeIdentity {
    EdgeIdentity {
        operation,
        group: group(operation[0]),
        fate: RecoveryOperationFate::Indeterminate,
        redo_sha256: operation,
        ordinary,
    }
}

fn retiring(
    members: &AdmittedPhysicalRedoMembers,
    extent: (u64, u64),
    edges: &[EdgeIdentity],
) -> Option<[u8; 32]> {
    members.retiring_operation(OLD, record(9), extent, edges.iter().copied())
}

#[test]
fn later_ordinary_retirement_of_the_exact_record_is_the_only_witness() {
    let admitted = members(record(9));
    let ordered = [edge(OLD, false), edge(RETIRING, true)];
    assert_eq!(retiring(&admitted, EXTENT, &ordered), Some(RETIRING));

    // Missing retiring edge: the history never removed the record.
    assert_eq!(retiring(&admitted, EXTENT, &[edge(OLD, false)]), None);
    // Retirement ordered before the record's own publication.
    assert_eq!(
        retiring(&admitted, EXTENT, &[edge(RETIRING, true), edge(OLD, false)]),
        None
    );
    // A released edge carrying the retiring identity is not an ordinary step.
    assert_eq!(
        retiring(
            &admitted,
            EXTENT,
            &[edge(OLD, false), edge(RETIRING, false)]
        ),
        None
    );
    // Duplicate retiring edges are ambiguous.
    assert_eq!(
        retiring(
            &admitted,
            EXTENT,
            &[edge(OLD, false), edge(RETIRING, true), edge(RETIRING, true)]
        ),
        None
    );
    // The target names a different extent generation than its member placed.
    assert_eq!(retiring(&admitted, (3, 5), &ordered), None);
}

#[test]
fn forged_retiring_edge_cannot_bind_an_admitted_member() {
    let admitted = members(record(9));
    for forged in [
        EdgeIdentity {
            redo_sha256: [0xee; 32],
            ..edge(RETIRING, true)
        },
        EdgeIdentity {
            group: group(0x99),
            ..edge(RETIRING, true)
        },
        EdgeIdentity {
            fate: RecoveryOperationFate::AcknowledgedDurable,
            ..edge(RETIRING, true)
        },
    ] {
        assert_eq!(
            retiring(&admitted, EXTENT, &[edge(OLD, false), forged]),
            None
        );
    }
    // A genuine retirement that drops a different record is not a witness.
    let other = members(record(11));
    assert_eq!(
        retiring(&other, EXTENT, &[edge(OLD, false), edge(RETIRING, true)]),
        None
    );
}

#[test]
fn selected_root_that_still_routes_the_record_denies_the_witness() {
    const SELECTED: [u8; 32] = [0x64; 32];
    let routes = [record(3), record(9), record(12)];
    assert_eq!(
        unrouted_under_anchored_root(Some(SELECTED), routes.iter().copied(), record(9)),
        None,
        "a record the selected root still routes was never retired",
    );
    assert_eq!(
        unrouted_under_anchored_root(Some(SELECTED), routes.iter().copied(), record(10)),
        Some(SELECTED)
    );
    assert_eq!(
        unrouted_under_anchored_root(None, std::iter::empty(), record(10)),
        None,
        "an unanchored history mints no witness",
    );
}

#[test]
fn history_must_start_at_the_selected_checkpoint_and_end_at_the_selected_root() {
    let anchors = |checkpoint: u8, selected: u8| RootAnchors {
        checkpoint_root_frame_sha256: [checkpoint; 32],
        selected_root_frame_sha256: [selected; 32],
    };
    let selection = anchors(0x11, 0x22);
    assert_eq!(
        anchored_root_identity(anchors(0x11, 0x22), selection),
        Some([0x22; 32])
    );
    // Walk from a different checkpoint.
    assert_eq!(anchored_root_identity(anchors(0x13, 0x22), selection), None);
    // Walk ending at a root other than the selected one.
    assert_eq!(anchored_root_identity(anchors(0x11, 0x23), selection), None);
    assert_eq!(anchored_root_identity(anchors(0x22, 0x11), selection), None);
}
