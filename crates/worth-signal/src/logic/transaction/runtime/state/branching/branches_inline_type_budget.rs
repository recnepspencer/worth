//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::authority::inline_type_budget::collect(census);
    super::catalog::inline_type_budget::collect(census);
    super::owner_partition::inline_type_budget::collect(census);
}
