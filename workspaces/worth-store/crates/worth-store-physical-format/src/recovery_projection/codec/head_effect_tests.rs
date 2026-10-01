use sha2::{Digest, Sha256};

use super::*;
use crate::{
    decode_canonical_redo_v3, BlobGenerationPublicationV1, BlobReclaimSourceBasisV1,
    CanonicalRedoWireDenial, CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement,
    ExtentArenaId, ExtentArenaRange, ExtentChunkCoordinate, PersistedBlobSemanticRecordBinding,
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryOperation, PersistedPhysicalRecoveryRootState,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageSizeClass, PhysicalRecordFormatDeclaration, RecordArtifactFile,
    RecordFrameCoordinate, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
    ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadTransitionLimitsV1,
    ReleaseCustodyHeadTransitionV1, ReleasedGenerationReclaimBasisV1, CANONICAL_REDO_V3_DOMAIN,
};

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

fn upsert_projection() -> (
    PersistedPhysicalRecoveryProjection,
    PhysicalRecordFormatDeclaration,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let publication = BlobGenerationPublicationV1::new(
        [7; 16],
        [2; 16],
        [3; 16],
        3,
        record(3),
        [4; 32],
        8,
        [5; 32],
        64 * 1024,
        [6; 32],
    )
    .unwrap();
    let basis = ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(4),
        Sha256::digest(publication.encode()).into(),
        [6; 32],
    )
    .unwrap();
    let identity = record(9);
    let entry = ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new(basis.object(), basis.generation()).unwrap(),
        identity,
        [5; 32],
        record(7),
        [8; 32],
        record(8),
        [9; 32],
        BlobReclaimSourceBasisV1::ReleasedGeneration(basis).digest([7; 16]),
        None,
        11,
        1,
        false,
    )
    .unwrap();
    let limits = ReleaseCustodyHeadTransitionLimitsV1::new(1, 3, 16 * 1024).unwrap();
    let planned = ReleaseCustodyHeadTransitionV1::plan(
        None,
        1,
        &[],
        ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior: None,
            next: entry,
        },
        12,
        6,
        format,
        limits,
    )
    .unwrap();
    let effect = PersistedReleaseCustodyHeadEffectV1::new_upsert(
        6,
        11,
        basis,
        vec![],
        planned,
        format,
        limits,
    )
    .unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(3).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap());
    let placement = DurableExtentRecordPlacement::legacy_unknown(
        identity,
        extent,
        1,
        ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 0, 53_248).unwrap(),
    )
    .unwrap();
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::ExtentChunk(
            ExtentChunkCoordinate::new(identity, extent, 1, 0, 1).unwrap(),
        ),
        RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 2 }, 0, 1).unwrap(),
        &[3],
    )
    .unwrap();
    let root = PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap();
    let binding = PersistedBlobSemanticRecordBinding::new(identity, [5; 32], 12).unwrap();
    let projection = PersistedPhysicalRecoveryProjection::new_with_operation(
        11,
        root,
        vec![identity],
        vec![frame],
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        vec![],
        vec![],
        PersistedPhysicalRecoveryOperation::RecordsDropped {
            binding,
            head_effect: Some(effect),
        },
    )
    .unwrap();
    (projection, format)
}

#[test]
fn v15_head_upsert_round_trips_and_rejects_substituted_node_bytes() {
    let (projection, format) = upsert_projection();
    let bytes = projection.encode();
    let limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: 1,
        record_identities: 1,
        placements: 1,
        segment_updates: 0,
        manifests: 0,
        total_entries: 2,
        inline_allocations: 0,
    };
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&bytes, limits, format),
        Ok(projection.clone())
    );
    let PersistedPhysicalRecoveryOperation::RecordsDropped {
        head_effect: Some(effect),
        ..
    } = projection.operation()
    else {
        panic!("fixture must carry a head effect")
    };
    let node = effect.node_writes()[0].frame();
    let offset = bytes
        .windows(node.len())
        .position(|window| window == node)
        .unwrap();
    let mut changed = bytes;
    changed[offset + node.len() - 1] ^= 1;
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&changed, limits, format),
        Err(PhysicalRecoveryProjectionDenial::Malformed)
    );
}

#[test]
fn canonical_redo_decodes_v15_only_with_its_actual_record_format() {
    let (projection, format) = upsert_projection();
    let field = |target: &mut Vec<u8>, bytes: &[u8]| {
        target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        target.extend_from_slice(bytes);
    };
    let mut target = vec![2];
    target.extend_from_slice(&[1; 16]);
    target.extend_from_slice(&9_u64.to_le_bytes());
    target.extend_from_slice(&3_u64.to_le_bytes());
    target.extend_from_slice(&4_u64.to_le_bytes());
    target.extend_from_slice(&1_u64.to_le_bytes());
    target.extend_from_slice(&0_u64.to_le_bytes());
    target.extend_from_slice(&1_u32.to_le_bytes());
    target.push(8);
    target.extend_from_slice(&2_u64.to_le_bytes());
    target.extend_from_slice(&0_u64.to_le_bytes());
    target.extend_from_slice(&0_u64.to_le_bytes());
    target.extend_from_slice(&1_u32.to_le_bytes());
    let mut member = Vec::new();
    field(&mut member, CANONICAL_REDO_V3_DOMAIN);
    member.extend_from_slice(&1_u64.to_le_bytes());
    member.extend_from_slice(&0_u32.to_le_bytes());
    member.extend_from_slice(&12_u64.to_le_bytes());
    member.extend_from_slice(&1_u64.to_le_bytes());
    field(&mut member, &target);
    member.extend_from_slice(&<[u8; 32]>::from(Sha256::digest([3])));
    field(&mut member, &[3]);
    field(&mut member, &projection.encode());
    let limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: 1,
        record_identities: 1,
        placements: 1,
        segment_updates: 0,
        manifests: 0,
        total_entries: 2,
        inline_allocations: 0,
    };
    let (_, decoded) = decode_canonical_redo_v3(&member, 12, 13, 1, None, limits, format)
        .expect("canonical WAL must carry the exact V15 head effect");
    assert_eq!(decoded, projection);
    let wrong_format = PhysicalRecordFormatDeclaration::builder()
        .page_size(PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    assert_eq!(
        decode_canonical_redo_v3(&member, 12, 13, 1, None, limits, wrong_format),
        Err(CanonicalRedoWireDenial::InvalidRecoveryProjection),
    );
}

#[test]
fn v15_rejects_unrelated_key_and_missing_or_extra_node() {
    let (projection, format) = upsert_projection();
    let PersistedPhysicalRecoveryOperation::RecordsDropped {
        head_effect: Some(effect),
        ..
    } = projection.operation()
    else {
        panic!("fixture must carry a head effect")
    };
    let ReleaseCustodyHeadMutationV1::Upsert { next, .. } = effect.mutation() else {
        panic!("fixture must be an upsert")
    };
    let wrong = ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([99; 16], next.key().generation()).unwrap(),
        next.descriptor_record(),
        next.descriptor_frame_sha256(),
        next.manifest_record(),
        next.manifest_frame_sha256(),
        next.reservation_record(),
        next.reservation_frame_sha256(),
        next.source_basis_digest(),
        next.predecessor(),
        next.source_root_generation(),
        next.cumulative_dropped(),
        next.terminal(),
    )
    .unwrap();
    let limits = ReleaseCustodyHeadTransitionLimitsV1::new(1, 3, 16 * 1024).unwrap();
    let planned = ReleaseCustodyHeadTransitionV1::plan(
        None,
        1,
        &[],
        ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior: None,
            next: wrong,
        },
        12,
        6,
        format,
        limits,
    )
    .unwrap();
    assert_eq!(
        PersistedReleaseCustodyHeadEffectV1::new_upsert(
            6,
            11,
            effect.source_basis(),
            vec![],
            planned,
            format,
            limits,
        ),
        Err(PhysicalRecoveryProjectionDenial::Malformed),
    );

    let encoded = encode_head_effect(effect);
    let frame_len = effect.node_writes()[0].frame().len();
    let write_count_offset = encoded.len() - (8 + 8 + 104 + 8 + frame_len);
    assert_eq!(
        &encoded[write_count_offset..write_count_offset + 8],
        &1_u64.to_le_bytes()
    );
    for count in [0_u64, 2_u64] {
        let mut changed = encoded.clone();
        changed[write_count_offset..write_count_offset + 8].copy_from_slice(&count.to_le_bytes());
        let mut remaining = 2;
        assert_eq!(
            decode_head_effect(&changed, 11, &mut remaining, format),
            Err(PhysicalRecoveryProjectionDenial::Malformed),
        );
    }
}
