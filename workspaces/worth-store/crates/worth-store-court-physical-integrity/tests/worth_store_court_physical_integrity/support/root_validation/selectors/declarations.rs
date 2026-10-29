pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::PhysicalPageSizeClass};
pub(crate) use {worth_store_physical_integrity::validate_current_root_selector, worth_store_physical_integrity::validate_previous_root_selector, worth_store_physical_integrity::CurrentRootSelectorIntegrityValidation, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::PreviousRootSelectorIntegrityValidation, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::support::root_validation::assert_damage, crate::support::root_validation::assert_rejected_counters, crate::support::root_validation::field_range, crate::support::root_validation::format, crate::support::root_validation::manifest_scope, crate::support::root_validation::reseal_durable_frame, crate::support::root_validation::selector_bytes, crate::support::root_validation::selector_scope, crate::support::root_validation::store, crate::support::root_validation::validate_selector_rejection, crate::support::root_validation::SelectorKind, crate::support::root_validation::SELECTOR_OFFSET};

pub(crate) fn assert_selector_damage(
    kind: SelectorKind,
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = validate_selector_rejection(kind, bytes, scope);
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(counters, kind.family(), bytes.len() as u64, cause);
}

pub(crate) fn assert_unsupported(
    kind: SelectorKind,
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (rejection, counters) = validate_selector_rejection(kind, bytes, scope);
    let worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(unsupported) = rejection else {
        panic!("expected unsupported selector version, got {rejection:?}");
    };
    assert_eq!(unsupported.scope(), scope);
    assert_eq!(unsupported.axis(), axis);
    assert_eq!(unsupported.observed(), observed);
    assert_eq!(counters.family(), kind.family());
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), bytes.len() as u64);
    assert_eq!(counters.intact_frames(), 0);
    assert_eq!(counters.rejected_frames(), 1);
    assert_eq!(
        counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Unsupported),
        1
    );
}

pub(crate) fn assert_intact_counters(
    counters: worth_store_physical_integrity::PhysicalIntegrityObservationCounters,
    family: PhysicalIntegrityArtifactFamily,
    byte_count: u64,
) {
    assert_eq!(counters.family(), family);
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.inspected_bytes(), byte_count);
    assert_eq!(counters.intact_frames(), 1);
    assert_eq!(counters.rejected_frames(), 0);
}