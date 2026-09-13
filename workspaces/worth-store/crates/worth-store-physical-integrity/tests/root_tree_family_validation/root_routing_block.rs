use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    maximum_current_root_entries, ManifestBlockReference, PersistedRecordIdentity,
    PhysicalRootRoutingBlock,
};
use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejection, PhysicalIntegrityValidationDigest,
    PhysicalIntegrityValidationMechanism, PhysicalIntegrityVersionAxis, UntrustedPhysicalArtifact,
};

use super::literal_vectors::{ROOT_ROUTING_LEAF_COMPLETE_CRC32C, ROOT_ROUTING_LEAF_LITERAL};
use super::support::{
    assert_damage, assert_rejected_counters, assert_root_checksum_mutation_contract,
    assert_root_mutation_contract, field_range, format, independent_crc32c, reseal_durable_frame,
    root_branch, root_leaf, root_rejection as validate_rejection, root_scope,
    root_scope_with_reference, store, validate_root_intact as validate_intact, ROOT_BLOCK_OFFSET,
};

#[test]
fn root_routing_literal_vector_has_independent_complete_child_crc_and_validates() {
    assert_eq!(root_leaf().encode(format()), ROOT_ROUTING_LEAF_LITERAL);
    assert_eq!(
        independent_crc32c(&[&ROOT_ROUTING_LEAF_LITERAL]),
        ROOT_ROUTING_LEAF_COMPLETE_CRC32C
    );
    assert_eq!(
        u32::from_le_bytes(ROOT_ROUTING_LEAF_LITERAL[44..48].try_into().unwrap()),
        independent_crc32c(&[
            &ROOT_ROUTING_LEAF_LITERAL[..44],
            &ROOT_ROUTING_LEAF_LITERAL[48..],
        ])
    );
    let record = PersistedRecordIdentity::new([0xa1; 16], 5).unwrap();
    let reference =
        ManifestBlockReference::new(11, 3, 0, ROOT_ROUTING_LEAF_COMPLETE_CRC32C, record, record)
            .unwrap();
    let scope = root_scope_with_reference(
        store(7),
        reference,
        ROOT_ROUTING_LEAF_LITERAL.len() as u64,
        ROOT_BLOCK_OFFSET,
    );
    let validated = validate_intact(
        UntrustedPhysicalArtifact::from_bounded_bytes(&ROOT_ROUTING_LEAF_LITERAL),
        scope,
    );
    assert_eq!(validated.entries().unwrap().len(), 1);
    let record = validated.into_validation_record();
    assert_eq!(
        record.byte_range_digest(),
        PhysicalIntegrityValidationDigest::crc32c(ROOT_ROUTING_LEAF_COMPLETE_CRC32C)
    );
    assert_eq!(
        record.mechanism(),
        PhysicalIntegrityValidationMechanism::Crc32cV1
    );
    assert_eq!(record.mechanism().version(), 1);
}

#[test]
fn clean_leaf_and_branch_seal_projections_records_and_recursive_child_scope() {
    let store = store(7);
    let leaf = root_leaf();
    let leaf_bytes = leaf.encode(format());
    let leaf_scope = root_scope(store, &leaf, &leaf_bytes, ROOT_BLOCK_OFFSET);
    let leaf_artifact = UntrustedPhysicalArtifact::from_bounded_bytes(&leaf_bytes);
    let leaf_validated = validate_intact(leaf_artifact, leaf_scope);
    assert_eq!(leaf_validated.scope(), leaf_scope);
    assert_eq!(leaf_validated.tree_identity(), 71);
    assert_eq!(leaf_validated.generation(), 11);
    assert_eq!(leaf_validated.block_identity(), 3);
    assert_eq!(leaf_validated.level(), 0);
    assert_eq!(leaf_validated.entries().unwrap().len(), 1);
    assert!(leaf_validated.children().is_none());
    assert!(leaf_validated.matches_input(leaf_artifact));
    let leaf_record = leaf_validated.into_validation_record();
    assert!(leaf_record.matches_scope(leaf_scope));
    let foreign_store_scope = root_scope(
        super::support::store(8),
        &leaf,
        &leaf_bytes,
        ROOT_BLOCK_OFFSET,
    );
    let foreign_store_record =
        validate_intact(leaf_artifact, foreign_store_scope).into_validation_record();
    assert_ne!(
        leaf_record.exact_scope_digest(),
        foreign_store_record.exact_scope_digest()
    );
    let shifted_scope = root_scope(store, &leaf, &leaf_bytes, ROOT_BLOCK_OFFSET + 512);
    let shifted_record = validate_intact(leaf_artifact, shifted_scope).into_validation_record();
    assert_ne!(
        leaf_record.exact_scope_digest(),
        shifted_record.exact_scope_digest()
    );

    let child_reference = leaf.reference(independent_crc32c(&[&leaf_bytes]));
    let branch = root_branch(child_reference);
    let branch_bytes = branch.encode(format());
    let branch_scope = root_scope(store, &branch, &branch_bytes, ROOT_BLOCK_OFFSET + 4_096);
    let branch_validated = validate_intact(
        UntrustedPhysicalArtifact::from_bounded_bytes(&branch_bytes),
        branch_scope,
    );
    assert_eq!(branch_validated.level(), 1);
    assert!(branch_validated.entries().is_none());
    assert_eq!(branch_validated.children(), Some(&[child_reference][..]));

    let recursive_scope = root_scope_with_reference(
        store,
        branch_validated.children().unwrap()[0],
        leaf_bytes.len() as u64,
        ROOT_BLOCK_OFFSET,
    );
    let _ = validate_intact(leaf_artifact, recursive_scope);
}

#[test]
fn internally_resealed_child_is_rejected_by_parent_complete_child_crc() {
    let store = store(7);
    let leaf = root_leaf();
    let clean = leaf.encode(format());
    let scope = root_scope(store, &leaf, &clean, ROOT_BLOCK_OFFSET);
    let mut resealed = clean;
    resealed[160..168].copy_from_slice(&24_u64.to_le_bytes());
    reseal_durable_frame(&mut resealed);

    assert_root_damage(
        &resealed,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        Some(PhysicalFormatField::Checksum),
        PhysicalBlastRadius::ReachableSubtree,
    );
}

#[test]
fn root_block_crc_framing_kind_length_and_truncation_are_bounded() {
    let store = store(7);
    let block = root_leaf();
    let clean = block.encode(format());
    let clean_scope = root_scope(store, &block, &clean, ROOT_BLOCK_OFFSET);

    let mut covered_flip = clean.clone();
    covered_flip[48] ^= 1;
    assert_root_damage(
        &covered_flip,
        clean_scope,
        PhysicalDamageCause::ChecksumMismatch,
        clean_scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_flip = clean.clone();
    checksum_flip[44] ^= 1;
    let scope = root_scope(store, &block, &checksum_flip, ROOT_BLOCK_OFFSET);
    assert_root_checksum_mutation_contract(&checksum_flip, scope);
    assert_root_damage(
        &checksum_flip,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut frame_kind = clean.clone();
    frame_kind[8] = 9;
    reseal_durable_frame(&mut frame_kind);
    let scope = root_scope(store, &block, &frame_kind, ROOT_BLOCK_OFFSET);
    assert_root_mutation_contract(&frame_kind, scope);
    assert_root_damage(
        &frame_kind,
        scope,
        PhysicalDamageCause::FamilyMismatch,
        field_range(scope, 8, 1),
        Some(PhysicalFormatField::ArtifactFamily),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut node_kind = clean.clone();
    node_kind[68] = 2;
    reseal_durable_frame(&mut node_kind);
    let scope = root_scope(store, &block, &node_kind, ROOT_BLOCK_OFFSET);
    assert_root_mutation_contract(&node_kind, scope);
    assert_root_damage(
        &node_kind,
        scope,
        PhysicalDamageCause::RecordKindMismatch,
        field_range(scope, 68, 1),
        Some(PhysicalFormatField::RoutingNodeKind),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut length_lie = clean.clone();
    length_lie[24..28].copy_from_slice(&0_u32.to_le_bytes());
    reseal_durable_frame(&mut length_lie);
    let scope = root_scope(store, &block, &length_lie, ROOT_BLOCK_OFFSET);
    assert_root_mutation_contract(&length_lie, scope);
    assert_root_damage(
        &length_lie,
        scope,
        PhysicalDamageCause::FramingLengthMismatch,
        field_range(scope, 20, 8),
        Some(PhysicalFormatField::EncodedLength),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut over_bound = clean.clone();
    let count = maximum_current_root_entries(format()) + 1;
    over_bound[66..68].copy_from_slice(&count.to_le_bytes());
    reseal_durable_frame(&mut over_bound);
    let scope = root_scope(store, &block, &over_bound, ROOT_BLOCK_OFFSET);
    assert_root_mutation_contract(&over_bound, scope);
    assert_root_damage(
        &over_bound,
        scope,
        PhysicalDamageCause::FramingLengthMismatch,
        field_range(scope, 66, 2),
        Some(PhysicalFormatField::EncodedLength),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let truncated = &clean[..clean.len() - 3];
    assert_root_damage(
        truncated,
        clean_scope,
        PhysicalDamageCause::Truncated,
        PhysicalByteRange::new(ROOT_BLOCK_OFFSET + truncated.len() as u64, 3).unwrap(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );
}

#[test]
fn root_tree_generation_block_reference_range_and_pointer_substitution_localize() {
    let store = store(7);
    let baseline = root_leaf();
    let clean = baseline.encode(format());
    for (substitute, cause, offset, field) in [
        (
            root_leaf_with(72, 11, 3),
            PhysicalDamageCause::ArtifactIdentityMismatch,
            48,
            PhysicalFormatField::TreeIdentity,
        ),
        (
            root_leaf_with(71, 11, 4),
            PhysicalDamageCause::ArtifactIdentityMismatch,
            28,
            PhysicalFormatField::BlockIdentity,
        ),
        (
            root_leaf_with(71, 12, 3),
            PhysicalDamageCause::PhysicalGenerationMismatch,
            72,
            PhysicalFormatField::PhysicalGeneration,
        ),
    ] {
        let bytes = substitute.encode(format());
        let scope = root_scope(store, &baseline, &bytes, ROOT_BLOCK_OFFSET);
        assert_root_mutation_contract(&bytes, scope);
        assert_root_damage(
            &bytes,
            scope,
            cause,
            field_range(scope, offset, 8),
            Some(field),
            PhysicalBlastRadius::ReachableSubtree,
        );
    }

    let reference = baseline.reference(independent_crc32c(&[&clean]));
    let wrong_first = PersistedRecordIdentity::new([0xa0; 16], 5).unwrap();
    let wrong_range = ManifestBlockReference::new(
        reference.generation(),
        reference.block(),
        reference.level(),
        reference.checksum(),
        wrong_first,
        reference.last(),
    )
    .unwrap();
    let range_scope =
        root_scope_with_reference(store, wrong_range, clean.len() as u64, ROOT_BLOCK_OFFSET);
    assert_root_damage(
        &clean,
        range_scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        field_range(range_scope, 88, 24),
        Some(PhysicalFormatField::RecordIdentity),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let child = baseline.reference(independent_crc32c(&[&clean]));
    let branch = root_branch(child);
    let mut pointer = branch.encode(format());
    pointer[88..96].copy_from_slice(&13_u64.to_le_bytes());
    reseal_durable_frame(&mut pointer);
    let branch_scope = root_scope(store, &branch, &pointer, ROOT_BLOCK_OFFSET + 4_096);
    assert_root_mutation_contract(&pointer, branch_scope);
    assert_root_damage(
        &pointer,
        branch_scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        field_range(branch_scope, 88, 8),
        Some(PhysicalFormatField::ChildReference),
        PhysicalBlastRadius::ReachableSubtree,
    );
}

#[test]
fn root_block_version_axes_remain_unsupported_not_corrupt() {
    let store = store(7);
    let block = root_leaf();
    let clean = block.encode(format());
    for (offset, width, axis, observed) in [
        (9, 1, PhysicalIntegrityVersionAxis::EnvelopeSchema, 3),
        (10, 2, PhysicalIntegrityVersionAxis::PhysicalFormat, 2),
    ] {
        let mut bytes = clean.clone();
        if width == 1 {
            bytes[offset] = observed as u8;
        } else {
            bytes[offset..offset + width].copy_from_slice(&(observed as u16).to_le_bytes());
        }
        reseal_durable_frame(&mut bytes);
        let scope = root_scope(store, &block, &bytes, ROOT_BLOCK_OFFSET);
        assert_root_mutation_contract(&bytes, scope);
        match validate_rejection(&bytes, scope).0 {
            PhysicalIntegrityRejection::Unsupported(posture) => {
                assert_eq!(posture.axis(), axis);
                assert_eq!(posture.observed(), observed);
            }
            other => panic!("unsupported root-routing version collapsed: {other:?}"),
        }
    }
}

fn root_leaf_with(tree: u64, generation: u64, block: u64) -> PhysicalRootRoutingBlock {
    let baseline = root_leaf();
    PhysicalRootRoutingBlock::leaf(
        tree,
        generation,
        block,
        baseline.entries().unwrap().to_vec(),
        8,
    )
    .unwrap()
}

fn assert_root_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = validate_rejection(bytes, scope);
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::RootRoutingBlock,
        bytes.len() as u64,
        cause,
    );
}
