use worth_store_physical_format::{
    certification_crc32c_invocations, durable_artifact_checksum, DurableArtifactCrc32c,
    DurableFreeSpaceManifestHeader, FreeSpaceHeaderScopeIdentity,
    FreeSpaceMembershipBlockScopeIdentity, PhysicalFreeSpaceMembershipBlock, PhysicalGeneration,
    PhysicalTreeIdentity, RecordAllocationClass, RecordArtifactFile, RecordFrameCoordinate,
    RecordFreeSpaceManifestEntry,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use super::super::free_space::{
    admit_resident_free_space_header, admit_resident_free_space_membership_block,
};
use super::super::load::ResidentAdmissionContext;
use super::support::*;
use crate::physical_runtime::ResidentAdmissionCounterCells;

#[test]
fn free_space_header_projection_does_not_rehash_and_invalidation_requires_actual_validation() {
    let store = store(81);
    let format = format();
    let header = DurableFreeSpaceManifestHeader::new(2, 71, 4, 4, 0, 1, 1, 1, 1, None).unwrap();
    let bytes = header.encode(format);
    let scope = PhysicalArtifactScope::free_space_header(
        store,
        format,
        FreeSpaceHeaderScopeIdentity::new(
            PhysicalGeneration::from_raw(2).unwrap(),
            PhysicalTreeIdentity::new(71).unwrap(),
            None,
            DurableArtifactCrc32c::new(durable_artifact_checksum(&bytes)),
        ),
        PhysicalByteRange::new(0, bytes.len() as u64).unwrap(),
    );
    let coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::FreeSpaceManifest { generation: 2 },
        0,
        bytes.len() as u32,
    )
    .unwrap();
    let (pool, _allocation, lease) = loaded_frame(store, coordinate, &bytes);
    let lifecycle = lifecycle();
    let counters = ResidentAdmissionCounterCells::default();
    let project = || {
        let before = certification_crc32c_invocations();
        let context = ResidentAdmissionContext::new(lifecycle.observation_state(), &counters);
        let admitted = admit_resident_free_space_header(&lease, scope, context.clone()).unwrap();
        let after_admission = certification_crc32c_invocations();
        let projected = admitted
            .with_owner_decoder(context, |view| view.project_header(4))
            .unwrap()
            .unwrap();
        assert_eq!(projected, (header.clone(), format));
        assert_eq!(certification_crc32c_invocations(), after_admission);
        after_admission - before
    };
    let cold = project();
    assert!(cold > 0);
    assert_eq!(project(), 0);
    pool.invalidate_integrity_validation_for_runtime_transition();
    assert_eq!(project(), cold);
    assert_eq!(project(), 0);
}

#[test]
fn free_space_membership_projection_does_not_rehash_and_invalidation_requires_actual_validation() {
    let store = store(82);
    let format = format();
    let entry =
        RecordFreeSpaceManifestEntry::new(RecordAllocationClass::InlinePage, 1, 2, 3, 2).unwrap();
    let block = PhysicalFreeSpaceMembershipBlock::leaf(71, 2, 1, vec![entry], 4).unwrap();
    let bytes = block.encode(format);
    let scope = PhysicalArtifactScope::free_space_membership_block(
        store,
        format,
        FreeSpaceMembershipBlockScopeIdentity::new(
            PhysicalTreeIdentity::new(71).unwrap(),
            block.reference(durable_artifact_checksum(&bytes)),
        ),
        PhysicalByteRange::new(0, bytes.len() as u64).unwrap(),
    );
    let coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::FreeSpaceMembershipBlock {
            generation: 2,
            block: 1,
        },
        0,
        bytes.len() as u32,
    )
    .unwrap();
    let (pool, _allocation, lease) = loaded_frame(store, coordinate, &bytes);
    let lifecycle = lifecycle();
    let counters = ResidentAdmissionCounterCells::default();
    let project = || {
        let before = certification_crc32c_invocations();
        let context = ResidentAdmissionContext::new(lifecycle.observation_state(), &counters);
        let admitted =
            admit_resident_free_space_membership_block(&lease, scope, context.clone()).unwrap();
        let after_admission = certification_crc32c_invocations();
        let projected = admitted
            .with_owner_decoder(context, |view| view.project_block(4))
            .unwrap()
            .unwrap();
        assert_eq!(projected, block);
        assert_eq!(certification_crc32c_invocations(), after_admission);
        after_admission - before
    };
    let cold = project();
    assert!(cold > 0);
    assert_eq!(project(), 0);
    pool.invalidate_integrity_validation_for_runtime_transition();
    assert_eq!(project(), cold);
    assert_eq!(project(), 0);
}
