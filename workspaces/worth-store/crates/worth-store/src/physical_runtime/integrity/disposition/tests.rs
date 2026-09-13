use std::num::{NonZeroU32, NonZeroU64};

use worth_foundational::PhysicalIntegrityPosture;
use worth_store_buffer_pool::{
    PhysicalFrameAccess, PhysicalFrameKey, PhysicalFrameLease, PhysicalOperationAllocationScope,
    PhysicalResidencyLimits, PhysicalResidencyPool, PhysicalSpeculativeWorkKind,
};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, FreeSpaceBlockReference, FreeSpaceKey, PhysicalPageSizeClass,
    PhysicalRecordFormatDeclaration, RecordAllocationClass, RecordArtifactFile,
    RecordFrameCoordinate,
};
use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalByteRange, PhysicalIntegrityObservationOutcome,
};

use super::*;

#[test]
fn intact_authority_requires_the_exact_owner_source_family_and_scope() {
    let format = format();
    let bytes = manifest_bytes(11, format);
    let scope = manifest_scope(store(1), format, 11, bytes.len());
    let source = resident_manifest_source(store(1), 11, &bytes);

    let disposition = project_resident_root_manifest_authority(&source, scope).unwrap();
    assert!(matches!(
        disposition.validator_outcome(),
        PhysicalIntegrityObservationOutcome::Intact(observed) if observed == scope
    ));
    assert_eq!(
        disposition.owner_role(),
        Some(PhysicalArtifactRoleDisposition::IntactAuthority(
            IntactPhysicalAuthorityObservation::new(scope),
        ))
    );

    for substituted in [
        manifest_scope(store(2), format, 11, bytes.len()),
        manifest_scope(store(1), format, 12, bytes.len()),
        PhysicalArtifactScope::root_manifest(
            store(1),
            format,
            11,
            PhysicalByteRange::new(1, bytes.len() as u64).unwrap(),
        )
        .unwrap(),
    ] {
        assert!(matches!(
            project_resident_root_manifest_authority(&source, substituted),
            Err(StoreOwnerDispositionAdapterDenial::SourceScopeSubstitution)
        ));
    }

    assert_selector_family_binding(format, bytes.len());
}

#[test]
fn authority_disposition_is_bound_to_the_validated_source_incarnation() {
    let format = format();
    let scope = manifest_scope(store(3), format, 21, manifest_bytes(21, format).len());

    let clean_bytes = manifest_bytes(21, format);
    let clean_source = resident_manifest_source(store(3), 21, &clean_bytes);
    let clean = project_resident_root_manifest_authority(&clean_source, scope).unwrap();
    assert!(matches!(
        clean.owner_role(),
        Some(PhysicalArtifactRoleDisposition::IntactAuthority(_))
    ));

    let mut damaged_bytes = clean_bytes;
    damaged_bytes[80] ^= 0x40;
    let damaged_source = resident_manifest_source(store(3), 21, &damaged_bytes);
    let damaged = project_resident_root_manifest_authority(&damaged_source, scope).unwrap();
    assert!(matches!(
        damaged.owner_role(),
        Some(PhysicalArtifactRoleDisposition::DamagedAuthority(_))
    ));

    let substituted = manifest_scope(store(3), format, 22, damaged_bytes.len());
    assert!(matches!(
        project_resident_root_manifest_authority(&damaged_source, substituted),
        Err(StoreOwnerDispositionAdapterDenial::SourceScopeSubstitution)
    ));
}

#[test]
fn unsupported_version_remains_distinct_from_corruption_and_gets_no_owner_role() {
    let format = format();
    let mut unsupported_bytes = manifest_bytes(31, format);
    let scope = manifest_scope(store(4), format, 31, unsupported_bytes.len());
    unsupported_bytes[10..12].copy_from_slice(&2_u16.to_le_bytes());
    reseal_durable_frame(&mut unsupported_bytes);
    let unsupported_source = resident_manifest_source(store(4), 31, &unsupported_bytes);

    let unsupported = project_resident_root_manifest_authority(&unsupported_source, scope).unwrap();
    assert_eq!(unsupported.owner_role(), None);
    assert_eq!(
        unsupported.validator_outcome().foundational_posture(),
        PhysicalIntegrityPosture::Unsupported
    );

    let mut damaged_bytes = manifest_bytes(31, format);
    damaged_bytes[80] ^= 0x20;
    let damaged_source = resident_manifest_source(store(4), 31, &damaged_bytes);
    let damaged = project_resident_root_manifest_authority(&damaged_source, scope).unwrap();
    assert_eq!(
        damaged.validator_outcome().foundational_posture(),
        PhysicalIntegrityPosture::Damaged
    );
    assert!(matches!(
        damaged.owner_role(),
        Some(PhysicalArtifactRoleDisposition::DamagedAuthority(_))
    ));
}

fn assert_selector_family_binding(format: PhysicalRecordFormatDeclaration, length: usize) {
    let selector_bytes = vec![0; length];
    let current_source = resident_artifact_source(
        store(1),
        RecordArtifactFile::CurrentRootSelector,
        &selector_bytes,
    );
    let previous_source = resident_artifact_source(
        store(1),
        RecordArtifactFile::PreviousRootSelector,
        &selector_bytes,
    );
    let range = PhysicalByteRange::new(0, length as u64).unwrap();
    let current_scope = PhysicalArtifactScope::current_root_selector(store(1), format, range);
    let previous_scope = PhysicalArtifactScope::previous_root_selector(store(1), format, range);

    assert!(
        project_resident_current_root_selector_authority(&current_source, current_scope).is_ok()
    );
    assert!(
        project_resident_previous_root_selector_authority(&previous_source, previous_scope).is_ok()
    );
    assert!(matches!(
        project_resident_current_root_selector_authority(&current_source, previous_scope),
        Err(StoreOwnerDispositionAdapterDenial::SourceScopeSubstitution)
    ));
    assert!(matches!(
        project_resident_previous_root_selector_authority(&previous_source, current_scope),
        Err(StoreOwnerDispositionAdapterDenial::SourceScopeSubstitution)
    ));
}

fn resident_manifest_source(
    store: StableStoreIdentity,
    generation: u64,
    bytes: &[u8],
) -> PhysicalFrameLease {
    resident_artifact_source(
        store,
        RecordArtifactFile::RootManifest { generation },
        bytes,
    )
}

fn resident_artifact_source(
    store: StableStoreIdentity,
    artifact: RecordArtifactFile,
    bytes: &[u8],
) -> PhysicalFrameLease {
    let length = u32::try_from(bytes.len()).unwrap();
    let pool = PhysicalResidencyPool::open(store, residency_limits(length)).unwrap();
    let allocation = pool
        .begin_operation(
            PhysicalOperationAllocationScope::ForegroundRead,
            NonZeroU64::new(u64::from(length)).unwrap(),
        )
        .unwrap();
    let coordinate = RecordFrameCoordinate::new(artifact, 0, length).unwrap();
    let key = PhysicalFrameKey::new(store, coordinate);
    let PhysicalFrameAccess::Fault(fault) = pool.access_frame(&allocation, key).unwrap() else {
        panic!("fresh resident pool must yield frame-fault ownership");
    };
    fault
        .load(|target| {
            target.copy_from_slice(bytes);
            Ok::<(), ()>(())
        })
        .unwrap()
}

fn residency_limits(frame_bytes: u32) -> PhysicalResidencyLimits {
    let bytes = NonZeroU64::new(u64::from(frame_bytes)).unwrap();
    let count = NonZeroU32::new(2).unwrap();
    let mut builder = PhysicalResidencyLimits::builder()
        .total_bytes(NonZeroU64::new(u64::from(frame_bytes) * 3 + 4096).unwrap())
        .resident_bytes(bytes)
        .metadata_bytes(NonZeroU64::new(4096).unwrap())
        .frame_entries(count)
        .pinned_frames(count)
        .pin_leases(count)
        .dirty_frames(count)
        .dirty_replacement_bytes(bytes)
        .operation_bytes(bytes);
    for scope in [
        PhysicalOperationAllocationScope::ForegroundRead,
        PhysicalOperationAllocationScope::ForegroundWrite,
        PhysicalOperationAllocationScope::Recovery,
        PhysicalOperationAllocationScope::Scrub,
        PhysicalOperationAllocationScope::Maintenance,
        PhysicalOperationAllocationScope::Verification,
        PhysicalOperationAllocationScope::Blob,
    ] {
        builder = builder.scope_bytes(scope, bytes);
    }
    for kind in [
        PhysicalSpeculativeWorkKind::Prefetch,
        PhysicalSpeculativeWorkKind::ReadAhead,
        PhysicalSpeculativeWorkKind::WriteBehind,
    ] {
        builder = builder.speculative_frames(kind, count);
    }
    builder.admit(NonZeroU64::MIN).unwrap()
}

fn store(byte: u8) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder()
        .page_size(PhysicalPageSizeClass::KiB16)
        .admit()
        .unwrap()
}

fn manifest_bytes(generation: u64, format: PhysicalRecordFormatDeclaration) -> Vec<u8> {
    let key = FreeSpaceKey::new(RecordAllocationClass::InlinePage, 1).unwrap();
    let free_space_root = FreeSpaceBlockReference::new(generation, 1, 0, 41, key, key).unwrap();
    DurablePhysicalRootManifest::builder(generation, 71, 2, 43)
        .free_space_root(Some(free_space_root))
        .admit()
        .unwrap()
        .encode(format)
}

fn manifest_scope(
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    generation: u64,
    length: usize,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::root_manifest(
        store,
        format,
        generation,
        PhysicalByteRange::new(0, length as u64).unwrap(),
    )
    .unwrap()
}

fn reseal_durable_frame(bytes: &mut [u8]) {
    let checksum = independent_crc32c(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
}

fn independent_crc32c(parts: &[&[u8]]) -> u32 {
    let mut crc = !0_u32;
    for byte in parts.iter().flat_map(|part| part.iter().copied()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
