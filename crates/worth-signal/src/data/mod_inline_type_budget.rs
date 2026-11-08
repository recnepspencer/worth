//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::conditional_execution::inline_type_budget::collect(census);
    super::error::inline_type_budget::collect(census);
    super::graph::inline_type_budget::collect(census);
    super::retained_storage::inline_type_budget::collect(census);
}
