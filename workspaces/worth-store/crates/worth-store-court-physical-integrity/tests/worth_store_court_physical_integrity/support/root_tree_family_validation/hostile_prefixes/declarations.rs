pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::{
    PhysicalPageSizeClass, PhysicalRecordFormatDeclaration,
};
pub(crate) use worth_store_physical_integrity::{
    validate_bootstrap_catalog, BootstrapCatalogIntegrityValidation, PhysicalArtifactScope,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    UntrustedPhysicalArtifact,
};

pub(crate) use crate::support::root_tree_family_validation::{
    assert_damage, assert_rejected_counters, assert_root_mutation_contract,
    assert_segment_mutation_contract, bootstrap_bytes, bootstrap_scope, field_range, format,
    reseal_durable_frame, root_leaf, root_rejection, root_scope, segment_leaf, segment_rejection,
    segment_scope, store, BOOTSTRAP_OFFSET, ROOT_BLOCK_OFFSET, SEGMENT_BLOCK_OFFSET,
};

pub(crate) fn assert_root_damage(
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
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootRoutingBlock,
        bytes.len() as u64,
        cause,
    );
}

pub(crate) fn assert_segment_damage(
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
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::SegmentMembership,
        bytes.len() as u64,
        cause,
    );
}

pub(crate) fn assert_catalog_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = match validate_bootstrap_catalog(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    ) {
        (
            worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::Rejected(
                rejection,
            ),
            counters,
        ) => (rejection, counters),
        (
            worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::ScopeMismatch(
                mismatch,
            ),
            counters,
        ) => (mismatch.rejection(), counters),
        (
            worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::UnsupportedFormat(
                unsupported,
            ),
            counters,
        ) => (unsupported.rejection(), counters),
        (worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::Intact(_), _) => {
            panic!("damaged bootstrap catalog validated")
        }
    };
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::BootstrapCatalog,
        bytes.len() as u64,
        cause,
    );
}
