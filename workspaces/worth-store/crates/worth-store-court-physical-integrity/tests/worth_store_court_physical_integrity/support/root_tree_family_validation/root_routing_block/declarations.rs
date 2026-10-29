pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::maximum_current_root_entries, worth_store_physical_format::ManifestBlockReference, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalRootRoutingBlock};
pub(crate) use {worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityValidationDigest, worth_store_physical_integrity::PhysicalIntegrityValidationMechanism, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::literal_vectors::root_tree_family_validation::ROOT_ROUTING_LEAF_COMPLETE_CRC32C, crate::literal_vectors::root_tree_family_validation::ROOT_ROUTING_LEAF_LITERAL};
pub(crate) use {crate::support::root_tree_family_validation::assert_damage, crate::support::root_tree_family_validation::assert_rejected_counters, crate::support::root_tree_family_validation::assert_root_checksum_mutation_contract, crate::support::root_tree_family_validation::assert_root_mutation_contract, crate::support::root_tree_family_validation::field_range, crate::support::root_tree_family_validation::format, crate::support::root_tree_family_validation::independent_crc32c, crate::support::root_tree_family_validation::reseal_durable_frame, crate::support::root_tree_family_validation::root_branch, crate::support::root_tree_family_validation::root_leaf, crate::support::root_tree_family_validation::root_rejection as validate_rejection, crate::support::root_tree_family_validation::root_scope, crate::support::root_tree_family_validation::root_scope_with_reference, crate::support::root_tree_family_validation::store, crate::support::root_tree_family_validation::validate_root_intact as validate_intact, crate::support::root_tree_family_validation::ROOT_BLOCK_OFFSET};

pub(crate) fn root_leaf_with(tree: u64, generation: u64, block: u64) -> PhysicalRootRoutingBlock {
    let baseline = root_leaf();
    worth_store_physical_format::PhysicalRootRoutingBlock::leaf(
        tree,
        generation,
        block,
        baseline.entries().unwrap().to_vec(),
        8,
    )
    .unwrap()
}

pub(crate) fn assert_root_damage(
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
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootRoutingBlock,
        bytes.len() as u64,
        cause,
    );
}