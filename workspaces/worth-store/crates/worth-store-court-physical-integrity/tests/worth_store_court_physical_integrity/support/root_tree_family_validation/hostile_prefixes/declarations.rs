pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::PhysicalPageSizeClass, worth_store_physical_format::PhysicalRecordFormatDeclaration};
pub(crate) use {worth_store_physical_integrity::validate_bootstrap_catalog, worth_store_physical_integrity::BootstrapCatalogIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::root_tree_family_validation::assert_damage, crate::support::root_tree_family_validation::assert_rejected_counters, crate::support::root_tree_family_validation::assert_root_mutation_contract, crate::support::root_tree_family_validation::assert_segment_mutation_contract, crate::support::root_tree_family_validation::bootstrap_bytes, crate::support::root_tree_family_validation::bootstrap_scope, crate::support::root_tree_family_validation::field_range, crate::support::root_tree_family_validation::format, crate::support::root_tree_family_validation::reseal_durable_frame, crate::support::root_tree_family_validation::root_leaf, crate::support::root_tree_family_validation::root_rejection, crate::support::root_tree_family_validation::root_scope, crate::support::root_tree_family_validation::segment_leaf, crate::support::root_tree_family_validation::segment_rejection, crate::support::root_tree_family_validation::segment_scope, crate::support::root_tree_family_validation::store, crate::support::root_tree_family_validation::BOOTSTRAP_OFFSET, crate::support::root_tree_family_validation::ROOT_BLOCK_OFFSET, crate::support::root_tree_family_validation::SEGMENT_BLOCK_OFFSET};

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
        (worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::Rejected(rejection), counters) => {
            (rejection, counters)
        }
        (worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::ScopeMismatch(mismatch), counters) => {
            (mismatch.rejection(), counters)
        }
        (worth_store_physical_integrity::BootstrapCatalogIntegrityValidation::UnsupportedFormat(unsupported), counters) => {
            (unsupported.rejection(), counters)
        }
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