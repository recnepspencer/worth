#[allow(unused_imports)] use crate::root_tree_family_validation::hostile_prefixes::*;
use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
use {worth_store_physical_format::PhysicalPageSizeClass, worth_store_physical_format::PhysicalRecordFormatDeclaration};
use {worth_store_physical_integrity::validate_bootstrap_catalog, worth_store_physical_integrity::BootstrapCatalogIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::support::root_tree_family_validation::assert_damage, crate::support::root_tree_family_validation::assert_rejected_counters, crate::support::root_tree_family_validation::assert_root_mutation_contract, crate::support::root_tree_family_validation::assert_segment_mutation_contract, crate::support::root_tree_family_validation::bootstrap_bytes, crate::support::root_tree_family_validation::bootstrap_scope, crate::support::root_tree_family_validation::field_range, crate::support::root_tree_family_validation::format, crate::support::root_tree_family_validation::reseal_durable_frame, crate::support::root_tree_family_validation::root_leaf, crate::support::root_tree_family_validation::root_rejection, crate::support::root_tree_family_validation::root_scope, crate::support::root_tree_family_validation::segment_leaf, crate::support::root_tree_family_validation::segment_rejection, crate::support::root_tree_family_validation::segment_scope, crate::support::root_tree_family_validation::store, crate::support::root_tree_family_validation::BOOTSTRAP_OFFSET, crate::support::root_tree_family_validation::ROOT_BLOCK_OFFSET, crate::support::root_tree_family_validation::SEGMENT_BLOCK_OFFSET};

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
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        field_range(short_scope, 48, 16),
        Some(worth_store_physical_integrity::PhysicalFormatField::Reserved),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
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
            worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
            field_range(scope, offset as u64, length),
            Some(worth_store_physical_integrity::PhysicalFormatField::Reserved),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
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
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        field_range(short_scope, 48, 16),
        Some(worth_store_physical_integrity::PhysicalFormatField::Reserved),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
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
            worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
            field_range(scope, offset as u64, length),
            Some(worth_store_physical_integrity::PhysicalFormatField::Reserved),
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
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
        worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
        field_range(root_scope, 28, 36),
        Some(worth_store_physical_integrity::PhysicalFormatField::BlockIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
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
        worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
        field_range(segment_scope, 28, 36),
        Some(worth_store_physical_integrity::PhysicalFormatField::BlockIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
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
        worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
        field_range(scope, 28, 44),
        Some(worth_store_physical_integrity::PhysicalFormatField::RootGeneration),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let other_format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .page_size(worth_store_physical_format::PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    let mut payload_format = bootstrap_bytes(store);
    payload_format[72..82].copy_from_slice(&other_format.canonical_identity_bytes());
    reseal_durable_frame(&mut payload_format);
    assert_catalog_damage(
        &payload_format,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FormatMismatch,
        field_range(scope, 72, 10),
        Some(worth_store_physical_integrity::PhysicalFormatField::FormatDeclaration),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}
