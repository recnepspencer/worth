//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::advancement::inline_type_budget::collect(census);
    super::branches::inline_type_budget::collect(census);
}
