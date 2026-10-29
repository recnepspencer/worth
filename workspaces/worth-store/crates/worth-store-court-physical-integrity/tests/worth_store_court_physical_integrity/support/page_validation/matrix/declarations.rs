pub(crate) use {worth_store_physical_format::encode_inline_page, worth_store_physical_format::PhysicalPageSizeClass};
pub(crate) use {worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityVersionAxis};

pub(crate) use {crate::support::page_validation::assert_damage, crate::support::page_validation::clean_page, crate::support::page_validation::field_range, crate::support::page_validation::format, crate::support::page_validation::page, crate::support::page_validation::page_scope, crate::support::page_validation::reseal, crate::support::page_validation::store, crate::support::page_validation::validate_rejection, crate::support::page_validation::PAGE_OFFSET, crate::support::page_validation::PAGE_SIZES};

pub(crate) fn assert_unsupported(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (rejection, counters) = validate_rejection(bytes, scope);
    let worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(unsupported) = rejection else {
        panic!("expected unsupported page version, got {rejection:?}");
    };
    assert_eq!(unsupported.scope(), scope);
    assert_eq!(unsupported.axis(), axis);
    assert_eq!(unsupported.observed(), observed);
    assert_eq!(
        counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Unsupported),
        1
    );
}