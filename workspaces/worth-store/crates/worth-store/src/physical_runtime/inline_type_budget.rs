//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::durability::inline_type_budget::collect(census);
    super::instance::inline_type_budget::collect(census);
    super::record_serving::inline_type_budget::collect(census);
}
