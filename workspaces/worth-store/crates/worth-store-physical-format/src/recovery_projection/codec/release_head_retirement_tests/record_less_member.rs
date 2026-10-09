//! A terminal head retirement is the one record-less member. These tests pin
//! that shape in both directions, in the projection and in canonical redo.

use super::super::head_effect_tests::{assert_funded_canonical_decode, one_record_member};
use super::*;
use crate::{
    decode_canonical_redo_v3, CanonicalRedoWireDenial, PersistedInlineSegmentAllocation,
    PersistedPhysicalRecoveryManifest, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageId, PhysicalSegmentId, RecordArtifactFile, RecordFrameCoordinate,
    RecordSegmentPageManifestEntry, CANONICAL_REDO_V3_DOMAIN,
};

fn operation() -> PersistedPhysicalRecoveryOperation {
    PersistedPhysicalRecoveryOperation::TerminalReleaseHeadRetired(retirement(
        vec![terminal_head()],
    ))
}

#[test]
fn a_terminal_head_retirement_carries_no_data_record_frame_or_placement() {
    let (data, _) = upsert_projection();
    let records = || data.record_identities().to_vec();
    let frames = || data.frames().unwrap().to_vec();
    let placements = || data.placements().to_vec();
    let build = PersistedPhysicalRecoveryProjection::new_with_operation;
    let generation = SOURCE_GENERATION;
    assert!(build(
        generation,
        root_state(),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        operation()
    )
    .is_some());
    for (records, frames, placements) in [
        (records(), frames(), placements()),
        (records(), vec![], vec![]),
        (vec![], frames(), vec![]),
        (vec![], vec![], placements()),
    ] {
        let projection = build(
            generation,
            root_state(),
            records,
            frames,
            placements,
            vec![],
            vec![],
            operation(),
        );
        assert_eq!(projection, None);
    }
    assert_eq!(
        build(
            generation + 1,
            root_state(),
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            operation()
        ),
        None,
        "the retirement is bound to the projection's source root generation"
    );
    for (records, frames) in [(vec![], vec![]), (vec![], frames()), (records(), vec![])] {
        let projection = build(
            generation,
            root_state(),
            records,
            frames,
            vec![],
            vec![],
            vec![],
            PersistedPhysicalRecoveryOperation::None,
        );
        assert_eq!(
            projection, None,
            "every other operation still requires its data record and frame"
        );
    }
}

#[test]
fn a_terminal_head_retirement_carries_no_segment_manifest_or_inline_delta() {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let generation = PhysicalGeneration::from_raw(3).unwrap();
    let segment_id = PhysicalSegmentId::from_raw(1).unwrap();
    let segment = authority
        .segment_cell(segment_id)
        .with_segment_generation(generation);
    let page = authority
        .page_cell(segment_id, PhysicalPageId::from_raw(9).unwrap())
        .with_page_generation(generation);
    let update = RecordSegmentPageManifestEntry::new(page, segment, 1, 0).unwrap();
    let manifest = PersistedPhysicalRecoveryManifest::new(
        RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 1 }, 0, 4).unwrap(),
        &[1, 2, 3, 4],
    )
    .unwrap();
    let allocation = PersistedInlineSegmentAllocation::new(segment, 4, 1).unwrap();
    let inline_allocation =
        PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![allocation], None, None).unwrap();
    let inline_tail = PersistedPhysicalRecoveryRootState::new(
        4096,
        1,
        32,
        vec![],
        Some(record(5)),
        Some(segment),
    )
    .unwrap();
    let build = |root, updates, manifests| {
        PersistedPhysicalRecoveryProjection::new_with_operation(
            SOURCE_GENERATION,
            root,
            vec![],
            vec![],
            vec![],
            updates,
            manifests,
            operation(),
        )
    };
    assert!(build(root_state(), vec![], vec![]).is_some());
    assert_eq!(build(root_state(), vec![update], vec![]), None);
    assert_eq!(build(root_state(), vec![], vec![manifest]), None);
    assert_eq!(build(inline_allocation, vec![], vec![]), None);
    assert_eq!(build(inline_tail, vec![], vec![]), None);
}

fn record_less_member(projection: &[u8]) -> Vec<u8> {
    let mut member = Vec::new();
    member.extend_from_slice(&(CANONICAL_REDO_V3_DOMAIN.len() as u64).to_le_bytes());
    member.extend_from_slice(CANONICAL_REDO_V3_DOMAIN);
    member.extend_from_slice(&0_u64.to_le_bytes());
    member.extend_from_slice(&(projection.len() as u64).to_le_bytes());
    member.extend_from_slice(projection);
    member
}

#[test]
fn canonical_redo_admits_a_record_less_member_only_as_a_terminal_head_retirement() {
    let projection = retirement_projection(retirement(vec![terminal_head(), survivor()]));
    let member = record_less_member(&projection.encode());
    let decode = |member: &[u8], start, end| {
        decode_canonical_redo_v3(member, start, end, 1, None, LIMITS, format())
    };
    let (records, decoded) = decode(&member, 12, 13).expect("one LSN, no record");
    assert!(records.is_empty());
    assert_eq!(decoded, projection);
    assert_funded_canonical_decode(&member, LIMITS, format(), &projection);
    for end in [12, 14] {
        assert_eq!(
            decode(&member, 12, end),
            Err(CanonicalRedoWireDenial::LsnRangeMismatch),
            "a record-less member occupies exactly one LSN"
        );
    }

    // No other operation may omit its records, and a retirement may carry none.
    let (data, _) = upsert_projection();
    let data_limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: 1,
        record_identities: 1,
        placements: 1,
        ..LIMITS
    };
    let decode_data =
        |member: &[u8]| decode_canonical_redo_v3(member, 12, 13, 1, None, data_limits, format());
    assert!(decode_data(&one_record_member(&data.encode())).is_ok());
    for mismatched in [
        record_less_member(&data.encode()),
        one_record_member(&projection.encode()),
    ] {
        assert_eq!(
            decode_data(&mismatched),
            Err(CanonicalRedoWireDenial::InvalidRecoveryProjection)
        );
    }
}
