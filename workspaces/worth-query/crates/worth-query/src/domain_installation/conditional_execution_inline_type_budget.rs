//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::installation::inline_type_budget::collect(census);
    super::registry::inline_type_budget::collect(census);
}
