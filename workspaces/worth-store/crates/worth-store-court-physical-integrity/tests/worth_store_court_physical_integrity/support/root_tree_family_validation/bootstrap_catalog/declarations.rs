pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::PhysicalPageSizeClass, worth_store_physical_format::PhysicalRecordFormatDeclaration};
pub(crate) use {worth_store_physical_integrity::validate_bootstrap_catalog, worth_store_physical_integrity::BootstrapCatalogIntegrityValidation, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityValidationDigest, worth_store_physical_integrity::PhysicalIntegrityValidationMechanism, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::literal_vectors::root_tree_family_validation::BOOTSTRAP_CATALOG_COMPLETE_CRC32C, crate::literal_vectors::root_tree_family_validation::BOOTSTRAP_CATALOG_LITERAL};
pub(crate) use {crate::support::root_tree_family_validation::assert_damage, crate::support::root_tree_family_validation::assert_rejected_counters, crate::support::root_tree_family_validation::bootstrap_bytes, crate::support::root_tree_family_validation::bootstrap_scope, crate::support::root_tree_family_validation::field_range, crate::support::root_tree_family_validation::format, crate::support::root_tree_family_validation::independent_crc32c, crate::support::root_tree_family_validation::reseal_durable_frame, crate::support::root_tree_family_validation::store, crate::support::root_tree_family_validation::BOOTSTRAP_OFFSET};

pub(crate) fn assert_catalog_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = validate_catalog_rejection(bytes, scope);
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::BootstrapCatalog,
        bytes.len() as u64,
        cause,
    );
}

pub(crate) fn validate_catalog_rejection(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
) -> (
    PhysicalIntegrityRejection,
    worth_store_physical_integrity::PhysicalIntegrityObservationCounters,
) {
    match validate_bootstrap_catalog(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope) {
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
    }
}