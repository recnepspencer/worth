//! Unit-only layout evidence; constructs no runtime, proof or authority.

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    super::current_root_owner::inline_type_budget::collect(census);
    super::failure::inline_type_budget::collect(census);
    super::namespace_durability::inline_type_budget::collect(census);
}
