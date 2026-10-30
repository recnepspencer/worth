pub(crate) use worth_store_physical_format::{encode_inline_page, PhysicalPageSizeClass};
pub(crate) use worth_store_physical_integrity::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
    PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass, PhysicalIntegrityVersionAxis,
};

pub(crate) use crate::support::page_validation::{
    assert_damage, clean_page, field_range, format, page, page_scope, reseal, store,
    validate_rejection, PAGE_OFFSET, PAGE_SIZES,
};

pub(crate) fn assert_unsupported(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (rejection, counters) = validate_rejection(bytes, scope);
    let worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(unsupported) =
        rejection
    else {
        panic!("expected unsupported page version, got {rejection:?}");
    };
    assert_eq!(unsupported.scope(), scope);
    assert_eq!(unsupported.axis(), axis);
    assert_eq!(unsupported.observed(), observed);
    assert_eq!(
        counters.rejected_for(
            worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Unsupported
        ),
        1
    );
}
