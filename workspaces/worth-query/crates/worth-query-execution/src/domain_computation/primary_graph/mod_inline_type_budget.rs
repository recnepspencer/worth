//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::application_attempt::inline_type_budget::collect(census);
    super::application_contribution::inline_type_budget::collect(census);
    super::application_entry::inline_type_budget::collect(census);
    super::application_installation::inline_type_budget::collect(census);
    super::application_output_demand::inline_type_budget::collect(census);
    super::application_program::inline_type_budget::collect(census);
    super::application_query::inline_type_budget::collect(census);
    super::application_runtime::inline_type_budget::collect(census);
    super::granular_invalidation::inline_type_budget::collect(census);
    super::product_operation::inline_type_budget::collect(census);
    super::provider::inline_type_budget::collect(census);
    super::causal_inspection::inline_type_budget::collect(census);
}
