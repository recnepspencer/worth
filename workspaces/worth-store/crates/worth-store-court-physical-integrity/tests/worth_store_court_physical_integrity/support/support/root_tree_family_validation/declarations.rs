pub(crate) use {worth_store_physical_format::store_namespace::ProposedStoreIdentity, worth_store_physical_format::store_namespace::StableStoreIdentity, worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord, worth_store_physical_format::store_namespace::StoreNamespaceVersion};
pub(crate) use {worth_store_physical_format::BootstrapCatalog, worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::CurrentRootCatalogEntry, worth_store_physical_format::CurrentRootCatalogGeneration, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::ExtentArenaId, worth_store_physical_format::ExtentArenaRange, worth_store_physical_format::ManifestBlockReference, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::PhysicalRootRoutingBlock, worth_store_physical_format::PhysicalSegmentId, worth_store_physical_format::PhysicalSegmentMembershipBlock, worth_store_physical_format::PhysicalTreeIdentity, worth_store_physical_format::RecordSegmentPageManifestEntry, worth_store_physical_format::RootRoutingBlockScopeIdentity, worth_store_physical_format::SegmentManifestBlockReference, worth_store_physical_format::SegmentMembershipBlockScopeIdentity};
pub(crate) use {worth_store_physical_integrity::validate_root_routing_block, worth_store_physical_integrity::validate_segment_membership_block, worth_store_physical_integrity::IntegrityValidatedRootRoutingBlock, worth_store_physical_integrity::IntegrityValidatedSegmentMembershipBlock, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalDamageLocalization, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityObservationCounters, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::RootRoutingBlockIntegrityValidation, worth_store_physical_integrity::SegmentMembershipBlockIntegrityValidation, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) const BOOTSTRAP_OFFSET: u64 = 4_096;
pub(crate) const ROOT_BLOCK_OFFSET: u64 = 16_384;
pub(crate) const SEGMENT_BLOCK_OFFSET: u64 = 32_768;

pub(crate) fn store(byte: u8) -> StableStoreIdentity {
    worth_store_physical_format::store_namespace::StoreNamespaceIdentityRecord::new(
        worth_store_physical_format::store_namespace::StoreNamespaceVersion::CURRENT,
        worth_store_physical_format::store_namespace::ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
    .published_identity()
}

pub(crate) fn format() -> PhysicalRecordFormatDeclaration {
    worth_store_physical_format::PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

pub(crate) fn bootstrap_bytes(store: StableStoreIdentity) -> Vec<u8> {
    worth_store_physical_format::BootstrapCatalog::new(
        store,
        format(),
        worth_store_physical_format::CurrentRootCatalogEntry::new(worth_store_physical_format::CurrentRootCatalogGeneration::new(11).unwrap()),
    )
    .encode()
    .to_vec()
}

pub(crate) fn bootstrap_scope(store: StableStoreIdentity, offset: u64) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::bootstrap_catalog(
        store,
        format(),
        worth_store_physical_integrity::PhysicalByteRange::new(offset, 82).unwrap(),
    )
}

pub(crate) fn root_leaf() -> PhysicalRootRoutingBlock {
    let record = worth_store_physical_format::PersistedRecordIdentity::new([0xa1; 16], 5).unwrap();
    let extent = worth_store_physical_format::PhysicalExtentId::from_raw(19).unwrap();
    let generation = worth_store_physical_format::PhysicalGeneration::from_raw(7).unwrap();
    let cell = worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(extent)
        .with_extent_generation(generation);
    let placement = worth_store_physical_format::DurableExtentRecordPlacement::legacy_unknown(
        record,
        cell,
        23,
        worth_store_physical_format::ExtentArenaRange::new(worth_store_physical_format::ExtentArenaId::new(1).unwrap(), 0, 32_768).unwrap(),
    )
    .map(worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent)
    .unwrap();
    worth_store_physical_format::PhysicalRootRoutingBlock::leaf(71, 11, 3, vec![placement], 8).unwrap()
}

pub(crate) fn root_branch(child: ManifestBlockReference) -> PhysicalRootRoutingBlock {
    worth_store_physical_format::PhysicalRootRoutingBlock::branch(71, 12, 4, 1, vec![child], 8).unwrap()
}

pub(crate) fn root_scope(
    store: StableStoreIdentity,
    block: &PhysicalRootRoutingBlock,
    bytes: &[u8],
    offset: u64,
) -> PhysicalArtifactScope {
    let reference = block.reference(independent_crc32c(&[bytes]));
    root_scope_with_reference(store, reference, bytes.len() as u64, offset)
}

pub(crate) fn root_scope_with_reference(
    store: StableStoreIdentity,
    reference: ManifestBlockReference,
    length: u64,
    offset: u64,
) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::root_routing_block(
        store,
        format(),
        worth_store_physical_format::RootRoutingBlockScopeIdentity::new(worth_store_physical_format::PhysicalTreeIdentity::new(71).unwrap(), reference),
        worth_store_physical_integrity::PhysicalByteRange::new(offset, length).unwrap(),
    )
}

pub(crate) fn segment_leaf() -> PhysicalSegmentMembershipBlock {
    let segment = worth_store_physical_format::PhysicalSegmentId::from_raw(13).unwrap();
    let page = worth_store_physical_format::PhysicalPageId::from_raw(17).unwrap();
    let authority = worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format();
    let entry = worth_store_physical_format::RecordSegmentPageManifestEntry::new(
        authority
            .page_cell(segment, page)
            .with_page_generation(worth_store_physical_format::PhysicalGeneration::from_raw(5).unwrap()),
        authority
            .segment_cell(segment)
            .with_segment_generation(worth_store_physical_format::PhysicalGeneration::from_raw(6).unwrap()),
        2,
        1,
    )
    .unwrap();
    worth_store_physical_format::PhysicalSegmentMembershipBlock::leaf(73, 11, 5, vec![entry], 8).unwrap()
}

pub(crate) fn segment_branch(child: SegmentManifestBlockReference) -> PhysicalSegmentMembershipBlock {
    worth_store_physical_format::PhysicalSegmentMembershipBlock::branch(73, 12, 6, 1, vec![child], 8).unwrap()
}

pub(crate) fn segment_scope(
    store: StableStoreIdentity,
    block: &PhysicalSegmentMembershipBlock,
    bytes: &[u8],
    offset: u64,
) -> PhysicalArtifactScope {
    let reference = block.reference(independent_crc32c(&[bytes]));
    segment_scope_with_reference(store, reference, bytes.len() as u64, offset)
}

pub(crate) fn segment_scope_with_reference(
    store: StableStoreIdentity,
    reference: SegmentManifestBlockReference,
    length: u64,
    offset: u64,
) -> PhysicalArtifactScope {
    worth_store_physical_integrity::PhysicalArtifactScope::segment_membership_block(
        store,
        format(),
        worth_store_physical_format::SegmentMembershipBlockScopeIdentity::new(worth_store_physical_format::PhysicalTreeIdentity::new(73).unwrap(), reference),
        worth_store_physical_integrity::PhysicalByteRange::new(offset, length).unwrap(),
    )
}

pub(crate) fn reseal_durable_frame(bytes: &mut [u8]) {
    let checksum = independent_crc32c(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
}

pub(crate) fn independent_crc32c(parts: &[&[u8]]) -> u32 {
    let mut crc = !0_u32;
    for byte in parts.iter().flat_map(|part| part.iter().copied()) {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

pub(crate) fn field_range(scope: PhysicalArtifactScope, offset: u64, length: u64) -> PhysicalByteRange {
    worth_store_physical_integrity::PhysicalByteRange::new(scope.byte_range().offset() + offset, length).unwrap()
}

pub(crate) fn assert_root_mutation_contract(bytes: &[u8], scope: PhysicalArtifactScope) {
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

pub(crate) fn assert_root_checksum_mutation_contract(bytes: &[u8], scope: PhysicalArtifactScope) {
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

pub(crate) fn assert_segment_mutation_contract(bytes: &[u8], scope: PhysicalArtifactScope) {
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

pub(crate) fn assert_segment_checksum_mutation_contract(bytes: &[u8], scope: PhysicalArtifactScope) {
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

pub(crate) fn assert_inner_and_scope_crc(bytes: &[u8], scope: PhysicalArtifactScope, scope_crc: u32) {
    assert_eq!(scope.byte_range().length(), bytes.len() as u64);
    assert_eq!(
        u32::from_le_bytes(bytes[44..48].try_into().unwrap()),
        independent_crc32c(&[&bytes[..44], &bytes[48..]])
    );
    assert_eq!(scope_crc, independent_crc32c(&[bytes]));
}

pub(crate) fn assert_invalid_inner_and_refreshed_scope_crc(
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

pub(crate) fn assert_damage(
    rejection: PhysicalIntegrityRejection,
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    assert_eq!(
        rejection,
        worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(worth_store_physical_integrity::PhysicalDamageLocalization::new(
            scope,
            cause,
            range,
            field,
            blast_radius,
        ))
    );
}

pub(crate) fn assert_rejected_counters(
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
        counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Damaged(cause)),
        1
    );
}

pub(crate) fn validate_root_intact<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedRootRoutingBlock<'media> {
    match validate_root_routing_block(artifact, scope).0 {
        worth_store_physical_integrity::RootRoutingBlockIntegrityValidation::Intact(validated) => validated,
        worth_store_physical_integrity::RootRoutingBlockIntegrityValidation::Rejected(rejection) => {
            panic!("clean root-routing block rejected: {rejection:?}")
        }
    }
}

pub(crate) fn root_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    match validate_root_routing_block(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope) {
        (worth_store_physical_integrity::RootRoutingBlockIntegrityValidation::Rejected(rejection), counters) => {
            (rejection, counters)
        }
        (worth_store_physical_integrity::RootRoutingBlockIntegrityValidation::Intact(_), _) => {
            panic!("damaged root-routing block validated")
        }
    }
}

pub(crate) fn validate_segment_intact<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> IntegrityValidatedSegmentMembershipBlock<'media> {
    match validate_segment_membership_block(artifact, scope).0 {
        worth_store_physical_integrity::SegmentMembershipBlockIntegrityValidation::Intact(validated) => validated,
        worth_store_physical_integrity::SegmentMembershipBlockIntegrityValidation::Rejected(rejection) => {
            panic!("clean segment-membership block rejected: {rejection:?}")
        }
    }
}

pub(crate) fn segment_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    PhysicalIntegrityObservationCounters,
) {
    match validate_segment_membership_block(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    ) {
        (worth_store_physical_integrity::SegmentMembershipBlockIntegrityValidation::Rejected(rejection), counters) => {
            (rejection, counters)
        }
        (worth_store_physical_integrity::SegmentMembershipBlockIntegrityValidation::Intact(_), _) => {
            panic!("damaged segment-membership block validated")
        }
    }
}