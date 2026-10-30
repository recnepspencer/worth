pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::{
    maximum_current_root_entries, ManifestBlockReference, PersistedRecordIdentity,
    PhysicalRootRoutingBlock,
};
pub(crate) use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejection, PhysicalIntegrityValidationDigest,
    PhysicalIntegrityValidationMechanism, PhysicalIntegrityVersionAxis, UntrustedPhysicalArtifact,
};

pub(crate) use crate::literal_vectors::root_tree_family_validation::{
    ROOT_ROUTING_LEAF_COMPLETE_CRC32C, ROOT_ROUTING_LEAF_LITERAL,
};
pub(crate) use crate::support::root_tree_family_validation::{
    assert_damage, assert_rejected_counters, assert_root_checksum_mutation_contract,
    assert_root_mutation_contract, field_range, format, independent_crc32c, reseal_durable_frame,
    root_branch, root_leaf, root_rejection as validate_rejection, root_scope,
    root_scope_with_reference, store, validate_root_intact as validate_intact, ROOT_BLOCK_OFFSET,
};

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
