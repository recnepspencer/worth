//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::computation::inline_type_budget::collect(census);
    super::producer::inline_type_budget::collect(census);
    super::request_execution::inline_type_budget::collect(census);
}
