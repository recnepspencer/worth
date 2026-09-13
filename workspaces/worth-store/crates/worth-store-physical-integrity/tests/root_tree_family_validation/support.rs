use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    BootstrapCatalog, CurrentPhysicalRecordPlacement, CurrentRootCatalogEntry,
    CurrentRootCatalogGeneration, DurableExtentRecordPlacement, ManifestBlockReference,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageId, PhysicalRecordFormatDeclaration, PhysicalRootRoutingBlock, PhysicalSegmentId,
    PhysicalSegmentMembershipBlock, PhysicalTreeIdentity, RecordSegmentPageManifestEntry,
    RootRoutingBlockScopeIdentity, SegmentManifestBlockReference,
    SegmentMembershipBlockScopeIdentity,
};
use worth_store_physical_integrity::{
    validate_root_routing_block, validate_segment_membership_block,
    IntegrityValidatedRootRoutingBlock, IntegrityValidatedSegmentMembershipBlock,
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalDamageLocalization, PhysicalFormatField, PhysicalIntegrityObservationCounters,
    PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass,
    RootRoutingBlockIntegrityValidation, SegmentMembershipBlockIntegrityValidation,
    UntrustedPhysicalArtifact,
};

pub const BOOTSTRAP_OFFSET: u64 = 4_096;
pub const ROOT_BLOCK_OFFSET: u64 = 16_384;
pub const SEGMENT_BLOCK_OFFSET: u64 = 32_768;

pub fn store(byte: u8) -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

pub fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

pub fn bootstrap_bytes(store: StableStoreIdentity) -> Vec<u8> {
    BootstrapCatalog::new(
        store,
        format(),
        CurrentRootCatalogEntry::new(CurrentRootCatalogGeneration::new(11).unwrap()),
    )
    .encode()
    .to_vec()
}

pub fn bootstrap_scope(store: StableStoreIdentity, offset: u64) -> PhysicalArtifactScope {
    PhysicalArtifactScope::bootstrap_catalog(
        store,
        format(),
        PhysicalByteRange::new(offset, 82).unwrap(),
    )
}

pub fn root_leaf() -> PhysicalRootRoutingBlock {
    let record = PersistedRecordIdentity::new([0xa1; 16], 5).unwrap();
    let extent = PhysicalExtentId::from_raw(19).unwrap();
    let generation = PhysicalGeneration::from_raw(7).unwrap();
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(extent)
        .with_extent_generation(generation);
    let placement = DurableExtentRecordPlacement::new(record, cell, 23)
        .map(CurrentPhysicalRecordPlacement::Extent)
        .unwrap();
    PhysicalRootRoutingBlock::leaf(71, 11, 3, vec![placement], 8).unwrap()
}

pub fn root_branch(child: ManifestBlockReference) -> PhysicalRootRoutingBlock {
    PhysicalRootRoutingBlock::branch(71, 12, 4, 1, vec![child], 8).unwrap()
}

pub fn root_scope(
    store: StableStoreIdentity,
    block: &PhysicalRootRoutingBlock,
    bytes: &[u8],
    offset: u64,
) -> PhysicalArtifactScope {
    let reference = block.reference(independent_crc32c(&[bytes]));
    root_scope_with_reference(store, reference, bytes.len() as u64, offset)
}

pub fn root_scope_with_reference(
    store: StableStoreIdentity,
    reference: ManifestBlockReference,
    length: u64,
    offset: u64,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::root_routing_block(
        store,
        format(),
        RootRoutingBlockScopeIdentity::new(PhysicalTreeIdentity::new(71).unwrap(), reference),
        PhysicalByteRange::new(offset, length).unwrap(),
    )
}

pub fn segment_leaf() -> PhysicalSegmentMembershipBlock {
    let segment = PhysicalSegmentId::from_raw(13).unwrap();
    let page = PhysicalPageId::from_raw(17).unwrap();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let entry = RecordSegmentPageManifestEntry::new(
        authority
            .page_cell(segment, page)
            .with_page_generation(PhysicalGeneration::from_raw(5).unwrap()),
        authority
            .segment_cell(segment)
            .with_segment_generation(PhysicalGeneration::from_raw(6).unwrap()),
        2,
        1,
    )
    .unwrap();
    PhysicalSegmentMembershipBlock::leaf(73, 11, 5, vec![entry], 8).unwrap()
}

pub fn segment_branch(child: SegmentManifestBlockReference) -> PhysicalSegmentMembershipBlock {
    PhysicalSegmentMembershipBlock::branch(73, 12, 6, 1, vec![child], 8).unwrap()
}

pub fn segment_scope(
    store: StableStoreIdentity,
    block: &PhysicalSegmentMembershipBlock,
    bytes: &[u8],
    offset: u64,
) -> PhysicalArtifactScope {
    let reference = block.reference(independent_crc32c(&[bytes]));
    segment_scope_with_reference(store, reference, bytes.len() as u64, offset)
}

pub fn segment_scope_with_reference(
    store: StableStoreIdentity,
    reference: SegmentManifestBlockReference,
    length: u64,
    offset: u64,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::segment_membership_block(
        store,
        format(),
        SegmentMembershipBlockScopeIdentity::new(PhysicalTreeIdentity::new(73).unwrap(), reference),
        PhysicalByteRange::new(offset, length).unwrap(),
    )
}

pub fn reseal_durable_frame(bytes: &mut [u8]) {
    let checksum = independent_crc32c(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
}

pub fn independent_crc32c(parts: &[&[u8]]) -> u32 {
    let mut crc = !0_u32;
    for byte in parts.iter().flat_map(|part| part.iter().copied()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

pub fn field_range(scope: PhysicalArtifactScope, offset: u64, length: u64) -> PhysicalByteRange {
    PhysicalByteRange::new(scope.byte_range().offset() + offset, length).unwrap()
}

pub fn assert_root_mutation_contract(bytes: &[u8], scope: PhysicalArtifactScope) {
    assert_inner_and_scope_crc(
        bytes,
        scope,
        scope
            .root_routing_block_identity()
            .expect("root mutation uses root scope")
            .reference()
            .checksum(),
    );
}

pub fn assert_root_checksum_mutation_contract(bytes: &[u8], scope: PhysicalArtifactScope) {
    assert_invalid_inner_and_refreshed_scope_crc(
        bytes,
        scope,
        scope
            .root_routing_block_identity()
            .expect("root checksum mutation uses root scope")
            .reference()
            .checksum(),
    );
}

pub fn assert_segment_mutation_contract(bytes: &[u8], scope: PhysicalArtifactScope) {
    assert_inner_and_scope_crc(
        bytes,
        scope,
        scope
            .segment_membership_block_identity()
            .expect("membership mutation uses membership scope")
            .reference()
            .checksum(),
    );
}

pub fn assert_segment_checksum_mutation_contract(bytes: &[u8], scope: PhysicalArtifactScope) {
    assert_invalid_inner_and_refreshed_scope_crc(
        bytes,
        scope,
        scope
            .segment_membership_block_identity()
            .expect("membership checksum mutation uses membership scope")
            .reference()
            .checksum(),
    );
}

fn assert_inner_and_scope_crc(bytes: &[u8], scope: PhysicalArtifactScope, scope_crc: u32) {
    assert_eq!(scope.byte_range().length(), bytes.len() as u64);
    assert_eq!(
        u32::from_le_bytes(bytes[44..48].try_into().unwrap()),
        independent_crc32c(&[&bytes[..44], &bytes[48..]])
    );
    assert_eq!(scope_crc, independent_crc32c(&[bytes]));
}

fn assert_invalid_inner_and_refreshed_scope_crc(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    scope_crc: u32,
) {
    assert_eq!(scope.byte_range().length(), bytes.len() as u64);
    assert_ne!(
        u32::from_le_bytes(bytes[44..48].try_into().unwrap()),
        independent_crc32c(&[&bytes[..44], &bytes[48..]])
    );
    assert_eq!(scope_crc, independent_crc32c(&[bytes]));
}

pub fn assert_damage(
    rejection: PhysicalIntegrityRejection,
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    assert_eq!(
        rejection,
        PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
            scope,
            cause,
            range,
            field,
            blast_radius,
        ))
    );
}

pub fn assert_rejected_counters(
    counters: PhysicalIntegrityObservationCounters,
    family: worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily,
    byte_count: u64,
    cause: PhysicalDamageCause,
) {
    assert_eq!(counters.family(), family);
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), byte_count);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Damaged(cause)),
        1
    );
}

pub fn validate_root_intact<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedRootRoutingBlock<'media> {
    match validate_root_routing_block(artifact, scope).0 {
        RootRoutingBlockIntegrityValidation::Intact(validated) => validated,
        RootRoutingBlockIntegrityValidation::Rejected(rejection) => {
            panic!("clean root-routing block rejected: {rejection:?}")
        }
    }
}

pub fn root_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    match validate_root_routing_block(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope) {
        (RootRoutingBlockIntegrityValidation::Rejected(rejection), counters) => {
            (rejection, counters)
        }
        (RootRoutingBlockIntegrityValidation::Intact(_), _) => {
            panic!("damaged root-routing block validated")
        }
    }
}

pub fn validate_segment_intact<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedSegmentMembershipBlock<'media> {
    match validate_segment_membership_block(artifact, scope).0 {
        SegmentMembershipBlockIntegrityValidation::Intact(validated) => validated,
        SegmentMembershipBlockIntegrityValidation::Rejected(rejection) => {
            panic!("clean segment-membership block rejected: {rejection:?}")
        }
    }
}

pub fn segment_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    match validate_segment_membership_block(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    ) {
        (SegmentMembershipBlockIntegrityValidation::Rejected(rejection), counters) => {
            (rejection, counters)
        }
        (SegmentMembershipBlockIntegrityValidation::Intact(_), _) => {
            panic!("damaged segment-membership block validated")
        }
    }
}
