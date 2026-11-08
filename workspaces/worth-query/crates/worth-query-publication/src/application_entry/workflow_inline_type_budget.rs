//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::definition::inline_type_budget::collect(census);
    super::instance::inline_type_budget::collect(census);
    super::operation::inline_type_budget::collect(census);
    super::ordinary::inline_type_budget::collect(census);
    super::progress::inline_type_budget::collect(census);
}
