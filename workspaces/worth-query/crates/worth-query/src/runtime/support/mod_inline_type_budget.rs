//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::authority_artifacts::inline_type_budget::collect(census);
    super::facade_families::inline_type_budget::collect(census);
}
