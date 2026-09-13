use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{PhysicalPageSizeClass, PhysicalRecordFormatDeclaration};
use worth_store_physical_integrity::{
    validate_bootstrap_catalog, BootstrapCatalogIntegrityValidation, PhysicalArtifactScope,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, assert_rejected_counters, assert_root_mutation_contract,
    assert_segment_mutation_contract, bootstrap_bytes, bootstrap_scope, field_range, format,
    reseal_durable_frame, root_leaf, root_rejection, root_scope, segment_leaf, segment_rejection,
    segment_scope, store, BOOTSTRAP_OFFSET, ROOT_BLOCK_OFFSET, SEGMENT_BLOCK_OFFSET,
};

#[test]
fn root_short_and_reserved_prefixes_reject_without_panicking_at_exact_fields() {
    let block = root_leaf();
    let clean = block.encode(format());
    let mut short = clean[..64].to_vec();
    short[24..28].copy_from_slice(&16_u32.to_le_bytes());
    reseal_durable_frame(&mut short);
    let short_scope = root_scope(store(7), &block, &short, ROOT_BLOCK_OFFSET);
    assert_root_mutation_contract(&short, short_scope);
    assert_root_damage(
        &short,
        short_scope,
        PhysicalDamageCause::MalformedStructure,
        field_range(short_scope, 48, 16),
        Some(PhysicalFormatField::Reserved),
        PhysicalBlastRadius::CompleteArtifact,
    );

    for (offset, length) in [(69_usize, 3_u64), (80, 8)] {
        let mut reserved = clean[..88].to_vec();
        reserved[24..28].copy_from_slice(&40_u32.to_le_bytes());
        reserved[offset] = 1;
        reseal_durable_frame(&mut reserved);
        let scope = root_scope(store(7), &block, &reserved, ROOT_BLOCK_OFFSET);
        assert_root_mutation_contract(&reserved, scope);
        assert_root_damage(
            &reserved,
            scope,
            PhysicalDamageCause::MalformedStructure,
            field_range(scope, offset as u64, length),
            Some(PhysicalFormatField::Reserved),
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn membership_short_and_reserved_prefixes_reject_without_panicking_at_exact_fields() {
    let block = segment_leaf();
    let clean = block.encode(format());
    let mut short = clean[..64].to_vec();
    short[24..28].copy_from_slice(&16_u32.to_le_bytes());
    reseal_durable_frame(&mut short);
    let short_scope = segment_scope(store(7), &block, &short, SEGMENT_BLOCK_OFFSET);
    assert_segment_mutation_contract(&short, short_scope);
    assert_segment_damage(
        &short,
        short_scope,
        PhysicalDamageCause::MalformedStructure,
        field_range(short_scope, 48, 16),
        Some(PhysicalFormatField::Reserved),
        PhysicalBlastRadius::CompleteArtifact,
    );

    for (offset, length) in [(69_usize, 3_u64), (80, 8)] {
        let mut reserved = clean[..88].to_vec();
        reserved[24..28].copy_from_slice(&40_u32.to_le_bytes());
        reserved[offset] = 1;
        reseal_durable_frame(&mut reserved);
        let scope = segment_scope(store(7), &block, &reserved, SEGMENT_BLOCK_OFFSET);
        assert_segment_mutation_contract(&reserved, scope);
        assert_segment_damage(
            &reserved,
            scope,
            PhysicalDamageCause::MalformedStructure,
            field_range(scope, offset as u64, length),
            Some(PhysicalFormatField::Reserved),
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn zero_routing_block_identity_copies_are_not_misreported_as_length_damage() {
    let root = root_leaf();
    let mut root_bytes = root.encode(format());
    root_bytes[28..36].fill(0);
    root_bytes[56..64].fill(0);
    reseal_durable_frame(&mut root_bytes);
    let root_scope = root_scope(store(7), &root, &root_bytes, ROOT_BLOCK_OFFSET);
    assert_root_mutation_contract(&root_bytes, root_scope);
    assert_root_damage(
        &root_bytes,
        root_scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        field_range(root_scope, 28, 36),
        Some(PhysicalFormatField::BlockIdentity),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let segment = segment_leaf();
    let mut segment_bytes = segment.encode(format());
    segment_bytes[28..36].fill(0);
    segment_bytes[56..64].fill(0);
    reseal_durable_frame(&mut segment_bytes);
    let segment_scope = segment_scope(store(7), &segment, &segment_bytes, SEGMENT_BLOCK_OFFSET);
    assert_segment_mutation_contract(&segment_bytes, segment_scope);
    assert_segment_damage(
        &segment_bytes,
        segment_scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        field_range(segment_scope, 28, 36),
        Some(PhysicalFormatField::BlockIdentity),
        PhysicalBlastRadius::ReachableSubtree,
    );
}

#[test]
fn zero_bootstrap_generation_and_supported_payload_format_localize_truthfully() {
    let store = store(7);
    let scope = bootstrap_scope(store, BOOTSTRAP_OFFSET);
    let mut zero_generation = bootstrap_bytes(store);
    zero_generation[28..36].fill(0);
    zero_generation[64..72].fill(0);
    reseal_durable_frame(&mut zero_generation);
    assert_catalog_damage(
        &zero_generation,
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        field_range(scope, 28, 44),
        Some(PhysicalFormatField::RootGeneration),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let other_format = PhysicalRecordFormatDeclaration::builder()
        .page_size(PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    let mut payload_format = bootstrap_bytes(store);
    payload_format[72..82].copy_from_slice(&other_format.canonical_identity_bytes());
    reseal_durable_frame(&mut payload_format);
    assert_catalog_damage(
        &payload_format,
        scope,
        PhysicalDamageCause::FormatMismatch,
        field_range(scope, 72, 10),
        Some(PhysicalFormatField::FormatDeclaration),
        PhysicalBlastRadius::CompleteArtifact,
    );
}

fn assert_root_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = root_rejection(bytes, scope);
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::RootRoutingBlock,
        bytes.len() as u64,
        cause,
    );
}

fn assert_segment_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = segment_rejection(bytes, scope);
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::SegmentMembership,
        bytes.len() as u64,
        cause,
    );
}

fn assert_catalog_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = match validate_bootstrap_catalog(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    ) {
        (BootstrapCatalogIntegrityValidation::Rejected(rejection), counters) => {
            (rejection, counters)
        }
        (BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch), counters) => {
            (mismatch.rejection(), counters)
        }
        (BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported), counters) => {
            (unsupported.rejection(), counters)
        }
        (BootstrapCatalogIntegrityValidation::Intact(_), _) => {
            panic!("damaged bootstrap catalog validated")
        }
    };
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::BootstrapCatalog,
        bytes.len() as u64,
        cause,
    );
}
