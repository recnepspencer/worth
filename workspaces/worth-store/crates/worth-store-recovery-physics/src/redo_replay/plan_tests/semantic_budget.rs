use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, encode_data_frame_page_lsn, prepare_extent_chunk,
    validate_blob_record_header, BlobRecordKind, BlobTreeEntryV1, BlobTreeNodeKind,
    BlobTreeNodeV1, BlobTreeOccurrenceV1, CurrentPhysicalRecordPlacement,
    DerivedFamilyDirectoryDenial, DerivedFamilyRootDirectoryV1, DurableExtentManifest,
    DurableExtentRecordPlacement, DurableFrameKind, ExtentArenaFrameLayout, ExtentArenaId,
    ExtentArenaRange, ExtentChunkCoordinate, IndexedThroughBlobPublication,
    PersistedBlobSemanticRecordBinding, PersistedDerivedDirectoryRecordBinding,
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryManifest, PersistedPhysicalRecoveryOperation,
    PersistedPhysicalRecoveryProjection, PersistedPhysicalRecoveryRootState,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageLsn, PhysicalRecoveryProjectionDecodeLimits, RecordArtifactFile,
    RecordFrameCoordinate, CANONICAL_REDO_V3_DOMAIN, DURABLE_EXTENT_FRAME_HEADER_BYTES,
    EXTENT_CHUNK_METADATA_BYTES,
};

use super::*;

fn extent_target_bytes(
    record: PersistedRecordIdentity,
    extent: u64,
    generation: u64,
    logical_bytes: u64,
    logical_offset: u64,
    chunk: u32,
    arena: u64,
    artifact_offset: u64,
    artifact_length: u32,
) -> Vec<u8> {
    let mut target = Vec::new();
    target.push(2);
    target.extend_from_slice(&record.allocation_epoch());
    target.extend_from_slice(&record.ordinal().to_le_bytes());
    target.extend_from_slice(&extent.to_le_bytes());
    target.extend_from_slice(&generation.to_le_bytes());
    target.extend_from_slice(&logical_bytes.to_le_bytes());
    target.extend_from_slice(&logical_offset.to_le_bytes());
    target.extend_from_slice(&chunk.to_le_bytes());
    target.push(16);
    target.extend_from_slice(&arena.to_le_bytes());
    target.extend_from_slice(&artifact_offset.to_le_bytes());
    target.extend_from_slice(&artifact_length.to_le_bytes());
    target
}

fn encode_extent_member(
    record: PersistedRecordIdentity,
    payload: &[u8],
    operation: PersistedPhysicalRecoveryOperation,
    reverse_frames: bool,
) -> PhysicalRedoMemberInput {
    let format = test_format();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let extent = authority
        .record_extent_cell(PhysicalExtentId::from_raw(7).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(2).unwrap());
    let max_frame = format.page_size().bytes();
    let chunk_cap = max_frame as usize - (DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES);
    let chunks_raw: Vec<&[u8]> = payload.chunks(chunk_cap).collect();
    let chunk_count = u32::try_from(chunks_raw.len()).unwrap();
    let arena = 3_u64;
    let artifact = RecordArtifactFile::ExtentArena { arena };
    let layout = ExtentArenaFrameLayout::new(format, 4096).unwrap();
    let manifest_offset = 4096_u64;
    let arena_bytes = layout.allocated_bytes(chunk_count).unwrap();
    let placement = DurableExtentRecordPlacement::legacy_unknown(
        record,
        extent,
        payload.len() as u64,
        ExtentArenaRange::new(
            ExtentArenaId::new(arena).unwrap(),
            manifest_offset,
            arena_bytes,
        )
        .unwrap(),
    )
    .unwrap();
    let manifest = DurableExtentManifest::new(
        format,
        record,
        extent,
        payload.len() as u64,
        max_frame,
        chunk_count,
        4096,
    )
    .unwrap();
    let manifest_bytes = manifest.encode(format);
    let manifest_frame = PersistedPhysicalRecoveryManifest::new(
        RecordFrameCoordinate::new(artifact, manifest_offset, manifest_bytes.len() as u32).unwrap(),
        &manifest_bytes,
    )
    .unwrap();

    let mut frames = Vec::with_capacity(chunks_raw.len());
    let mut targets = Vec::with_capacity(chunks_raw.len());
    let mut logical_offset = 0_u64;
    for (idx, chunk_slice) in chunks_raw.iter().enumerate() {
        let ordinal = u32::try_from(idx + 1).unwrap();
        let coord = ExtentChunkCoordinate::new(
            record,
            extent,
            payload.len() as u64,
            logical_offset,
            ordinal,
        )
        .unwrap();
        let mut unsealed = prepare_extent_chunk(format, coord, chunk_slice.len()).unwrap();
        unsealed.payload_mut().copy_from_slice(chunk_slice);
        let mut sealed = unsealed.seal();
        encode_data_frame_page_lsn(
            &mut sealed,
            DurableFrameKind::Extent,
            PhysicalPageLsn::new(10),
        )
        .unwrap();
        let digest: [u8; 32] = Sha256::digest(&sealed).into();
        let art_offset = manifest_offset + layout.chunk_offset(ordinal).unwrap();
        let art_len = u32::try_from(sealed.len()).unwrap();
        frames.push(
            PersistedPhysicalRecoveryFrame::new(
                PersistedPhysicalDataFrameSubject::ExtentChunk(coord),
                RecordFrameCoordinate::new(artifact, art_offset, art_len).unwrap(),
                &sealed,
            )
            .unwrap(),
        );
        let target_bytes = extent_target_bytes(
            record,
            7,
            2,
            payload.len() as u64,
            logical_offset,
            ordinal,
            arena,
            art_offset,
            art_len,
        );
        targets.push((target_bytes, digest));
        logical_offset += chunk_slice.len() as u64;
    }
    if reverse_frames {
        frames.reverse();
    }
    let root = PersistedPhysicalRecoveryRootState::new(4096, 1, 4, Vec::new(), None, None).unwrap();
    let projection = PersistedPhysicalRecoveryProjection::new_with_operation(
        1,
        root,
        vec![record],
        frames,
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        Vec::new(),
        vec![manifest_frame],
        operation,
    )
    .unwrap()
    .encode();

    let mut wire = Vec::new();
    wire.extend_from_slice(&(CANONICAL_REDO_V3_DOMAIN.len() as u64).to_le_bytes());
    wire.extend_from_slice(CANONICAL_REDO_V3_DOMAIN);
    wire.extend_from_slice(&1_u64.to_le_bytes());
    wire.extend_from_slice(&0_u32.to_le_bytes());
    wire.extend_from_slice(&10_u64.to_le_bytes());
    wire.extend_from_slice(&(targets.len() as u64).to_le_bytes());
    for (target_bytes, digest) in targets {
        wire.extend_from_slice(&(target_bytes.len() as u64).to_le_bytes());
        wire.extend_from_slice(&target_bytes);
        wire.extend_from_slice(&digest);
    }
    wire.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    wire.extend_from_slice(payload);
    wire.extend_from_slice(&(projection.len() as u64).to_le_bytes());
    wire.extend_from_slice(&projection);

    PhysicalRedoMemberInput::new(
        range(),
        [0x33; 32],
        RecoveryOperationFate::Indeterminate,
        &wire,
    )
}

fn find_exact_budget(member: &PhysicalRedoMemberInput, limits: PhysicalRedoAdmissionLimits) -> u64 {
    let mut lo = 0_u64;
    let mut hi = 4 * 1024 * 1024_u64;
    admit_physical_redo_members_with_memory_budget(
        vec![member.clone()],
        test_store(),
        test_format(),
        limits,
        hi,
    )
    .expect("generous budget must admit valid member");
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if admit_physical_redo_members_with_memory_budget(
            vec![member.clone()],
            test_store(),
            test_format(),
            limits,
            mid,
        )
        .is_ok()
        {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
}

fn generous_limits() -> PhysicalRedoAdmissionLimits {
    PhysicalRedoAdmissionLimits {
        recovery_memory_bytes: 4 * 1024 * 1024,
        targets: 16,
        distinct_targets: 16,
        projection: PhysicalRecoveryProjectionDecodeLimits {
            frames: 16,
            record_identities: 16,
            placements: 16,
            segment_updates: 16,
            manifests: 16,
            total_entries: 64,
            inline_allocations: 16,
        },
    }
}

#[test]
fn nonempty_blob_tree_node_semantic_record_admits_at_exact_budget_and_handles_unsorted_chunks() {
    let store_bytes = test_store().bytes();
    let record = PersistedRecordIdentity::new([0x21; 16], 1).unwrap();
    let entries: Vec<BlobTreeEntryV1> = (1_u64..=600)
        .map(|i| {
            let mut digest = [0_u8; 32];
            digest[..8].copy_from_slice(&i.to_le_bytes());
            BlobTreeEntryV1::new(
                digest,
                PersistedRecordIdentity::new([0x22; 16], i).unwrap(),
                4096,
            )
            .unwrap()
        })
        .collect();
    let tree = BlobTreeNodeV1::new(
        BlobTreeOccurrenceV1::new(store_bytes, [0x23; 16], BlobTreeNodeKind::Leaf, 0, 0).unwrap(),
        entries,
    )
    .unwrap();
    let payload = tree.encode();
    let header = validate_blob_record_header(&payload).expect("borrowed header validates");
    assert_eq!(header.kind(), BlobRecordKind::TreeNode);
    assert_eq!(header.store(), store_bytes);
    assert_eq!(decode_blob_record(&payload).unwrap().kind(), header.kind());

    // Tampered tree-node byte fails both borrowed and owned validation identically.
    let mut corrupted = payload.clone();
    let last = corrupted.len() - 1;
    corrupted[last] ^= 1;
    assert_eq!(
        validate_blob_record_header(&corrupted).err(),
        decode_blob_record(&corrupted).err()
    );

    // Split payload across 3 canonical extent chunks and reverse frame arrival order.
    let member = encode_extent_member(
        record,
        &payload,
        PersistedPhysicalRecoveryOperation::None,
        true,
    );
    let opaque_member = encode_extent_member(
        record,
        &vec![0xAB; payload.len()],
        PersistedPhysicalRecoveryOperation::None,
        true,
    );
    let limits = generous_limits();
    let exact_opaque = find_exact_budget(&opaque_member, limits);
    let exact = find_exact_budget(&member, limits);
    assert!(exact > 0);
    assert_eq!(
        exact, exact_opaque,
        "borrowed tree-node validation must require zero extra heap bytes beyond non-blob decode"
    );
    assert_eq!(
        admit_physical_redo_members_with_memory_budget(
            vec![member.clone()],
            test_store(),
            test_format(),
            limits,
            exact - 1,
        )
        .err(),
        Some(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
            observed: exact,
            admitted: exact - 1,
        })
    );
    let admitted = admit_physical_redo_members_with_memory_budget(
        vec![member],
        test_store(),
        test_format(),
        limits,
        exact,
    )
    .expect("exact budget admits 128-entry blob tree node across 3 unsorted chunks");
    assert_eq!(admitted.target_identity_count(), 3);
    assert_eq!(admitted.distinct_target_count(), 3);
}

#[test]
fn nonempty_derived_directory_semantic_record_admits_at_exact_budget_without_vec_alloc() {
    let record = PersistedRecordIdentity::new([0x31; 16], 1).unwrap();
    let pub_rec = PersistedRecordIdentity::new([0x31; 16], 2).unwrap();
    let quar_rec = PersistedRecordIdentity::new([0x31; 16], 3).unwrap();
    let indexed = IndexedThroughBlobPublication::new(1, pub_rec, [0x44; 32]).unwrap();
    let mut payload = DerivedFamilyRootDirectoryV1::new(Vec::new())
        .unwrap()
        .with_indexed_through(indexed)
        .with_indexed_through_quarantine(Some(quar_rec))
        .encode();
    payload[9] = 2;
    for (tag, ordinal) in [(1_u16, 1_u64), (2_u16, 2_u64)] {
        payload.extend_from_slice(&tag.to_le_bytes());
        payload.extend_from_slice(&[0x32; 16]);
        payload.extend_from_slice(&ordinal.to_le_bytes());
    }
    let decoded = DerivedFamilyRootDirectoryV1::decode(&payload).unwrap();
    assert_eq!(decoded.entries().len(), 2);
    let header = DerivedFamilyRootDirectoryV1::validate_header(&payload).unwrap();
    assert_eq!(header.indexed_through_blob_publication(), Some(indexed));
    assert_eq!(header.indexed_through_quarantine(), Some(quar_rec));
    assert_eq!(header.entry_count(), 2);

    // Non-canonical order (swap the two entries) must fail validate_header identically to decode.
    let mut swapped = payload.clone();
    let entry_start = 101;
    let first = swapped[entry_start..entry_start + 26].to_vec();
    let second = swapped[entry_start + 26..entry_start + 52].to_vec();
    swapped[entry_start..entry_start + 26].copy_from_slice(&second);
    swapped[entry_start + 26..entry_start + 52].copy_from_slice(&first);
    assert_eq!(
        DerivedFamilyRootDirectoryV1::validate_header(&swapped),
        Err(DerivedFamilyDirectoryDenial::NonCanonicalOrder)
    );
    assert_eq!(
        DerivedFamilyRootDirectoryV1::decode(&swapped).err(),
        Some(DerivedFamilyDirectoryDenial::NonCanonicalOrder)
    );

    let binding = PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
        PersistedBlobSemanticRecordBinding::new(record, Sha256::digest(&payload).into(), 2)
            .unwrap(),
        Some(indexed),
        Some(quar_rec),
    );
    let member = encode_extent_member(
        record,
        &payload,
        PersistedPhysicalRecoveryOperation::DerivedDirectory {
            binding,
            retirement: None,
        },
        false,
    );
    let limits = generous_limits();
    let exact = find_exact_budget(&member, limits);
    assert!(exact > 0);
    assert_eq!(
        admit_physical_redo_members_with_memory_budget(
            vec![member.clone()],
            test_store(),
            test_format(),
            limits,
            exact - 1,
        )
        .err(),
        Some(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
            observed: exact,
            admitted: exact - 1,
        })
    );
    let admitted = admit_physical_redo_members_with_memory_budget(
        vec![member],
        test_store(),
        test_format(),
        limits,
        exact,
    )
    .expect("exact budget admits nonempty derived directory without directory vec allocation");
    assert_eq!(admitted.target_identity_count(), 1);
}
