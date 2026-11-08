//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::delivery::inline_type_budget::collect(census);
    super::installation::inline_type_budget::collect(census);
}
