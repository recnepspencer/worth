use sha2::{Digest, Sha256};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobReclaimDescriptorV2, BlobReclaimDescriptorV3,
    BlobReclaimSourceKind, CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement,
    ExtentArenaId, ExtentArenaRange, ExtentChunkCoordinate, OriginalDropReservationRequestV1,
    PersistedBlobSemanticRecordBinding, PersistedPhysicalDataFrameSubject,
    PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryOperation,
    PersistedPhysicalRecoveryRootState, PersistedRecordIdentity,
    PersistedReleaseCustodyHeadEffectV1, PhysicalExtentId, PhysicalGeneration,
    PhysicalGenerationAuthority, RecordArtifactFile, RecordFrameCoordinate,
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadMutationV1,
    ReleaseCustodyHeadTransitionLimitsV1, ReleaseCustodyHeadTransitionV1, ReleasedDropCustodyV1,
    ReleasedGenerationReclaimBasisV1,
};

use super::*;

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

fn fixture() -> (
    Vec<u8>,
    PersistedRecordIdentity,
    PersistedPhysicalRecoveryProjection,
    StableStoreIdentity,
    PhysicalRecordFormatDeclaration,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([7; 16]).unwrap(),
    )
    .published_identity();
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
    let source_digest = BlobReclaimSourceBasisV1::ReleasedGeneration(basis).digest([7; 16]);
    let base = BlobReclaimDescriptorV2::new(
        [7; 16],
        [8; 16],
        BlobReclaimSourceKind::ReleasedGeneration,
        source_digest,
        record(7),
        [8; 32],
        1,
        11,
        12,
        None,
        1,
        false,
    )
    .unwrap();
    let request = OriginalDropReservationRequestV1::new([10; 32], [11; 32], 10, 20).unwrap();
    let custody = ReleasedDropCustodyV1::new(
        [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [6; 32], request,
    )
    .unwrap();
    let bytes = BlobReclaimDescriptorV3::new(base, custody)
        .unwrap()
        .encode();
    let identity = record(9);
    let entry = ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new(basis.object(), basis.generation()).unwrap(),
        identity,
        Sha256::digest(&bytes).into(),
        record(7),
        [8; 32],
        record(8),
        [9; 32],
        source_digest,
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
        bytes.len() as u64,
        ExtentArenaRange::new(ExtentArenaId::new(2).unwrap(), 0, 53_248).unwrap(),
    )
    .unwrap();
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::ExtentChunk(
            ExtentChunkCoordinate::new(identity, extent, bytes.len() as u64, 0, bytes.len() as u32)
                .unwrap(),
        ),
        RecordFrameCoordinate::new(
            RecordArtifactFile::ExtentArena { arena: 2 },
            0,
            bytes.len() as u32,
        )
        .unwrap(),
        &bytes,
    )
    .unwrap();
    let binding =
        PersistedBlobSemanticRecordBinding::new(identity, Sha256::digest(&bytes).into(), 12)
            .unwrap();
    let projection = PersistedPhysicalRecoveryProjection::new_with_operation(
        11,
        PersistedPhysicalRecoveryRootState::new(4096, 1, 32, vec![], None, None).unwrap(),
        vec![identity],
        vec![frame],
        vec![CurrentPhysicalRecordPlacement::Extent(placement)],
        vec![],
        vec![],
        PersistedPhysicalRecoveryOperation::RecordsDropped {
            binding,
            head_effect: Some(effect),
            directory_replacement: None,
        },
    )
    .unwrap();
    (bytes, identity, projection, store, format)
}

#[test]
fn current_head_effect_binds_exact_descriptor_and_charges_roster() {
    let (bytes, identity, projection, store, format) = fixture();
    let PersistedPhysicalRecoveryOperation::RecordsDropped {
        head_effect: Some(effect),
        ..
    } = projection.operation()
    else {
        panic!("fixture retains the exact head effect");
    };
    assert_eq!(
        validate_head_descriptor(&bytes, identity, &projection, store, format, effect),
        Ok(())
    );
    let mut changed = bytes.clone();
    let last = changed.len() - 1;
    changed[last] ^= 1;
    assert_eq!(
        validate_head_descriptor(&changed, identity, &projection, store, format, effect),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
    assert_eq!(
        validate_head_descriptor(&bytes, record(10), &projection, store, format, effect),
        Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
    );
    let charged =
        crate::redo_replay::plan::supersession::admit_scratch_bytes(0, &[], &projection, u64::MAX)
            .unwrap();
    assert!(charged > effect.framed_bytes().unwrap());
    assert!(matches!(
        crate::redo_replay::plan::supersession::admit_scratch_bytes(
            0,
            &[],
            &projection,
            charged - 1,
        ),
        Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit { .. }),
    ));
}
