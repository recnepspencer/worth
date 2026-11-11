//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::home_retirement::inline_type_budget::collect(census);
    super::execution_report::inline_type_budget::collect(census);
}
