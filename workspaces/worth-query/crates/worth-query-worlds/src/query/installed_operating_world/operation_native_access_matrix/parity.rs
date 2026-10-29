pub use worth_foundational::facade::{AbsenceLaw, AspectValue, StructAspectValue};
pub use worth_query::facade::domain;

pub use super::fixture::{bind, matrix_workspace_with_values};
pub use super::samples::{
    defaulted_field, matrix_aspect_key, matrix_value, optional_field, sample_field, scalar_samples,
};

pub fn assert_refinement_context(
    denial: &worth_query::facade::foundation::ConsumedNativeRefinementDenial,
    key: &domain::WorthQueryNativeAccessKey,
    fact: &worth_query::facade::foundation::ConsumedFieldValueFact,
) {
    assert_eq!(denial.field_path(), key.field_path());
    assert_eq!(denial.contract_key(), Some(key.contract_key()));
    assert_eq!(denial.contract_identity(), Some(key.contract_identity()));
    assert_eq!(denial.contract_revision(), Some(key.contract_revision()));
    assert_eq!(denial.source_family(), fact.source_family());
    assert_eq!(denial.source_identity(), fact.source_identity());
    assert_eq!(denial.source_row_identity(), fact.source_row_identity());
    assert_eq!(denial.projection_authority(), fact.projection_authority());
}

pub enum ExpectedNativeValue {
    Struct(StructAspectValue),
    Scalar(AspectValue),
    Absent(AbsenceLaw),
}

pub fn assert_constant_resolution(counters: domain::WorthQueryNativeKeyResolutionCounters) {
    assert_eq!(counters.declaration_checks, 1);
    assert_eq!(counters.indexed_slot_lookups, 2);
    assert_eq!(counters.path_matches, 0);
    assert_eq!(counters.key_scans, 0);
    assert_eq!(counters.path_parses, 0);
}

pub fn assert_constant_access(counters: domain::WorthQueryNativeAccessCounters) {
    assert_eq!(counters.authority_checks, 5);
    assert_eq!(counters.indexed_accesses, 1);
    assert_eq!(counters.refinement_checks, 1);
    assert_eq!(counters.fact_scans, 0);
    assert_eq!(counters.row_scans, 0);
    assert_eq!(counters.path_parses, 0);
    assert_eq!(counters.view_registry_inspections, 0);
    assert_eq!(counters.domain_registry_inspections, 0);
}
