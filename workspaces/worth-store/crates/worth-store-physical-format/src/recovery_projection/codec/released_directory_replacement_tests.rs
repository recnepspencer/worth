use sha2::{Digest, Sha256};

use super::head_effect_tests::{record, upsert_projection};
use super::*;
use crate::{
    BlobRecordKind, CurrentPhysicalRecordPlacement, DerivedFamilyRootDirectoryBinding,
    DerivedFamilyRootDirectoryV1, DurableExtentRecordPlacement, ExtentArenaId, ExtentArenaRange,
    ExtentChunkCoordinate, IndexedThroughBlobPublication, PersistedBlobSemanticRecordBinding,
    PersistedDerivedDirectoryRecordBinding, PersistedPhysicalDataFrameSubject,
    PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryOperation,
    PersistedReleasedDirectoryReplacementV1, PhysicalExtentId, PhysicalGeneration,
    PhysicalGenerationAuthority, RecordArtifactFile, RecordFrameCoordinate,
    SelectedRecordContentClass, SelectedRecordRouteMetadata,
};

#[test]
fn released_directory_replacement_round_trips_as_one_two_record_drop() {
    let (base, format) = upsert_projection();
    let bytes = DerivedFamilyRootDirectoryV1::new(vec![]).unwrap().encode();
    let directory = record(10);
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(4).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(4).unwrap());
    let route = CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::new_selected(
            directory,
            extent,
            bytes.len() as u64,
            ExtentArenaRange::new(ExtentArenaId::new(3).unwrap(), 0, 53_248).unwrap(),
            SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::DerivedDirectory)
                .unwrap(),
        )
        .unwrap(),
    );
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::ExtentChunk(
            ExtentChunkCoordinate::new(
                directory,
                extent,
                bytes.len() as u64,
                0,
                bytes.len() as u32,
            )
            .unwrap(),
        ),
        RecordFrameCoordinate::new(
            RecordArtifactFile::ExtentArena { arena: 3 },
            0,
            bytes.len() as u32,
        )
        .unwrap(),
        &bytes,
    )
    .unwrap();
    let old = DerivedFamilyRootDirectoryBinding::new(
        record(8),
        Some(IndexedThroughBlobPublication::new(11, record(3), [4; 32]).unwrap()),
    );
    let next = PersistedDerivedDirectoryRecordBinding::new(
        PersistedBlobSemanticRecordBinding::new(directory, Sha256::digest(&bytes).into(), 12)
            .unwrap(),
        None,
    );
    let replacement = PersistedReleasedDirectoryReplacementV1::new(old, [7; 32], next).unwrap();
    let PersistedPhysicalRecoveryOperation::RecordsDropped {
        binding,
        head_effect: Some(effect),
        ..
    } = base.operation()
    else {
        panic!("head fixture");
    };
    let projection = |replacement, head_effect| {
        let mut frames = base.frames().unwrap().to_vec();
        frames.push(frame.clone());
        let mut routes = base.placements().to_vec();
        if let CurrentPhysicalRecordPlacement::Extent(first) = routes[0] {
            routes[0] = CurrentPhysicalRecordPlacement::Extent(
                DurableExtentRecordPlacement::new_selected(
                    first.record(),
                    first.extent_cell(),
                    first.payload_bytes(),
                    first.arena_range(),
                    SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Blob(
                        BlobRecordKind::ReclaimDescriptorV3,
                    ))
                    .unwrap(),
                )
                .unwrap(),
            );
        }
        routes.push(route);
        PersistedPhysicalRecoveryProjection::new_with_operation(
            11,
            base.root_state().clone(),
            vec![record(9), directory],
            frames,
            routes,
            vec![],
            vec![],
            PersistedPhysicalRecoveryOperation::RecordsDropped {
                binding: *binding,
                head_effect,
                directory_replacement: replacement,
            },
        )
    };
    let valid = projection(Some(replacement), Some((*effect).clone())).unwrap();
    let limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: 2,
        record_identities: 2,
        placements: 2,
        segment_updates: 0,
        manifests: 0,
        total_entries: 4,
        inline_allocations: 0,
    };
    assert_eq!(
        PersistedPhysicalRecoveryProjection::decode(&valid.encode(), limits, format),
        Ok(valid.clone())
    );
    let mut unclassified = valid.placements().to_vec();
    let CurrentPhysicalRecordPlacement::Extent(directory_extent) = unclassified[1] else {
        panic!("directory extent");
    };
    unclassified[1] = CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::legacy_unknown(
            directory_extent.record(),
            directory_extent.extent_cell(),
            directory_extent.payload_bytes(),
            directory_extent.arena_range(),
        )
        .unwrap(),
    );
    assert!(PersistedPhysicalRecoveryProjection::new_with_operation(
        11,
        valid.root_state().clone(),
        valid.record_identities().to_vec(),
        valid.frames().unwrap().to_vec(),
        unclassified,
        vec![],
        vec![],
        valid.operation().clone(),
    )
    .is_none());
    assert!(projection(Some(replacement), None).is_none());
    assert!(projection(None, Some((*effect).clone())).is_none());
    let foreign = PersistedDerivedDirectoryRecordBinding::new(
        PersistedBlobSemanticRecordBinding::new(record(11), [1; 32], 12).unwrap(),
        None,
    );
    assert!(projection(
        PersistedReleasedDirectoryReplacementV1::new(old, [7; 32], foreign),
        Some((*effect).clone()),
    )
    .is_none());
    assert!(PersistedReleasedDirectoryReplacementV1::new(
        DerivedFamilyRootDirectoryBinding::new(record(8), None),
        [7; 32],
        next,
    )
    .is_none());
}
