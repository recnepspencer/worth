#[allow(unused_imports)] use crate::root_tree_family_validation::segment_membership_block::*;
use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
use {worth_store_physical_format::maximum_segment_manifest_pages, worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalSegmentId, worth_store_physical_format::PhysicalSegmentMembershipBlock, worth_store_physical_format::SegmentManifestBlockReference, worth_store_physical_format::SegmentPageKey};
use {worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityValidationDigest, worth_store_physical_integrity::PhysicalIntegrityValidationMechanism, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::literal_vectors::root_tree_family_validation::SEGMENT_MEMBERSHIP_LEAF_COMPLETE_CRC32C, crate::literal_vectors::root_tree_family_validation::SEGMENT_MEMBERSHIP_LEAF_LITERAL};
use {crate::support::root_tree_family_validation::assert_damage, crate::support::root_tree_family_validation::assert_rejected_counters, crate::support::root_tree_family_validation::assert_segment_checksum_mutation_contract, crate::support::root_tree_family_validation::assert_segment_mutation_contract, crate::support::root_tree_family_validation::field_range, crate::support::root_tree_family_validation::format, crate::support::root_tree_family_validation::independent_crc32c, crate::support::root_tree_family_validation::reseal_durable_frame, crate::support::root_tree_family_validation::segment_branch, crate::support::root_tree_family_validation::segment_leaf, crate::support::root_tree_family_validation::segment_rejection as validate_rejection, crate::support::root_tree_family_validation::segment_scope, crate::support::root_tree_family_validation::segment_scope_with_reference, crate::support::root_tree_family_validation::store, crate::support::root_tree_family_validation::validate_segment_intact as validate_intact, crate::support::root_tree_family_validation::SEGMENT_BLOCK_OFFSET};

#[test]
fn membership_literal_vector_has_independent_complete_child_crc_and_validates() {
    assert_eq!(
        segment_leaf().encode(format()),
        SEGMENT_MEMBERSHIP_LEAF_LITERAL
    );
    assert_eq!(
        independent_crc32c(&[&SEGMENT_MEMBERSHIP_LEAF_LITERAL]),
        SEGMENT_MEMBERSHIP_LEAF_COMPLETE_CRC32C
    );
    assert_eq!(
        u32::from_le_bytes(SEGMENT_MEMBERSHIP_LEAF_LITERAL[44..48].try_into().unwrap()),
        independent_crc32c(&[
            &SEGMENT_MEMBERSHIP_LEAF_LITERAL[..44],
            &SEGMENT_MEMBERSHIP_LEAF_LITERAL[48..],
        ])
    );
    let key = worth_store_physical_format::SegmentPageKey::new(
        worth_store_physical_format::PhysicalSegmentId::from_raw(13).unwrap(),
        worth_store_physical_format::PhysicalPageId::from_raw(17).unwrap(),
    );
    let reference = worth_store_physical_format::SegmentManifestBlockReference::new(
        11,
        5,
        0,
        SEGMENT_MEMBERSHIP_LEAF_COMPLETE_CRC32C,
        key,
        key,
    )
    .unwrap();
    let scope = segment_scope_with_reference(
        store(7),
        reference,
        SEGMENT_MEMBERSHIP_LEAF_LITERAL.len() as u64,
        SEGMENT_BLOCK_OFFSET,
    );
    let validated = validate_intact(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&SEGMENT_MEMBERSHIP_LEAF_LITERAL),
        scope,
    );
    assert_eq!(validated.entries().unwrap().len(), 1);
    let record = validated.into_validation_record();
    assert_eq!(
        record.byte_range_digest(),
        worth_store_physical_integrity::PhysicalIntegrityValidationDigest::crc32c(SEGMENT_MEMBERSHIP_LEAF_COMPLETE_CRC32C)
    );
    assert_eq!(
        record.mechanism(),
        worth_store_physical_integrity::PhysicalIntegrityValidationMechanism::Crc32cV1
    );
    assert_eq!(record.mechanism().version(), 1);
}

#[test]
fn clean_leaf_and_branch_seal_membership_projections_and_recursive_child_scope() {
    let store = store(7);
    let leaf = segment_leaf();
    let leaf_bytes = leaf.encode(format());
    let leaf_scope = segment_scope(store, &leaf, &leaf_bytes, SEGMENT_BLOCK_OFFSET);
    let leaf_artifact = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&leaf_bytes);
    let leaf_validated = validate_intact(leaf_artifact, leaf_scope);
    assert_eq!(leaf_validated.scope(), leaf_scope);
    assert_eq!(leaf_validated.tree_identity(), 73);
    assert_eq!(leaf_validated.generation(), 11);
    assert_eq!(leaf_validated.block_identity(), 5);
    assert_eq!(leaf_validated.level(), 0);
    assert_eq!(leaf_validated.entries().unwrap().len(), 1);
    assert!(leaf_validated.children().is_none());
    assert!(leaf_validated.matches_input(leaf_artifact));
    let leaf_record = leaf_validated.into_validation_record();
    assert!(leaf_record.matches_scope(leaf_scope));
    let foreign_store_scope = segment_scope(
        crate::support::root_tree_family_validation::store(8),
        &leaf,
        &leaf_bytes,
        SEGMENT_BLOCK_OFFSET,
    );
    let foreign_store_record =
        validate_intact(leaf_artifact, foreign_store_scope).into_validation_record();
    assert_ne!(
        leaf_record.exact_scope_digest(),
        foreign_store_record.exact_scope_digest()
    );
    let shifted_scope = segment_scope(store, &leaf, &leaf_bytes, SEGMENT_BLOCK_OFFSET + 512);
    let shifted_record = validate_intact(leaf_artifact, shifted_scope).into_validation_record();
    assert_ne!(
        leaf_record.exact_scope_digest(),
        shifted_record.exact_scope_digest()
    );

    let child_reference = leaf.reference(independent_crc32c(&[&leaf_bytes]));
    let branch = segment_branch(child_reference);
    let branch_bytes = branch.encode(format());
    let branch_scope = segment_scope(store, &branch, &branch_bytes, SEGMENT_BLOCK_OFFSET + 4_096);
    let branch_validated = validate_intact(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&branch_bytes),
        branch_scope,
    );
    assert_eq!(branch_validated.level(), 1);
    assert!(branch_validated.entries().is_none());
    assert_eq!(branch_validated.children(), Some(&[child_reference][..]));

    let recursive_scope = segment_scope_with_reference(
        store,
        branch_validated.children().unwrap()[0],
        leaf_bytes.len() as u64,
        SEGMENT_BLOCK_OFFSET,
    );
    let _ = validate_intact(leaf_artifact, recursive_scope);
}

#[test]
fn internally_resealed_membership_child_fails_parent_complete_child_crc() {
    let store = store(7);
    let leaf = segment_leaf();
    let clean = leaf.encode(format());
    let scope = segment_scope(store, &leaf, &clean, SEGMENT_BLOCK_OFFSET);
    let mut resealed = clean;
    resealed[120..124].copy_from_slice(&3_u32.to_le_bytes());
    reseal_durable_frame(&mut resealed);

    assert_segment_damage(
        &resealed,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        Some(worth_store_physical_integrity::PhysicalFormatField::Checksum),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );
}

#[test]
fn membership_crc_framing_kinds_length_and_truncation_are_bounded() {
    let store = store(7);
    let block = segment_leaf();
    let clean = block.encode(format());
    let clean_scope = segment_scope(store, &block, &clean, SEGMENT_BLOCK_OFFSET);

    let mut covered_flip = clean.clone();
    covered_flip[48] ^= 1;
    assert_segment_damage(
        &covered_flip,
        clean_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        clean_scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_flip = clean.clone();
    checksum_flip[44] ^= 1;
    let scope = segment_scope(store, &block, &checksum_flip, SEGMENT_BLOCK_OFFSET);
    assert_segment_checksum_mutation_contract(&checksum_flip, scope);
    assert_segment_damage(
        &checksum_flip,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut frame_kind = clean.clone();
    frame_kind[8] = 8;
    reseal_durable_frame(&mut frame_kind);
    let scope = segment_scope(store, &block, &frame_kind, SEGMENT_BLOCK_OFFSET);
    assert_segment_mutation_contract(&frame_kind, scope);
    assert_segment_damage(
        &frame_kind,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
        field_range(scope, 8, 1),
        Some(worth_store_physical_integrity::PhysicalFormatField::ArtifactFamily),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut node_kind = clean.clone();
    node_kind[68] = 2;
    reseal_durable_frame(&mut node_kind);
    let scope = segment_scope(store, &block, &node_kind, SEGMENT_BLOCK_OFFSET);
    assert_segment_mutation_contract(&node_kind, scope);
    assert_segment_damage(
        &node_kind,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::RecordKindMismatch,
        field_range(scope, 68, 1),
        Some(worth_store_physical_integrity::PhysicalFormatField::RoutingNodeKind),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut length_lie = clean.clone();
    length_lie[24..28].copy_from_slice(&0_u32.to_le_bytes());
    reseal_durable_frame(&mut length_lie);
    let scope = segment_scope(store, &block, &length_lie, SEGMENT_BLOCK_OFFSET);
    assert_segment_mutation_contract(&length_lie, scope);
    assert_segment_damage(
        &length_lie,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        field_range(scope, 20, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut over_bound = clean.clone();
    let count = u16::try_from(maximum_segment_manifest_pages(format())).unwrap() + 1;
    over_bound[66..68].copy_from_slice(&count.to_le_bytes());
    reseal_durable_frame(&mut over_bound);
    let scope = segment_scope(store, &block, &over_bound, SEGMENT_BLOCK_OFFSET);
    assert_segment_mutation_contract(&over_bound, scope);
    assert_segment_damage(
        &over_bound,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        field_range(scope, 66, 2),
        Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let truncated = &clean[..clean.len() - 5];
    assert_segment_damage(
        truncated,
        clean_scope,
        worth_store_physical_integrity::PhysicalDamageCause::Truncated,
        worth_store_physical_integrity::PhysicalByteRange::new(SEGMENT_BLOCK_OFFSET + truncated.len() as u64, 5).unwrap(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );
}

#[test]
fn membership_tree_generation_block_reference_range_and_pointer_substitution_localize() {
    let store = store(7);
    let baseline = segment_leaf();
    let clean = baseline.encode(format());
    for (substitute, cause, offset, field) in [
        (
            segment_leaf_with(74, 11, 5),
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            48,
            worth_store_physical_integrity::PhysicalFormatField::TreeIdentity,
        ),
        (
            segment_leaf_with(73, 11, 6),
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            28,
            worth_store_physical_integrity::PhysicalFormatField::BlockIdentity,
        ),
        (
            segment_leaf_with(73, 12, 5),
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            72,
            worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
        ),
    ] {
        let bytes = substitute.encode(format());
        let scope = segment_scope(store, &baseline, &bytes, SEGMENT_BLOCK_OFFSET);
        assert_segment_mutation_contract(&bytes, scope);
        assert_segment_damage(
            &bytes,
            scope,
            cause,
            field_range(scope, offset, 8),
            Some(field),
            worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
        );
    }

    let reference = baseline.reference(independent_crc32c(&[&clean]));
    let wrong_first = worth_store_physical_format::SegmentPageKey::new(
        worth_store_physical_format::PhysicalSegmentId::from_raw(12).unwrap(),
        worth_store_physical_format::PhysicalPageId::from_raw(17).unwrap(),
    );
    let wrong_range = worth_store_physical_format::SegmentManifestBlockReference::new(
        reference.generation(),
        reference.block(),
        reference.level(),
        reference.checksum(),
        wrong_first,
        reference.last(),
    )
    .unwrap();
    let range_scope =
        segment_scope_with_reference(store, wrong_range, clean.len() as u64, SEGMENT_BLOCK_OFFSET);
    assert_segment_damage(
        &clean,
        range_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        field_range(range_scope, 88, 16),
        Some(worth_store_physical_integrity::PhysicalFormatField::RecordIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let child = baseline.reference(independent_crc32c(&[&clean]));
    let branch = segment_branch(child);
    let mut pointer = branch.encode(format());
    pointer[88..96].copy_from_slice(&13_u64.to_le_bytes());
    reseal_durable_frame(&mut pointer);
    let branch_scope = segment_scope(store, &branch, &pointer, SEGMENT_BLOCK_OFFSET + 4_096);
    assert_segment_mutation_contract(&pointer, branch_scope);
    assert_segment_damage(
        &pointer,
        branch_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        field_range(branch_scope, 88, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::ChildReference),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );
}

#[test]
fn membership_version_axes_remain_unsupported_not_corrupt() {
    let store = store(7);
    let block = segment_leaf();
    let clean = block.encode(format());
    for (offset, width, axis, observed) in [
        (9, 1, worth_store_physical_integrity::PhysicalIntegrityVersionAxis::EnvelopeSchema, 3),
        (10, 2, worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat, 3),
    ] {
        let mut bytes = clean.clone();
        if width == 1 {
            bytes[offset] = observed as u8;
        } else {
            bytes[offset..offset + width].copy_from_slice(&(observed as u16).to_le_bytes());
        }
        reseal_durable_frame(&mut bytes);
        let scope = segment_scope(store, &block, &bytes, SEGMENT_BLOCK_OFFSET);
        assert_segment_mutation_contract(&bytes, scope);
        match validate_rejection(&bytes, scope).0 {
            worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(posture) => {
                assert_eq!(posture.axis(), axis);
                assert_eq!(posture.observed(), observed);
            }
            other => panic!("unsupported membership version collapsed: {other:?}"),
        }
    }
}
