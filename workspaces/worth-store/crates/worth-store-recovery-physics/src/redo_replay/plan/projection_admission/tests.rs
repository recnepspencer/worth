use super::*;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    encode_data_frame_page_lsn, prepare_extent_chunk, CurrentPhysicalRecordPlacement,
    DurableExtentRecordPlacement, DurableFrameKind, PersistedPhysicalDataFrameSubject,
    PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryManifest,
    PersistedPhysicalRecoveryRootState, PersistedRecordIdentity, PhysicalExtentId,
    PhysicalGeneration, PhysicalGenerationAuthority, RecordArtifactFile, RecordFrameCoordinate,
};

#[test]
fn extent_redo_admission_carries_manifest_chunk_payload_and_page_lsn() {
    let (projection, payload, coordinate) = extent_projection(false, false);
    let mut budget = RedoAllocationBudget::new(u64::MAX);
    let admitted = admit_projection(&projection, store(), format(), &mut budget).unwrap();
    assert_eq!(admitted.extent_manifests().len(), 1);
    assert_eq!(admitted.extent_chunks().len(), 1);
    let chunk = &admitted.extent_chunks()[0];
    assert_eq!(chunk.coordinate, coordinate);
    assert_eq!(chunk.page_lsn, PhysicalPageLsn::new(33));
    assert_eq!(
        &admitted.frame_bytes(chunk.frame_index)[chunk.payload_range.clone()],
        payload,
    );
}

#[test]
fn extent_redo_rejects_corruption_and_foreign_manifest_incarnation() {
    for (corrupt_chunk, foreign_manifest) in [(true, false), (false, true)] {
        let (projection, _, _) = extent_projection(corrupt_chunk, foreign_manifest);
        let mut budget = RedoAllocationBudget::new(u64::MAX);
        assert!(matches!(
            admit_projection(&projection, store(), format(), &mut budget),
            Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
        ));
    }
}

fn extent_projection(
    corrupt_chunk: bool,
    foreign_manifest: bool,
) -> (
    PersistedPhysicalRecoveryProjection,
    &'static [u8],
    ExtentChunkCoordinate,
) {
    const PAYLOAD: &[u8] = b"extent-redo";
    let format = format();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let record = PersistedRecordIdentity::new([4; 16], 1).unwrap();
    let extent = authority
        .record_extent_cell(PhysicalExtentId::from_raw(7).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(2).unwrap());
    let placement = DurableExtentRecordPlacement::legacy_unknown(
        record,
        extent,
        PAYLOAD.len() as u64,
        worth_store_physical_format::ExtentArenaRange::new(
            worth_store_physical_format::ExtentArenaId::new(3).unwrap(),
            4096,
            20480,
        )
        .unwrap(),
    )
    .unwrap();
    let coordinate =
        ExtentChunkCoordinate::new(record, extent, PAYLOAD.len() as u64, 0, 1).unwrap();
    let mut unsealed = prepare_extent_chunk(format, coordinate, PAYLOAD.len()).unwrap();
    unsealed.payload_mut().copy_from_slice(PAYLOAD);
    let mut bytes = unsealed.seal();
    encode_data_frame_page_lsn(
        &mut bytes,
        DurableFrameKind::Extent,
        PhysicalPageLsn::new(33),
    )
    .unwrap();
    if corrupt_chunk {
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
    }
    let artifact = RecordArtifactFile::ExtentArena { arena: 3 };
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::ExtentChunk(coordinate),
        RecordFrameCoordinate::new(artifact, 8192, bytes.len() as u32).unwrap(),
        &bytes,
    )
    .unwrap();
    let manifest_extent = if foreign_manifest {
        authority
            .record_extent_cell(PhysicalExtentId::from_raw(7).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(3).unwrap())
    } else {
        extent
    };
    let manifest = DurableExtentManifest::new(
        format,
        record,
        manifest_extent,
        PAYLOAD.len() as u64,
        format.page_size().bytes(),
        1,
        4096,
    )
    .unwrap();
    let manifest = PersistedPhysicalRecoveryManifest::new(
        RecordFrameCoordinate::new(artifact, 4096, 104).unwrap(),
        &manifest.encode(format),
    )
    .unwrap();
    let root =
        PersistedPhysicalRecoveryRootState::new(4096, 1, 4, Vec::new(), None, None).unwrap();
    let projection = PersistedPhysicalRecoveryProjection::new(
        1,
        root,
        vec![record],
        vec![frame],
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        Vec::new(),
        vec![manifest],
    )
    .unwrap();
    (projection, PAYLOAD, coordinate)
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn store() -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x51; 16]).unwrap(),
    )
    .published_identity()
}
