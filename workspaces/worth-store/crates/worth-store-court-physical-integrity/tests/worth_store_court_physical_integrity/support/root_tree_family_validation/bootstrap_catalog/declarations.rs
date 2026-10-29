pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::{
    PhysicalPageSizeClass, PhysicalRecordFormatDeclaration,
};
pub(crate) use worth_store_physical_integrity::{
    validate_bootstrap_catalog, BootstrapCatalogIntegrityValidation, PhysicalArtifactScope,
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    PhysicalIntegrityRejection, PhysicalIntegrityValidationDigest,
    PhysicalIntegrityValidationMechanism, PhysicalIntegrityVersionAxis, UntrustedPhysicalArtifact,
};

pub(crate) use crate::literal_vectors::root_tree_family_validation::{
    BOOTSTRAP_CATALOG_COMPLETE_CRC32C, BOOTSTRAP_CATALOG_LITERAL,
};
pub(crate) use crate::support::root_tree_family_validation::{
    assert_damage, assert_rejected_counters, bootstrap_bytes, bootstrap_scope, field_range, format,
    independent_crc32c, reseal_durable_frame, store, BOOTSTRAP_OFFSET,
};

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
    match validate_bootstrap_catalog(
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
    }
}
