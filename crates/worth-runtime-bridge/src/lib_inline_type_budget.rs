//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::conditional_execution::inline_type_budget::collect(census);
    super::correspondence::inline_type_budget::collect(census);
    super::error::inline_type_budget::collect(census);
    super::execution_basis::inline_type_budget::collect(census);
    super::home_retirement::inline_type_budget::collect(census);
    super::relational_source::inline_type_budget::collect(census);
    super::source::inline_type_budget::collect(census);
}
